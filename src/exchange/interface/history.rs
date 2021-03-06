use crate::exchange::{Exchange, interface::private::AggressiveOrderType};
use crate::history::{
    parser::EventProcessor,
    types::{HistoryEventBody, OrderOrigin},
};
use crate::lags::interface::NanoSecondGenerator;
use crate::order::{LimitOrder, Order};
use crate::trader::Trader;
use crate::types::{Direction, OrderID, Price, Size};

struct TRDummyOrder {
    size: Size,
    direction: Direction,
}

impl const Order for TRDummyOrder {
    fn get_order_id(&self) -> OrderID { unreachable!() }
    fn get_order_size(&self) -> Size { self.size }
    fn mut_order_size(&mut self) -> &mut Size { &mut self.size }
    fn get_order_direction(&self) -> Direction { self.direction }
}

impl<
    T: Trader,
    E: EventProcessor,
    ObLagGen: NanoSecondGenerator,
    TrdLagGen: NanoSecondGenerator,
    WkpLagGen: NanoSecondGenerator,
    const DEBUG: bool,
    const TRD_UPDATES_OB: bool,
    const OB_SUBSCRIPTION: bool,
    const TRD_SUBSCRIPTION: bool,
    const WAKEUP_SUBSCRIPTION: bool
>
Exchange<'_, T, E, ObLagGen, TrdLagGen, WkpLagGen, DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION>
{
    pub(crate)
    fn handle_history_event(&mut self, event: HistoryEventBody)
    {
        match event {
            HistoryEventBody::OrderBookDiff(size, direction, price, order_id) => {
                self.handle_ob_diff_event(size, direction, price, order_id)
            }
            HistoryEventBody::Trade(size, direction) => {
                self.handle_trd_event(size, direction)
            }
        }
        if let Some(event) = self.event_processor.yield_next_event() {
            self.event_queue.schedule_history_event(event)
        } else {
            self.has_history_events_in_queue = false;
        }
    }

    fn handle_ob_diff_event(&mut self, size: Size, direction: Direction, price: Price, order_id: OrderID)
    {
        if size == Size(0) {
            self.remove_ob_entry(direction, price, order_id)
        } else if self.history_order_ids.contains(&order_id) {
            self.update_traded_ob_entry(size, direction, price, order_id)
        } else {
            self.insert_limit_order::<LimitOrder, { OrderOrigin::History }>(
                LimitOrder::new(order_id, size, direction, price)
            );
            self.history_order_ids.insert(order_id);
        }
    }

    fn remove_ob_entry(&mut self, direction: Direction, price: Price, order_id: OrderID)
    {
        let mut side_cursor = match direction {
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
                if limit_order.order_id == order_id && limit_order.from == OrderOrigin::History {
                    break;
                }
                level_cursor.move_next();
            }
            if let None = level_cursor.remove_current() {
                if DEBUG {
                    eprintln!(
                        "{} :: \
                        remove_ob_entry :: ERROR in case of non-trading Trader :: \
                        Order with such ID {:?} does not exist at the OB level with corresponding price: {:?}",
                        self.current_dt,
                        order_id,
                        price
                    )
                }
                break;
            }
            if ob_level.queue.is_empty() {
                side_cursor.remove_current();
            }
            if !self.history_order_ids.remove(&order_id) && DEBUG {
                eprintln!(
                    "{} :: \
                    remove_ob_entry :: ERROR in case of non-trading Trader :: \
                    History order HashSet does not contain such ID: {:?}",
                    self.current_dt,
                    order_id
                )
            }
            return;
        }
        if DEBUG {
            eprintln!(
                "{} :: remove_ob_entry :: ERROR in case of non-trading Trader \
                :: History order has not been deleted: {:?}",
                self.current_dt,
                order_id
            )
        }
    }

    fn update_traded_ob_entry(&mut self, size: Size, direction: Direction, price: Price, order_id: OrderID)
    {
        let side = match direction {
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
                        :: update_traded_ob_entry :: ERROR in case of non-trading Trader \
                        :: OB level with such price does not exist: {:?}",
                        self.current_dt,
                        price
                    );
                }
                return;
            }
        };
        let order = match ob_level.queue
            .iter_mut()
            .skip_while(|order|
                order.order_id != order_id || order.from != OrderOrigin::History
            )
            .next()
        {
            Some(order) => { order }
            None => {
                if DEBUG {
                    eprintln!(
                        "{} \
                         :: update_traded_ob_entry :: ERROR in case of non-trading Trader \
                         :: OB level does not contain history order with such ID: {:?}",
                        self.current_dt,
                        order_id
                    );
                }
                return;
            }
        };
        order.size = size
    }

    fn handle_trd_event(&mut self, size: Size, direction: Direction)
    {
        self.insert_aggressive_order::<TRDummyOrder, { AggressiveOrderType::HistoryMarketOrder }>(
            TRDummyOrder { size, direction }
        )
    }
}