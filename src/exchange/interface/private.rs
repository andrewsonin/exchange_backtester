use std::cmp::{Ordering, Reverse};

use crate::exchange::Exchange;
use crate::exchange::types::{Event, EventBody, OrderBookEntry, OrderBookLevel};
use crate::history::types::{HistoryEventWithTime, OrderOrigin};
use crate::input::InputInterface;
use crate::message::{CancellationReason, ExchangeReply};
use crate::message::ExchangeReply::{OrderCancelled, OrderExecuted, OrderPartiallyExecuted};
use crate::order::{MarketOrder, Order, PricedOrder};
use crate::trader::Trader;
use crate::types::{Duration, OrderDirection, OrderID, OrderSize, Price, Timestamp};

#[derive(Eq, PartialEq)]
pub(crate) enum AggressiveOrderType {
    IntersectingLimitOrder,
    MarketOrder,
}

impl<T, TTC, NSC, PInfo, const DEBUG: bool> Exchange<'_, T, TTC, NSC, PInfo, DEBUG>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          NSC: Fn(Timestamp, Timestamp) -> bool,
          PInfo: InputInterface
{
    pub(crate)
    fn cleanup(&mut self) {
        let self_ptr = self as *mut Self;
        self.history_order_ids.clear();
        self.bids.clear();
        self.asks.clear();
        self.trader_submitted_orders.clear();

        for id in self.trader_pending_market_orders.iter()
            .map(|order| order.get_order_id())
            .chain(self.trader_pending_limit_orders.keys().map(|id| *id))
        {
            let reply = OrderCancelled(id, CancellationReason::ExchangeClosed);
            unsafe { (*self_ptr).schedule_reply_for_trader(reply) }
        }
        self.trader_pending_market_orders.clear();
        self.trader_pending_limit_orders.clear();
    }

    pub(crate)
    fn insert_aggressive_order<const ORDER_ORIGIN: AggressiveOrderType>(&mut self,
                                                                        mut order: MarketOrder)
    {
        let self_ptr = self as *mut Self;
        let mut side_cursor = match order.get_order_direction() {
            OrderDirection::Buy => { self.asks.cursor_front_mut() }
            OrderDirection::Sell => { self.bids.cursor_front_mut() }
        };

        while let Some(level) = side_cursor.current()
        {
            let limit_price = level.price;
            let mut level_size = level.queue.len();
            let mut level_cursor = level.queue.cursor_front_mut();

            while let Some(mut limit_order) = level_cursor.current()
            {
                let init_limit_size = limit_order.size;
                let init_market_size = order.get_order_size();

                type F = fn(OrderID, OrderSize, Price) -> ExchangeReply;
                let (limit_status, market_status): (F, F) = match order.get_order_size().cmp(&limit_order.size) {
                    Ordering::Less => {
                        *order.mut_order_size() = OrderSize(0);
                        limit_order.size -= order.get_order_size();
                        (OrderPartiallyExecuted, OrderExecuted)
                    }
                    Ordering::Equal => {
                        *order.mut_order_size() = OrderSize(0);
                        limit_order.size = OrderSize(0);
                        (OrderExecuted, OrderExecuted)
                    }
                    Ordering::Greater => {
                        *order.mut_order_size() -= limit_order.size;
                        limit_order.size = OrderSize(0);
                        (OrderExecuted, OrderPartiallyExecuted)
                    }
                };
                if limit_order.from == OrderOrigin::Trader {
                    let reply = limit_status(limit_order.order_id, init_limit_size - limit_order.size, limit_price);
                    unsafe { (*self_ptr).schedule_reply_for_trader(reply) }
                    if limit_status == OrderExecuted {
                        self.trader_pending_limit_orders.remove(&limit_order.order_id);
                    }
                }
                let reply_status = match ORDER_ORIGIN {
                    AggressiveOrderType::IntersectingLimitOrder => { OrderPartiallyExecuted }
                    AggressiveOrderType::MarketOrder => { market_status }
                };
                let reply = reply_status(order.get_order_id(), init_market_size - order.get_order_size(), limit_price);
                unsafe { (*self_ptr).schedule_reply_for_trader(reply) }

                if limit_status == OrderPartiallyExecuted {
                    return;
                } else {
                    level_cursor.remove_current();
                    level_size -= 1;
                    if level_size == 0 {
                        side_cursor.remove_current();
                        if market_status == OrderExecuted { return; }
                        break;
                    }
                    if market_status == OrderExecuted { return; }
                }
            }
        }
        match ORDER_ORIGIN {
            AggressiveOrderType::IntersectingLimitOrder => {
                eprintln!(
                    "Timestamp :: {} :: submit_aggressive_order :: ERROR :: \
                    Intersecting market order {:?} has not been fully executed",
                    self.current_time,
                    order.get_order_id()
                )
            }
            AggressiveOrderType::MarketOrder => {
                self.trader_pending_market_orders.push_back(order)
            }
        }
    }

    pub(crate) fn insert_limit_order<O, const COME_FROM: OrderOrigin>(&mut self, order: O)
        where O: PricedOrder
    {
        let price = order.get_price();
        let mut order = order.extract_body();

        if COME_FROM == OrderOrigin::Trader {
            // Check that the Exchange have pending market orders
            let self_ptr = self as *mut Self;
            let mut cursor = self.trader_pending_market_orders.cursor_front_mut();
            while let Some(pending_order) = cursor.current() {
                if pending_order.get_order_direction() != order.direction {
                    let init_market_size = pending_order.get_order_size();

                    type F = fn(OrderID, OrderSize, Price) -> ExchangeReply;
                    let (market_status, limit_status, exec_size): (F, F, _) = match order.size.cmp(&init_market_size) {
                        Ordering::Less => {
                            (OrderPartiallyExecuted, OrderExecuted, order.size)
                        }
                        Ordering::Equal => {
                            (OrderExecuted, OrderExecuted, order.size)
                        }
                        Ordering::Greater => {
                            (OrderExecuted, OrderPartiallyExecuted, init_market_size)
                        }
                    };
                    let mkt_reply = market_status(pending_order.get_order_id(), exec_size, price);
                    let lim_reply = limit_status(order.order_id, exec_size, price);
                    unsafe {
                        (*self_ptr).schedule_reply_for_trader(mkt_reply);
                        (*self_ptr).schedule_reply_for_trader(lim_reply)
                    }
                    if market_status != OrderExecuted {
                        *pending_order.mut_order_size() -= exec_size;
                    } else {
                        cursor.remove_current();
                    }
                    if limit_status != OrderExecuted {
                        order.size -= exec_size;
                    } else {
                        return;
                    }
                } else {
                    cursor.move_next();
                }
            }

            // Check that the Trader submitted LimitOrder that intersects the opposite side of the Order Book
            let get_ob_level_size = |level: &OrderBookLevel|
                level.queue.iter()
                    .map(|limit_order| limit_order.size)
                    .sum();
            let intersection_size = match order.direction {
                OrderDirection::Buy => {
                    self.asks.iter()
                        .take_while(|level| level.price <= price)
                        .map(get_ob_level_size)
                        .sum()
                }
                OrderDirection::Sell => {
                    self.bids.iter()
                        .take_while(|level| level.price >= price)
                        .map(get_ob_level_size)
                        .sum()
                }
            };
            if intersection_size < order.size {
                if intersection_size != OrderSize(0) {
                    self.insert_intersecting_limit(
                        MarketOrder::new(order.order_id, intersection_size, order.direction)
                    )
                }
            } else {
                self.submit_market_order(
                    MarketOrder::new(order.order_id, order.size, order.direction)
                );
                return;
            }
        }

        // Insert Order in the Order Book
        let mut insert_new_level = true;
        let mut cursor = match order.direction {
            OrderDirection::Buy => {
                let mut cursor = self.bids.cursor_front_mut();
                while let Some(ob_level) = cursor.current() {
                    match ob_level.price.cmp(&price) {
                        Ordering::Less => { break; }
                        Ordering::Equal => {
                            insert_new_level = false;
                            break;
                        }
                        Ordering::Greater => { cursor.move_next() }
                    }
                }
                cursor
            }
            OrderDirection::Sell => {
                let mut cursor = self.asks.cursor_front_mut();
                while let Some(ob_level) = cursor.current() {
                    match ob_level.price.cmp(&price) {
                        Ordering::Greater => { break; }
                        Ordering::Equal => {
                            insert_new_level = false;
                            break;
                        }
                        Ordering::Less => { cursor.move_next() }
                    }
                }
                cursor
            }
        };
        if insert_new_level {
            cursor.insert_before(OrderBookLevel { price, queue: Default::default() });
            cursor.move_prev();
        }
        let ob_level = cursor.current().unwrap();
        ob_level.queue.push_back(
            OrderBookEntry {
                order_id: order.order_id,
                size: order.size,
                from: COME_FROM,
            }
        );
        if COME_FROM == OrderOrigin::Trader {
            self.trader_pending_limit_orders.insert(order.order_id, (price, order.direction));
        }
    }

    fn insert_intersecting_limit(&mut self, order: MarketOrder) {
        self.insert_aggressive_order::<{ AggressiveOrderType::IntersectingLimitOrder }>(order)
    }

    pub(crate) fn is_now_trading_time(&self) -> bool {
        (self._is_trading_time)(self.current_time)
    }

    pub(crate) fn is_next_session(&self, next_time: Timestamp) -> bool {
        (self._is_next_session)(self.current_time, next_time)
    }

    pub(crate) fn schedule_reply_for_trader(&mut self, reply: ExchangeReply) {
        self.event_queue.push(
            Reverse(
                Event {
                    timestamp: self.current_time + Duration::nanoseconds(self.trader.get_latency() as i64),
                    body: EventBody::ExchangeReply(reply),
                }
            )
        )
    }

    pub(crate) fn schedule_history_event(&mut self, event: HistoryEventWithTime) {
        self.event_queue.push(
            Reverse(
                Event {
                    timestamp: event.timestamp,
                    body: EventBody::HistoryEvent(event.event),
                }
            )
        )
    }

    pub(crate) fn set_new_trading_period(&mut self, first_event_time: Timestamp)
    {
        self.trader.set_new_trading_period();
        let trader_next_wakeup = first_event_time + Duration::nanoseconds(self.trader.get_wakeup_frequency_ns().get() as i64);
        if (self._is_trading_time)(trader_next_wakeup) {
            self.event_queue.push(Reverse(Event { timestamp: trader_next_wakeup, body: EventBody::WakeUp }))
        }
    }
}