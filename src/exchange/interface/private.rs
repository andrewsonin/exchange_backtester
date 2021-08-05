use std::cmp::Ordering;
use std::collections::LinkedList;

use crate::exchange::Exchange;
use crate::exchange::types::{Event, EventBody, OrderBookEntry, OrderBookLevel};
use crate::history::types::OrderOrigin;
use crate::input::InputInterface;
use crate::message::{CancellationReason, ExchangeReply, SubscriptionSchedule, SubscriptionUpdate};
use crate::message::ExchangeReply::{OrderCancelled, OrderExecuted, OrderPartiallyExecuted};
use crate::message::SubscriptionSchedule::{OrderBook, TradeInfo};
use crate::order::{MarketOrder, Order, PricedOrder};
use crate::trader::subscriptions::{OrderBookSnapshot, SubscriptionConfig};
use crate::trader::Trader;
use crate::types::{Duration, OrderDirection, OrderID, OrderSize, Price, Timestamp};

#[derive(Eq, PartialEq)]
pub(crate) enum AggressiveOrderType {
    IntersectingLimitOrder,
    MarketOrder,
}

impl<T, TTC, PInfo, const DEBUG: bool, const SUBSCRIPTIONS: SubscriptionConfig>
Exchange<'_, T, TTC, PInfo, DEBUG, SUBSCRIPTIONS>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          PInfo: InputInterface
{
    pub(crate)
    fn cleanup(&mut self) {
        self.history_order_ids.clear();
        self.bids.clear();
        self.asks.clear();
        self.trader_submitted_orders.clear();

        for id in self.trader_pending_market_orders.iter()
            .map(|order| order.get_order_id())
            .chain(self.trader_pending_limit_orders.keys().map(|id| *id))
        {
            let reply = OrderCancelled(id, CancellationReason::ExchangeClosed);
            self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
        }
        self.trader_pending_market_orders.clear();
        self.trader_pending_limit_orders.clear();
    }

    pub(crate)
    fn insert_aggressive_order<const ORDER_ORIGIN: AggressiveOrderType>(&mut self,
                                                                        mut order: MarketOrder)
    {
        let mut side_cursor = match order.get_order_direction() {
            OrderDirection::Buy => { self.asks.cursor_front_mut() }
            OrderDirection::Sell => { self.bids.cursor_front_mut() }
        };

        while let Some(level) = side_cursor.current()
        {
            let limit_price = level.price;
            let mut level_size = level.queue.len();
            let mut level_cursor = level.queue.cursor_front_mut();

            while let Some(limit_order) = level_cursor.current()
            {
                type F = fn(OrderID, OrderSize, Price) -> ExchangeReply;
                let (limit_status, market_status, exec_size): (F, F, _) = match order.get_order_size().cmp(&limit_order.size) {
                    Ordering::Less => {
                        (OrderPartiallyExecuted, OrderExecuted, order.get_order_size())
                    }
                    Ordering::Equal => {
                        (OrderExecuted, OrderExecuted, limit_order.size)
                    }
                    Ordering::Greater => {
                        (OrderExecuted, OrderPartiallyExecuted, limit_order.size)
                    }
                };
                *order.mut_order_size() -= exec_size;
                limit_order.size -= exec_size;

                if limit_order.from == OrderOrigin::Trader {
                    let reply = limit_status(limit_order.order_id, exec_size, limit_price);
                    self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                    if limit_status == OrderExecuted {
                        self.trader_pending_limit_orders.remove(&limit_order.order_id);
                    }
                }
                let reply_status = match ORDER_ORIGIN {
                    AggressiveOrderType::IntersectingLimitOrder => { OrderPartiallyExecuted }
                    AggressiveOrderType::MarketOrder => { market_status }
                };
                let reply = reply_status(order.get_order_id(), exec_size, limit_price);
                self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);

                self.executed_trades.push((limit_price, exec_size, order.get_order_direction()));

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
                    "{} :: submit_aggressive_order :: ERROR :: \
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
            let mut cursor = self.trader_pending_market_orders.cursor_front_mut();
            while let Some(pending_order) = cursor.current()
            {
                if pending_order.get_order_direction() != order.direction
                {
                    type F = fn(OrderID, OrderSize, Price) -> ExchangeReply;
                    let (market_status, limit_status, exec_size): (F, F, _) = match order.size.cmp(&pending_order.get_order_size()) {
                        Ordering::Less => {
                            (OrderPartiallyExecuted, OrderExecuted, order.size)
                        }
                        Ordering::Equal => {
                            (OrderExecuted, OrderExecuted, order.size)
                        }
                        Ordering::Greater => {
                            (OrderExecuted, OrderPartiallyExecuted, pending_order.get_order_size())
                        }
                    };
                    let mkt_reply = market_status(pending_order.get_order_id(), exec_size, price);
                    let lim_reply = limit_status(order.order_id, exec_size, price);

                    self.event_queue.schedule_reply_for_trader(mkt_reply, self.current_time, self.trader);
                    self.event_queue.schedule_reply_for_trader(lim_reply, self.current_time, self.trader);

                    self.executed_trades.push((price, exec_size, pending_order.get_order_direction()));
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
            let intersection_size = match order.direction {
                OrderDirection::Buy => {
                    self.asks.iter()
                        .take_while(|level| level.price <= price)
                        .map(OrderBookLevel::get_ob_level_size)
                        .sum()
                }
                OrderDirection::Sell => {
                    self.bids.iter()
                        .take_while(|level| level.price >= price)
                        .map(OrderBookLevel::get_ob_level_size)
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
        (self.is_trading_time)(self.current_time)
    }

    pub(crate) fn set_new_trading_period(&mut self)
    {
        self.executed_trades.clear();
        if let Some((_, freq)) = SUBSCRIPTIONS.ob_depth_and_interval_ns {
            let next_time = self.current_time + Duration::nanoseconds(freq.get() as i64);
            if (self.is_trading_time)(next_time) {
                self.event_queue.push(
                    Event {
                        timestamp: next_time,
                        body: EventBody::SubscriptionSchedule(OrderBook),
                    }
                )
            }
        }
        if let Some(freq) = SUBSCRIPTIONS.trade_info_interval_ns {
            let next_time = self.current_time + Duration::nanoseconds(freq.get() as i64);
            if (self.is_trading_time)(next_time) {
                self.event_queue.push(
                    Event {
                        timestamp: next_time,
                        body: EventBody::SubscriptionSchedule(TradeInfo),
                    }
                )
            }
        }
        self.trader.set_new_trading_period();
    }

    pub(crate) fn handle_subscription_schedule(&mut self, subscription_type: SubscriptionSchedule) {
        match subscription_type {
            SubscriptionSchedule::OrderBook => {
                if let Some((depth, freq)) = SUBSCRIPTIONS.ob_depth_and_interval_ns {
                    let get_snapshot = |ob_side: &LinkedList<OrderBookLevel>| {
                        ob_side.iter()
                            .enumerate()
                            .take_while(|(i, _)| *i != depth)
                            .map(|(_, level)| (level.price, level.get_ob_level_size()))
                            .collect::<Vec<_>>()
                    };
                    self.event_queue.push(
                        Event {
                            timestamp: self.current_time + Duration::nanoseconds(self.trader.exchange_to_trader_latency() as i64),
                            body: EventBody::SubscriptionUpdate(SubscriptionUpdate::OrderBook(
                                OrderBookSnapshot {
                                    bids: get_snapshot(&self.bids),
                                    asks: get_snapshot(&self.asks),
                                }
                            )),
                        }
                    );
                    let next_time = self.current_time + Duration::nanoseconds(freq.get() as i64);
                    if (self.is_trading_time)(next_time) {
                        self.event_queue.push(
                            Event {
                                timestamp: next_time,
                                body: EventBody::SubscriptionSchedule(OrderBook),
                            }
                        )
                    }
                }
            }
            SubscriptionSchedule::TradeInfo => {
                if let Some(freq) = SUBSCRIPTIONS.trade_info_interval_ns {
                    self.event_queue.push(
                        Event {
                            timestamp: self.current_time + Duration::nanoseconds(self.trader.exchange_to_trader_latency() as i64),
                            body: EventBody::SubscriptionUpdate(SubscriptionUpdate::TradeInfo(
                                self.executed_trades.get_trade_info()
                            )),
                        }
                    );
                    self.executed_trades.clear();
                    let next_time = self.current_time + Duration::nanoseconds(freq.get() as i64);
                    if (self.is_trading_time)(next_time) {
                        self.event_queue.push(
                            Event {
                                timestamp: next_time,
                                body: EventBody::SubscriptionSchedule(TradeInfo),
                            }
                        )
                    }
                }
            }
        }
    }
}