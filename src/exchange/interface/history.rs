use crate::exchange::{Exchange, interface::private::AggressiveOrderType};
use crate::history::{
    parser::EventProcessor,
    types::{HistoryEventBody, OrderOrigin},
};
use crate::order::{LimitOrder, Order, OrderInfo};
use crate::trader::{subscriptions::SubscriptionConfig, Trader};
use crate::types::{Direction, OrderID, Price, Size, Timestamp};

struct TRDummyOrder {
    size: Size,
    direction: Direction,
}

impl Order for TRDummyOrder {
    fn get_order_id(&self) -> OrderID { unreachable!("get_order_id could not be called for TrdDummyOrder") }
    fn get_order_size(&self) -> Size { self.size }
    fn mut_order_size(&mut self) -> &mut Size { &mut self.size }
    fn get_order_direction(&self) -> Direction { self.direction }
    fn extract_body(self) -> OrderInfo { unreachable!("extract_body could not be called for TrdDummyOrder") }
}

impl<T, TTC, EP, const DEBUG: bool, const SUBSCRIPTIONS: SubscriptionConfig>
Exchange<'_, T, TTC, EP, DEBUG, SUBSCRIPTIONS>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          EP: EventProcessor
{
    pub(crate)
    fn handle_history_event(&mut self, event: HistoryEventBody)
    {
        match event {
            HistoryEventBody::PRL((price, order_info)) => { self.handle_prl_event(price, order_info) }
            HistoryEventBody::TRD((size, direction)) => { self.handle_trd_event(size, direction) }
        }
        if let Some(event) = self.event_processor.yield_next_event() {
            self.event_queue.schedule_history_event(event)
        }
    }

    fn handle_prl_event(&mut self, price: Price, order_info: OrderInfo)
    {
        if order_info.size == Size(0) {
            self.remove_prl_entry(price, order_info)
        } else if self.history_order_ids.contains(&order_info.order_id) {
            self.update_traded_prl_entry(price, order_info)
        } else {
            self.insert_limit_order::<LimitOrder, { OrderOrigin::History }>(
                LimitOrder::new(order_info.order_id, order_info.size, order_info.direction, price)
            );
            self.history_order_ids.insert(order_info.order_id);
        }
    }

    fn remove_prl_entry(&mut self, price: Price, order_info: OrderInfo)
    {
        let mut side_cursor = match order_info.direction {
            Direction::Buy => { self.bids.cursor_front_mut() }
            Direction::Sell => { self.asks.cursor_front_mut() }
        };

        while let Some(ob_level) = side_cursor.current()
        {
            if ob_level.price != price {
                side_cursor.move_next();
                continue;
            }
            let mut level_cursor = ob_level.queue.cursor_front_mut();
            while let Some(limit_order) = level_cursor.current()
            {
                if limit_order.order_id == order_info.order_id
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
                        order_info.order_id,
                        price
                    )
                }
                break;
            }
            if ob_level.queue.is_empty() {
                side_cursor.remove_current();
            }
            if !self.history_order_ids.remove(&order_info.order_id) && DEBUG {
                eprintln!(
                    "{} :: \
                    remove_prl_entry :: ERROR in case of non-trading Trader :: \
                    History order HashSet does not contain such ID: {:?}",
                    self.current_time,
                    order_info.order_id
                )
            }
            return;
        }
        if DEBUG {
            eprintln!(
                "{} :: remove_prl_entry :: ERROR in case of non-trading Trader \
                :: History order has not been deleted: {:?}",
                self.current_time,
                order_info.order_id
            )
        }
    }

    fn update_traded_prl_entry(&mut self, price: Price, updated: OrderInfo)
    {
        let side = match updated.direction {
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
                order.order_id != updated.order_id || order.from != OrderOrigin::History
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
                        updated.order_id
                    );
                }
                return;
            }
        };
        order.size = updated.size
    }

    fn handle_trd_event(&mut self, size: Size, direction: Direction)
    {
        self.insert_aggressive_order::<TRDummyOrder, { AggressiveOrderType::HistoryMarketOrder }>(
            TRDummyOrder { size, direction }
        )
    }
}