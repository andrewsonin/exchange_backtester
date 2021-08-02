use chrono::NaiveDateTime;

use crate::cli::InputInterface;
use crate::exchange::Exchange;
use crate::history::types::{HistoryEvent, HistoryTickType, OrderOrigin};
use crate::message::ExchangeReply::{OrderExecuted, OrderPartiallyExecuted};
use crate::order::Order;
use crate::trader::Trader;
use crate::types::{OrderDirection, OrderSize};

impl<T, TTC, NSC, PInfo> Exchange<'_, T, TTC, NSC, PInfo>
    where T: Trader,
          TTC: Fn(NaiveDateTime) -> bool,
          NSC: Fn(NaiveDateTime, NaiveDateTime) -> bool,
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
            self.schedule_history_event(event)
        }
    }

    fn handle_prl_event(&mut self, event: HistoryEvent)
    {
        if event.get_order_size() == OrderSize(0) {
            self.remove_prl_entry(event)
        } else if self.history_order_ids.contains(&event.get_order_id()) {
            self.update_traded_prl_entry(event)
        } else {
            self.insert_limit_order::<HistoryEvent, { OrderOrigin::History }>(event)
        }
    }

    fn remove_prl_entry(&mut self, event: HistoryEvent)
    {
        let mut cursor = match event.get_order_direction() {
            OrderDirection::Buy => { self.bids.cursor_front_mut() }
            OrderDirection::Sell => { self.asks.cursor_front_mut() }
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
                        eprintln!(
                            "Timestamp: {} :: \
                            remove_prl_entry :: \
                            Order with such ID does not exists at the OB level with corresponding price: {:?}",
                            self.current_time,
                            event.get_order_id()
                        )
                    }
                    _ => { deleted = true; }
                }
                if !self.history_order_ids.remove(&event.get_order_id()) {
                    eprintln!(
                        "Timestamp: {} :: \
                        remove_prl_entry :: \
                        History order HashSet does not contain such ID: {:?}",
                        self.current_time,
                        event.get_order_id()
                    )
                }
                break;
            }
        }
        if !deleted {
            eprintln!(
                "Timestamp: {} :: remove_prl_entry :: History order has not been deleted: {:?}",
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
            OrderDirection::Buy => { &mut self.bids }
            OrderDirection::Sell => { &mut self.asks }
        };
        let ob_level = match side
            .iter_mut()
            .skip_while(|level| level.price != price)
            .next()
        {
            Some(ob_level) => { ob_level }
            None => {
                eprintln!(
                    "Timestamp: {} :: \
                    update_traded_prl_entry :: \
                    OB level with such price does not exists: {:?}",
                    self.current_time,
                    event.order_id
                );
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
                eprintln!(
                    "Timestamp: {} \
                    :: update_traded_prl_entry \
                    :: OB level does not contain history order with such ID: {:?}",
                    self.current_time,
                    event.order_id
                );
                return;
            }
        };
        order.size = event.size;
    }

    fn handle_trd_event(&mut self, event: HistoryEvent)
    {
        let self_ptr = self as *mut Self;
        let price = event.price;
        let mut event = event.order_info;
        let mut side_cursor = match event.direction {
            OrderDirection::Buy => { self.asks.cursor_front_mut() }
            OrderDirection::Sell => { self.bids.cursor_front_mut() }
        };

        let mut check_ref_order: bool = true;
        while let Some(level) = side_cursor.current()
        {
            let limit_price = level.price;
            let mut level_cursor = level.queue.cursor_front_mut();
            while let Some(limit_order) = level_cursor.current()
            {
                let limit_order_id = limit_order.order_id;
                if event.size >= limit_order.size {
                    event.size -= limit_order.size;
                    match limit_order.from {
                        OrderOrigin::History => {
                            level_cursor.move_next()
                        }
                        OrderOrigin::Trader => {
                            unsafe { (*self_ptr).schedule_reply_for_trader(OrderExecuted(limit_order_id, limit_order.size, price)) }
                            level_cursor.remove_current();
                            check_ref_order = false;
                        }
                    }
                } else {
                    if limit_order.from == OrderOrigin::Trader {
                        limit_order.size -= event.size;
                        unsafe { (*self_ptr).schedule_reply_for_trader(OrderPartiallyExecuted(limit_order_id, event.size, price)) }
                    }
                    event.size = OrderSize(0);
                    level_cursor.move_next();
                }
                if check_ref_order {
                    if event.order_id != limit_order_id {
                        eprintln!(
                            "Timestamp: {} :: \
                            handle_trd_event :: \
                            market order {:?} matched with limit order {:?}",
                            self.current_time,
                            event.order_id,
                            limit_order_id
                        )
                    }
                    if price != limit_price {
                        eprintln!(
                            "Timestamp: {} :: \
                            handle_trd_event :: \
                            market order with {:?} matched with limit order with {:?}",
                            self.current_time,
                            price,
                            limit_price
                        )
                    }
                    if event.size != OrderSize(0) {
                        eprintln!(
                            "Timestamp: {} :: \
                            handle_trd_event :: \
                            market order with {:?} did not fully match with limit order {:?}",
                            self.current_time,
                            event.order_id,
                            limit_order_id
                        )
                    } else {
                        break;
                    }
                } else if event.size == OrderSize(0) {
                    break;
                }
            }
            if level.queue.len() != 0 {
                side_cursor.move_next()
            } else {
                side_cursor.remove_current();
            }
            if event.size == OrderSize(0) {
                return;
            }
        }
        eprintln!(
            "Timestamp: {} :: handle_trd_event :: \
            market order with {:?} did not fully executed. Its remaining size: {:?}",
            self.current_time,
            event.order_id,
            event.size
        )
    }
}