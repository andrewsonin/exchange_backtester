use std::{cmp::Ordering, collections::LinkedList, iter::FromIterator};

use AggressiveOrderType::*;

use crate::exchange::{Exchange, types::{Event, EventBody, OrderBookEntry, OrderBookLevel}};
use crate::history::{parser::EventProcessor, types::OrderOrigin};
use crate::message::{
    CancellationReason,
    DiscardingReason::ZeroSize,
    ExchangeReply::{OrderCancelled, OrderExecuted, OrderPartiallyExecuted, OrderPlacementDiscarded},
    ExchangeReply,
    SubscriptionSchedule::{OrderBook, TradeInfo},
    SubscriptionSchedule,
    SubscriptionUpdate,
    TraderRequest::{CancelLimitOrder, CancelMarketOrder, PlaceLimitOrder, PlaceMarketOrder},
    TraderRequest,
};
use crate::order::{MarketOrder, Order, PricedOrder};
use crate::trader::{subscriptions::{OrderBookSnapshot, SubscriptionConfig}, Trader};
use crate::types::{Direction, Duration, Size, Timestamp};
use crate::utils::ExpectWith;

#[derive(Eq, PartialEq)]
pub(crate) enum AggressiveOrderType {
    TraderMarketOrder,
    HistoryMarketOrder,
    TraderIntersectingLimitOrder,
    HistoryIntersectingLimitOrder,
}

