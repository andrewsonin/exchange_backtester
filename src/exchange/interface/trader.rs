use std::collections::hash_map::Entry;

use crate::exchange::Exchange;
use crate::exchange::interface::private::AggressiveOrderType;
use crate::exchange::types::{Event, EventBody};
use crate::history::types::OrderOrigin;
use crate::input::InputInterface;
use crate::message::{CancellationReason, DiscardingReason, ExchangeReply, InabilityToCancelReason, SubscriptionUpdate};
use crate::order::{LimitOrder, MarketOrder, Order};
use crate::trader::subscriptions::SubscriptionConfig;
use crate::trader::Trader;
use crate::types::{Duration, OrderDirection, OrderID, Timestamp};

impl<T, TTC, PInfo, const DEBUG: bool, const SUBSCRIPTIONS: SubscriptionConfig>
Exchange<'_, T, TTC, PInfo, DEBUG, SUBSCRIPTIONS>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          PInfo: InputInterface
{
    pub(crate) fn handle_subscription_update(&mut self, update: SubscriptionUpdate) {
        let current_time = self.current_time;

        let trader_reactions = match update {
            SubscriptionUpdate::OrderBook(ob_snapshot) => {
                self.trader.handle_order_book_snapshot(current_time, ob_snapshot)
            }
            SubscriptionUpdate::TradeInfo(trade_info) => {
                self.trader.handle_trade_info_update(current_time, trade_info)
            }
        };
        let trader = &self.trader;
        self.event_queue.extend(
            trader_reactions
                .into_iter()
                .map(
                    |request| Event {
                        timestamp: current_time + Duration::nanoseconds(trader.trader_to_exchange_latency() as i64),
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
            self.trader_submitted_orders.insert(order.get_order_id());
            self.insert_limit_order::<LimitOrder, { OrderOrigin::Trader }>(order);
            ExchangeReply::OrderAccepted(order_id)
        };
        self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
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
            self.trader_submitted_orders.insert(order.get_order_id());
            self.insert_aggressive_order::<{ AggressiveOrderType::MarketOrder }>(order);
            ExchangeReply::OrderAccepted(order_id)
        };
        self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
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
                        OrderDirection::Buy => { self.bids.cursor_front_mut() }
                        OrderDirection::Sell => { self.asks.cursor_front_mut() }
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
        self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
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
        self.event_queue.schedule_reply_for_trader(reply, self.current_time, self.trader);
    }
}