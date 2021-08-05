use crate::exchange::Exchange;
use crate::history::types::{HistoryEvent, HistoryTickType, OrderOrigin};
use crate::input::InputInterface;
use crate::message::ExchangeReply::{OrderExecuted, OrderPartiallyExecuted};
use crate::order::Order;
use crate::trader::subscriptions::SubscriptionConfig;
use crate::trader::Trader;
use crate::types::{Direction, Size, Timestamp};

impl<T, TTC, PInfo, const DEBUG: bool, const SUBSCRIPTIONS: SubscriptionConfig>
Exchange<'_, T, TTC, PInfo, DEBUG, SUBSCRIPTIONS>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          PInfo: InputInterface
{
    pub(crate)
    fn handle_history_event(&mut self, event: HistoryEvent)
    {
        match event.tick_type {
            HistoryTickType::PRL => { self.handle_prl_event(event) }
            HistoryTickType::TRD => { self.handle_trd_event(event) }
        }
        if let Some(event) = self.history_reader.yield_next_event() {
            self.event_queue.schedule_history_event(event)
        }
    }

    fn handle_prl_event(&mut self, event: HistoryEvent)
    {
        if event.get_order_size() == Size(0) {
            self.remove_prl_entry(event)
        } else if self.history_order_ids.contains(&event.get_order_id()) {
            self.update_traded_prl_entry(event)
        } else {
            self.history_order_ids.insert(event.get_order_id());
            self.insert_limit_order::<HistoryEvent, { OrderOrigin::History }>(event)
        }
    }

    fn remove_prl_entry(&mut self, event: HistoryEvent)
    {
        let mut cursor = match event.get_order_direction() {
            Direction::Buy => { self.bids.cursor_front_mut() }
            Direction::Sell => { self.asks.cursor_front_mut() }
        };

        let mut deleted = false;
        while let Some(ob_level) = cursor.current()
        {
            if ob_level.price != event.price {
                cursor.move_next()
            } else {
                let mut cursor = ob_level.queue.cursor_front_mut();
                while let Some(limit_order) = cursor.current()
                {
                    if limit_order.from == OrderOrigin::History
                        && limit_order.order_id == event.get_order_id() {
                        break;
                    }
                    cursor.move_next();
                }
                match cursor.remove_current() {
                    None => {
                        if DEBUG {
                            eprintln!(
                                "{} :: \
                                remove_prl_entry :: ERROR in case of non-trading Trader :: \
                                Order with such ID does not exists at the OB level with corresponding price: {:?}",
                                self.current_time,
                                event.get_order_id()
                            )
                        }
                    }
                    _ => { deleted = true; }
                }
                if DEBUG && !self.history_order_ids.remove(&event.get_order_id()) {
                    eprintln!(
                        "{} :: \
                        remove_prl_entry :: ERROR in case of non-trading Trader :: \
                        History order HashSet does not contain such ID: {:?}",
                        self.current_time,
                        event.get_order_id()
                    )
                }
                break;
            }
        }
        if DEBUG && !deleted {
            eprintln!(
                "{} :: remove_prl_entry :: ERROR in case of non-trading Trader \
                :: History order has not been deleted: {:?}",
                self.current_time,
                event.get_order_id()
            )
        }
    }

    fn update_traded_prl_entry(&mut self, event: HistoryEvent)
    {
        let price = event.price;
        let event = event.order_info;
        let side = match event.direction {
            Direction::Buy => { &mut self.bids }
            Direction::Sell => { &mut self.asks }
        };
        let ob_level = match side
            .iter_mut()
            .skip_while(|level| level.price != price)
            .next()
        {
            Some(ob_level) => { ob_level }
            None => {
                if DEBUG {
                    eprintln!(
                        "{} \
                        :: update_traded_prl_entry :: ERROR in case of non-trading Trader \
                        :: OB level with such price does not exists: {:?}",
                        self.current_time,
                        event.order_id
                    );
                }
                return;
            }
        };
        let order = match ob_level.queue
            .iter_mut()
            .filter(|order| order.from == OrderOrigin::History)
            .skip_while(|order| order.order_id != event.order_id)
            .next()
        {
            Some(order) => { order }
            None => {
                if DEBUG {
                    eprintln!(
                        "{} \
                         :: update_traded_prl_entry :: ERROR in case of non-trading Trader \
                         :: OB level does not contain history order with such ID: {:?}",
                        self.current_time,
                        event.order_id
                    );
                }
                return;
            }
        };
        order.size = event.size;
    }

    fn handle_trd_event(&mut self, event: HistoryEvent)
    {
        let price = event.price;
        let mut event = event.order_info;
        let mut side_cursor = match event.direction {
            Direction::Buy => { self.asks.cursor_front_mut() }
            Direction::Sell => { self.bids.cursor_front_mut() }
        };

        let mut check_ref_order: bool = true;
        while let Some(level) = side_cursor.current()
        {
            let limit_price = level.price;
            let mut level_cursor = level.queue.cursor_front_mut();
            while let Some(limit_order) = level_cursor.current()
            {
                let limit_order_id = limit_order.order_id;
                let exec_size = if event.size >= limit_order.size {
                    let exec_size = limit_order.size;
                    event.size -= exec_size;
                    match limit_order.from {
                        OrderOrigin::History => {
                            level_cursor.move_next()
                        }
                        OrderOrigin::Trader => {
                            self.event_queue.schedule_reply_for_trader(
                                OrderExecuted(limit_order_id, limit_order.size, price),
                                self.current_time,
                                self.trader,
                            );
                            level_cursor.remove_current();
                            check_ref_order = false;
                        }
                    }
                    exec_size
                } else {
                    let exec_size = event.size;
                    if limit_order.from == OrderOrigin::Trader {
                        limit_order.size -= exec_size;
                        self.event_queue.schedule_reply_for_trader(
                            OrderPartiallyExecuted(limit_order_id, event.size, price),
                            self.current_time,
                            self.trader,
                        );
                    }
                    event.size = Size(0);
                    level_cursor.move_next();
                    exec_size
                };
                self.executed_trades.push((limit_price, exec_size, event.direction));
                if DEBUG && check_ref_order {
                    if event.order_id != limit_order_id {
                        eprintln!(
                            "{} :: \
                            handle_trd_event :: ERROR in case of non-trading Trader :: \
                            market order {:?} matched with limit order {:?}",
                            self.current_time,
                            event.order_id,
                            limit_order_id
                        )
                    }
                    if price != limit_price {
                        eprintln!(
                            "{} :: \
                            handle_trd_event :: ERROR in case of non-trading Trader :: \
                            market order with {:?} matched with limit order with {:?}",
                            self.current_time,
                            price,
                            limit_price
                        )
                    }
                    if event.size != Size(0) {
                        eprintln!(
                            "{} :: \
                            handle_trd_event :: ERROR in case of non-trading Trader :: \
                            market order with {:?} did not fully match with limit order {:?}",
                            self.current_time,
                            event.order_id,
                            limit_order_id
                        )
                    } else {
                        break;
                    }
                } else if event.size == Size(0) {
                    break;
                }
            }
            if level.queue.len() != 0 {
                side_cursor.move_next()
            } else {
                side_cursor.remove_current();
            }
            if event.size == Size(0) {
                return;
            }
        }
        if DEBUG {
            eprintln!(
                "{} :: handle_trd_event :: ERROR in case of non-trading Trader :: \
                market order with {:?} did not fully executed. Its remaining size: {:?}",
                self.current_time,
                event.order_id,
                event.size
            )
        }
    }
}