impl<T, TTC, EP, const DEBUG: bool, const TRD_UPDATES_OB: bool, const SUBSCRIPTIONS: SubscriptionConfig>
Exchange<'_, T, TTC, EP, DEBUG, TRD_UPDATES_OB, SUBSCRIPTIONS>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          EP: EventProcessor
{
    pub(crate)
    fn cleanup(&mut self) {
        self.history_order_ids.clear();
        self.bids.clear();
        self.asks.clear();
        self.trader_submitted_orders.clear();
        self.executed_trades.clear();

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

    const fn react_with_history_limit_orders<const ORDER_TYPE: AggressiveOrderType>() -> bool {
        match (ORDER_TYPE, TRD_UPDATES_OB) {
            (TraderMarketOrder | TraderIntersectingLimitOrder, _) | (_, true) => { true }
            _ => { false }
        }
    }

    const fn is_trader_aggressive_order<const ORDER_TYPE: AggressiveOrderType>() -> bool {
        if let TraderMarketOrder | TraderIntersectingLimitOrder = ORDER_TYPE {
            true
        } else {
            false
        }
    }

    pub(crate)
    fn insert_aggressive_order<O, const ORDER_TYPE: AggressiveOrderType>(&mut self, mut order: O)
        where O: Order
    {
        let mut side_cursor = match order.get_order_direction() {
            Direction::Buy => { self.asks.cursor_front_mut() }
            Direction::Sell => { self.bids.cursor_front_mut() }
        };

        while let Some(level) = side_cursor.current()
        {
            let price = level.price;
            let mut level_cursor = level.queue.cursor_front_mut();
            let mut limit_order = level_cursor.current().expect_with(
                || format!("Level at price {:?} does not have any orders", price)
            );

            loop {
                match order.get_order_size().cmp(&limit_order.size)
                {
                    Ordering::Less => {
                        // (OrderExecuted, OrderPartiallyExecuted)
                        let exec_size = order.get_order_size();
                        if SUBSCRIPTIONS.trade_info_interval_ns.is_some() {
                            self.executed_trades.push((price, exec_size, order.get_order_direction()))
                        }
                        match ORDER_TYPE {
                            TraderMarketOrder => {
                                let reply = OrderExecuted(order.get_order_id(), exec_size, price);
                                self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader)
                            }
                            TraderIntersectingLimitOrder => {
                                let reply = OrderPartiallyExecuted(order.get_order_id(), exec_size, price);
                                self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader)
                            }
                            _ => {}
                        }
                        if limit_order.from == OrderOrigin::Trader {
                            let reply = OrderPartiallyExecuted(limit_order.order_id, exec_size, price);
                            self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                            limit_order.size -= exec_size
                        } else if Self::react_with_history_limit_orders::<ORDER_TYPE>() {
                            limit_order.size -= exec_size
                        }
                        return;
                    }
                    Ordering::Equal => {
                        // (OrderExecuted, OrderExecuted)
                        let exec_size = order.get_order_size();
                        if SUBSCRIPTIONS.trade_info_interval_ns.is_some() {
                            self.executed_trades.push((price, exec_size, order.get_order_direction()))
                        }
                        match ORDER_TYPE {
                            TraderMarketOrder => {
                                let reply = OrderExecuted(order.get_order_id(), exec_size, price);
                                self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                            }
                            TraderIntersectingLimitOrder => {
                                let reply = OrderPartiallyExecuted(order.get_order_id(), exec_size, price);
                                self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                            }
                            _ => {}
                        }
                        if limit_order.from == OrderOrigin::Trader {
                            let reply = OrderExecuted(limit_order.order_id, exec_size, price);
                            self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                            self.trader_pending_limit_orders.remove(&limit_order.order_id);
                            level_cursor.remove_current();
                            if level.queue.is_empty() {
                                side_cursor.remove_current();
                            }
                        } else if Self::react_with_history_limit_orders::<ORDER_TYPE>() {
                            level_cursor.remove_current();
                            if level.queue.is_empty() {
                                side_cursor.remove_current();
                            }
                        };
                        return;
                    }
                    Ordering::Greater => {
                        // (OrderPartiallyExecuted, OrderExecuted)
                        let exec_size = limit_order.size;
                        *order.mut_order_size() -= exec_size;
                        if SUBSCRIPTIONS.trade_info_interval_ns.is_some() {
                            self.executed_trades.push((price, exec_size, order.get_order_direction()))
                        }
                        if Self::is_trader_aggressive_order::<ORDER_TYPE>() {
                            let reply = OrderPartiallyExecuted(order.get_order_id(), exec_size, price);
                            self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                        }
                        match limit_order.from {
                            OrderOrigin::History => {
                                if Self::react_with_history_limit_orders::<ORDER_TYPE>() {
                                    level_cursor.remove_current();
                                    match level_cursor.current() {
                                        Some(entry) => { limit_order = entry }
                                        None => {
                                            if level.queue.is_empty() {
                                                side_cursor.remove_current();
                                            } else {
                                                side_cursor.move_next();
                                            }
                                            break;
                                        }
                                    }
                                } else {
                                    level_cursor.move_next();
                                    match level_cursor.current() {
                                        Some(entry) => { limit_order = entry }
                                        None => {
                                            side_cursor.move_next();
                                            break;
                                        }
                                    }
                                }
                            }
                            OrderOrigin::Trader => {
                                let reply = OrderExecuted(limit_order.order_id, exec_size, price);
                                self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                                self.trader_pending_limit_orders.remove(&limit_order.order_id);
                                level_cursor.remove_current();
                                match level_cursor.current() {
                                    Some(entry) => { limit_order = entry }
                                    None => {
                                        if level.queue.is_empty() {
                                            side_cursor.remove_current();
                                        } else {
                                            side_cursor.move_next();
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        match ORDER_TYPE {
            TraderMarketOrder => {
                self.trader_pending_market_orders.push_back(
                    MarketOrder::new(order.get_order_id(), order.get_order_size(), order.get_order_direction())
                )
            }
            HistoryMarketOrder => {
                if DEBUG {
                    eprintln!(
                        "{} :: insert_aggressive_order<HistoryMarketOrder> :: ERROR in case of non-trading Trader :: \
                        market order with {:?} did not fully executed. Its remaining size: {:?}",
                        self.current_time,
                        order.get_order_id(),
                        order.get_order_size()
                    )
                }
            }
            TraderIntersectingLimitOrder | HistoryIntersectingLimitOrder => {
                panic!("{}. Intersection LimitOrder has not been fully executed", self.current_time)
            }
        }
    }

    pub(crate) fn insert_limit_order<O, const COME_FROM: OrderOrigin>(&mut self, mut order: O)
        where O: PricedOrder
    {
        let price = order.get_price();

        // Check that the Exchange have pending market orders
        let mut cursor = self.trader_pending_market_orders.cursor_front_mut();
        while let Some(pending) = cursor.current()
        {
            if pending.get_order_direction() == order.get_order_direction() {
                cursor.move_next();
                continue;
            }
            match order.get_order_size().cmp(&pending.get_order_size()) {
                Ordering::Less => {
                    // (OrderExecuted, OrderPartiallyExecuted)
                    let exec_size = order.get_order_size();
                    *pending.mut_order_size() -= exec_size;
                    let reply = OrderPartiallyExecuted(pending.get_order_id(), exec_size, price);
                    self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                    if COME_FROM == OrderOrigin::Trader {
                        let reply = OrderExecuted(order.get_order_id(), exec_size, price);
                        self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                    }
                    if SUBSCRIPTIONS.trade_info_interval_ns.is_some() {
                        self.executed_trades.push((price, exec_size, pending.get_order_direction()))
                    }
                    return;
                }
                Ordering::Equal => {
                    // (OrderExecuted, OrderExecuted)
                    let exec_size = order.get_order_size();
                    let reply = OrderExecuted(pending.get_order_id(), exec_size, price);
                    self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                    if COME_FROM == OrderOrigin::Trader {
                        let reply = OrderExecuted(order.get_order_id(), exec_size, price);
                        self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                    }
                    if SUBSCRIPTIONS.trade_info_interval_ns.is_some() {
                        self.executed_trades.push((price, exec_size, pending.get_order_direction()))
                    }
                    cursor.remove_current();
                    return;
                }
                Ordering::Greater => {
                    // (OrderPartiallyExecuted, OrderExecuted)
                    let exec_size = pending.get_order_size();
                    *order.mut_order_size() -= exec_size;
                    let reply = OrderExecuted(pending.get_order_id(), exec_size, price);
                    self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                    if COME_FROM == OrderOrigin::Trader {
                        let reply = OrderPartiallyExecuted(order.get_order_id(), exec_size, price);
                        self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
                    }
                    if SUBSCRIPTIONS.trade_info_interval_ns.is_some() {
                        self.executed_trades.push((price, exec_size, pending.get_order_direction()))
                    }
                    cursor.remove_current();
                }
            }
        }

        // Check whether LimitOrder intersects the opposite side of the Order Book
        let intersection_size = match order.get_order_direction() {
            Direction::Buy => {
                self.asks.iter()
                    .take_while(|level| level.price <= price)
                    .map(OrderBookLevel::get_ob_level_size)
                    .sum()
            }
            Direction::Sell => {
                self.bids.iter()
                    .take_while(|level| level.price >= price)
                    .map(OrderBookLevel::get_ob_level_size)
                    .sum()
            }
        };
        if intersection_size < order.get_order_size() {
            if intersection_size != Size(0) {
                let order = MarketOrder::new(order.get_order_id(), intersection_size, order.get_order_direction());
                match COME_FROM {
                    OrderOrigin::History => {
                        self.insert_aggressive_order::<MarketOrder, { HistoryIntersectingLimitOrder }>(order)
                    }
                    OrderOrigin::Trader => {
                        self.insert_aggressive_order::<MarketOrder, { TraderIntersectingLimitOrder }>(order)
                    }
                }
            }
        } else {
            let order = MarketOrder::new(order.get_order_id(), order.get_order_size(), order.get_order_direction());
            match COME_FROM {
                OrderOrigin::History => {
                    self.insert_aggressive_order::<MarketOrder, { AggressiveOrderType::HistoryMarketOrder }>(order)
                }
                OrderOrigin::Trader => {
                    self.insert_aggressive_order::<MarketOrder, { AggressiveOrderType::TraderMarketOrder }>(order)
                }
            }
            return;
        }

        // Insert Order in the Order Book
        let mut insert_new_level = true;
        let mut cursor = match order.get_order_direction() {
            Direction::Buy => {
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
            Direction::Sell => {
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
            let iter = [OrderBookEntry { order_id: order.get_order_id(), size: order.get_order_size(), from: COME_FROM }];
            cursor.insert_before(OrderBookLevel { price, queue: FromIterator::from_iter(iter) })
        } else {
            cursor.current().unwrap().queue.push_back(
                OrderBookEntry {
                    order_id: order.get_order_id(),
                    size: order.get_order_size(),
                    from: COME_FROM,
                }
            )
        }
        if COME_FROM == OrderOrigin::Trader {
            self.trader_pending_limit_orders.insert(order.get_order_id(), (price, order.get_order_direction()));
        }
    }

    pub(crate) fn is_now_trading_time(&self) -> bool {
        (self.is_trading_time)(self.current_time)
    }

    pub(crate) fn set_new_trading_period(&mut self)
    {
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
        if let Some(freq) = SUBSCRIPTIONS.wakeup {
            let next_time = self.current_time + Duration::nanoseconds(freq.get() as i64);
            if (self.is_trading_time)(next_time) {
                self.event_queue.push(
                    Event {
                        timestamp: next_time,
                        body: EventBody::TraderWakeUp,
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
                } else {
                    unreachable!()
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
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub(crate) fn handle_exchange_reply(&mut self, reply: ExchangeReply) {
        let current_time = self.current_time;
        let trader_reactions = self.trader.handle_exchange_reply(reply);
        let trader = &mut self.trader;
        self.event_queue.extend(
            trader_reactions.into_iter()
                .map(
                    |request| Event {
                        timestamp: current_time + Duration::nanoseconds(trader.trader_to_exchange_latency() as i64),
                        body: EventBody::TraderRequest(request),
                    }
                )
        )
    }

    pub(crate) fn handle_trader_request(&mut self, request: TraderRequest) {
        match request {
            PlaceLimitOrder(order) => {
                if order.get_order_size() != Size(0) {
                    self.submit_limit_order(order)
                } else {
                    self.event_queue.schedule_reply_for_trader(
                        OrderPlacementDiscarded(order.get_order_id(), ZeroSize), self.current_time, self.trader,
                    )
                }
            }
            PlaceMarketOrder(order) => {
                if order.get_order_size() != Size(0) {
                    self.submit_market_order(order)
                } else {
                    self.event_queue.schedule_reply_for_trader(
                        OrderPlacementDiscarded(order.get_order_id(), ZeroSize), self.current_time, self.trader,
                    )
                }
            }
            CancelLimitOrder(order_id) => { self.cancel_limit_order(order_id) }
            CancelMarketOrder(order_id) => { self.cancel_market_order(order_id) }
        }
    }

    pub(crate) fn handle_trader_wakeup(&mut self) {
        if let Some(freq) = SUBSCRIPTIONS.wakeup {
            let current_time = self.current_time;
            let trader_reactions = self.trader.handle_wakeup(current_time);
            let trader = &mut self.trader;
            self.event_queue.extend(
                trader_reactions.into_iter()
                    .map(
                        |request| Event {
                            timestamp: current_time + Duration::nanoseconds(trader.trader_to_exchange_latency() as i64),
                            body: EventBody::TraderRequest(request),
                        }
                    )
            );
            let next_wakeup_time = current_time + Duration::nanoseconds(freq.get() as i64);
            if (self.is_trading_time)(next_wakeup_time) {
                self.event_queue.push(
                    Event {
                        timestamp: next_wakeup_time,
                        body: EventBody::TraderWakeUp,
                    }
                )
            }
        } else {
            unreachable!()
        }
    }

    pub(crate)
    fn process_next_event(&mut self, event: Event) {
        self.current_time = event.timestamp;
        if self.exchange_closed {
            if self.is_now_trading_time() {
                if DEBUG {
                    eprintln!("{} :: process_next_event :: CLEANUP", event.timestamp)
                }
                self.cleanup();
                self.set_new_trading_period();
                self.exchange_closed = false;
            }
        } else if !self.is_now_trading_time() {
            self.exchange_closed = true
        }
        if DEBUG {
            eprintln!("{} :: process_next_event :: EVENT :: {:?}", event.timestamp, event.body)
        }
        match event.body {
            EventBody::HistoryEvent(event) => { self.handle_history_event(event) }
            EventBody::TraderRequest(request) => { self.handle_trader_request(request) }
            EventBody::ExchangeReply(reply) => { self.handle_exchange_reply(reply) }
            EventBody::SubscriptionUpdate(update) => { self.handle_subscription_update(update) }
            EventBody::SubscriptionSchedule(subscription_type) => { self.handle_subscription_schedule(subscription_type) }
            EventBody::TraderWakeUp => { self.handle_trader_wakeup() }
        }
    }
}