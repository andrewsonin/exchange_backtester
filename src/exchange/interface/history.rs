use crate::exchange::{Exchange, interface::private::AggressiveOrderType};
use crate::history::types::{HistoryEvent, HistoryTickType, OrderOrigin};
use crate::input::InputInterface;
use crate::order::{MarketOrder, Order};
use crate::trader::{subscriptions::SubscriptionConfig, Trader};
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
            self.insert_limit_order::<HistoryEvent, { OrderOrigin::History }>(event);
            self.history_order_ids.insert(event.get_order_id());
        }
    }

    fn remove_prl_entry(&mut self, event: HistoryEvent)
    {
        let mut side_cursor = match event.get_order_direction() {
            Direction::Buy => { self.bids.cursor_front_mut() }
            Direction::Sell => { self.asks.cursor_front_mut() }
        };

        while let Some(ob_level) = side_cursor.current()
        {
            if ob_level.price != event.price {
                side_cursor.move_next();
                continue;
            }
            let mut level_cursor = ob_level.queue.cursor_front_mut();
            while let Some(limit_order) = level_cursor.current()
            {
                if limit_order.order_id == event.get_order_id()
                    && limit_order.from == OrderOrigin::History {
                    break;
                }
                level_cursor.move_next();
            }
            if let None = level_cursor.remove_current() {
                if DEBUG {
                    eprintln!(
                        "{} :: \
                        remove_prl_entry :: ERROR in case of non-trading Trader :: \
                        Order with such ID {:?} does not exist at the OB level with corresponding price: {:?}",
                        self.current_time,
                        event.get_order_id(),
                        event.price
                    )
                }
                break;
            }
            if ob_level.queue.is_empty() {
                side_cursor.remove_current();
            }
            if !self.history_order_ids.remove(&event.get_order_id()) && DEBUG {
                eprintln!(
                    "{} :: \
                    remove_prl_entry :: ERROR in case of non-trading Trader :: \
                    History order HashSet does not contain such ID: {:?}",
                    self.current_time,
                    event.get_order_id()
                )
            }
            return;
        }
        if DEBUG {
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
                        :: OB level with such price does not exist: {:?}",
                        self.current_time,
                        price
                    );
                }
                return;
            }
        };
        let order = match ob_level.queue
            .iter_mut()
            .skip_while(|order|
                order.order_id != event.order_id || order.from != OrderOrigin::History
            )
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
        order.size = event.size
    }

    fn handle_trd_event(&mut self, event: HistoryEvent)
    {
        self.insert_aggressive_order::<{ AggressiveOrderType::HistoryMarketOrder }>(
            MarketOrder::new(event.get_order_id(), event.get_order_size(), event.get_order_direction())
        )
    }
}