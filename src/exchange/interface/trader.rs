use std::collections::hash_map::Entry;

use crate::exchange::{Exchange, interface::private::AggressiveOrderType, types::{Event, EventBody}};
use crate::history::{parser::EventProcessor, types::OrderOrigin};
use crate::lags::interface::NanoSecondGenerator;
use crate::message::{CancellationReason, DiscardingReason, ExchangeReply, InabilityToCancelReason, SubscriptionUpdate};
use crate::order::{LimitOrder, MarketOrder, Order};
use crate::trader::Trader;
use crate::types::{DateTime, Direction, Duration, OrderID};

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
    pub(crate) fn handle_subscription_update(&mut self, update: SubscriptionUpdate, exchange_ts: DateTime) {
        let deliver_ts = self.current_dt;
        let trader_reactions = match update {
            SubscriptionUpdate::OrderBook(ob_snapshot) => {
                self.trader.handle_order_book_snapshot(exchange_ts, deliver_ts, ob_snapshot)
            }
            SubscriptionUpdate::TradeInfo(trade_info) => {
                self.trader.handle_trade_info_update(exchange_ts, deliver_ts, trade_info)
            }
            SubscriptionUpdate::ExchangeOpen => {
                self.trader.exchange_open(exchange_ts, deliver_ts);
                return;
            }
            SubscriptionUpdate::ExchangeClosed => {
                self.trader.exchange_closed(exchange_ts, deliver_ts);
                return;
            }
        };
        let rng = &mut self.rng;
        self.event_queue.extend(
            trader_reactions
                .into_iter()
                .map(
                    |request| Event {
                        datetime: deliver_ts + Duration::nanoseconds(T::trader_to_exchange_latency(rng, deliver_ts) as i64),
                        body: EventBody::TraderRequest(request),
                    }
                )
        )
    }

    pub(crate) fn submit_limit_order(&mut self, order: LimitOrder) {
        let order_id = order.get_order_id();
        let reply = if !self.is_now_trading_time() {
            ExchangeReply::OrderPlacementDiscarded(
                order_id,
                DiscardingReason::ExchangeClosed,
            )
        } else if self.trader_submitted_orders.contains(&order_id) {
            ExchangeReply::OrderPlacementDiscarded(
                order_id,
                DiscardingReason::OrderWithSuchIDAlreadySubmitted,
            )
        } else {
            self.insert_limit_order::<LimitOrder, { OrderOrigin::Trader }>(order);
            self.trader_submitted_orders.insert(order_id);
            ExchangeReply::OrderAccepted(order_id)
        };
        self.event_queue.schedule_reply_for_trader::<T>(reply, self.current_dt, &mut self.rng);
    }

    pub(crate) fn submit_market_order(&mut self, order: MarketOrder) {
        let order_id = order.get_order_id();
        let reply = if !self.is_now_trading_time() {
            ExchangeReply::OrderPlacementDiscarded(
                order_id,
                DiscardingReason::ExchangeClosed,
            )
        } else if self.trader_submitted_orders.contains(&order_id) {
            ExchangeReply::OrderPlacementDiscarded(
                order_id,
                DiscardingReason::OrderWithSuchIDAlreadySubmitted,
            )
        } else {
            self.insert_aggressive_order::<MarketOrder, { AggressiveOrderType::TraderMarketOrder }>(order);
            self.trader_submitted_orders.insert(order_id);
            ExchangeReply::OrderAccepted(order_id)
        };
        self.event_queue.schedule_reply_for_trader::<T>(reply, self.current_dt, &mut self.rng);
    }

    pub(crate) fn cancel_limit_order(&mut self, order_id: OrderID) {
        let reply = if !self.is_now_trading_time() {
            ExchangeReply::CannotCancelOrder(
                order_id,
                InabilityToCancelReason::ExchangeClosed,
            )
        } else if !self.trader_submitted_orders.contains(&order_id) {
            ExchangeReply::CannotCancelOrder(
                order_id,
                InabilityToCancelReason::OrderHasNotBeenSubmitted,
            )
        } else {
            match self.trader_pending_limit_orders.entry(order_id) {
                Entry::Occupied(value) => {
                    let (price, direction) = value.get();
                    let mut side_cursor = match direction {
                        Direction::Buy => { self.bids.cursor_front_mut() }
                        Direction::Sell => { self.asks.cursor_front_mut() }
                    };
                    while let Some(level) = side_cursor.current() {
                        if level.price == *price {
                            let level_size = level.queue.len();
                            let mut level_cursor = level.queue.cursor_front_mut();
                            while let Some(order) = level_cursor.current() {
                                if order.from == OrderOrigin::Trader && order.order_id == order_id {
                                    level_cursor.remove_current();
                                    if level_size == 1 {
                                        side_cursor.remove_current();
                                    }
                                    break;
                                }
                                level_cursor.move_next()
                            }
                            break;
                        }
                        side_cursor.move_next()
                    }
                    ExchangeReply::OrderCancelled(order_id, CancellationReason::TraderRequested)
                }
                _ => {
                    ExchangeReply::CannotCancelOrder(
                        order_id,
                        InabilityToCancelReason::OrderAlreadyExecuted,
                    )
                }
            }
        };
        self.event_queue.schedule_reply_for_trader::<T>(reply, self.current_dt, &mut self.rng);
    }

    pub(crate) fn cancel_market_order(&mut self, order_id: OrderID) {
        let reply = if !self.is_now_trading_time() {
            ExchangeReply::CannotCancelOrder(
                order_id,
                InabilityToCancelReason::ExchangeClosed,
            )
        } else if !self.trader_submitted_orders.contains(&order_id) {
            ExchangeReply::CannotCancelOrder(
                order_id,
                InabilityToCancelReason::OrderHasNotBeenSubmitted,
            )
        } else {
            let mut cursor = self.trader_pending_market_orders.cursor_front_mut();
            (
                || {
                    while let Some(order) = cursor.current() {
                        if order.get_order_id() == order_id {
                            cursor.remove_current();
                            return ExchangeReply::OrderCancelled(order_id, CancellationReason::TraderRequested);
                        }
                        cursor.move_next()
                    }
                    return ExchangeReply::CannotCancelOrder(
                        order_id,
                        InabilityToCancelReason::OrderAlreadyExecuted,
                    );
                }
            )()
        };
        self.event_queue.schedule_reply_for_trader::<T>(reply, self.current_dt, &mut self.rng);
    }
}