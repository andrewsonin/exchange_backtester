use crate::exchange::trades::history::OrderBookDiff;
use crate::order::{LimitOrder, MarketOrder};
use crate::trader::subscriptions::OrderBookSnapshot;
use crate::types::{OrderID, Price, Size};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TraderRequest {
    CancelLimitOrder(OrderID),
    CancelMarketOrder(OrderID),
    PlaceLimitOrder(LimitOrder),
    PlaceMarketOrder(MarketOrder),
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeReply {
    OrderAccepted(OrderID),
    OrderPlacementDiscarded(OrderID, DiscardingReason),
    OrderPartiallyExecuted(OrderID, Size, Price),
    OrderExecuted(OrderID, Size, Price),
    OrderCancelled(OrderID, CancellationReason),
    CannotCancelOrder(OrderID, InabilityToCancelReason),
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DiscardingReason {
    OrderWithSuchIDAlreadySubmitted,
    ZeroSize,
    ExchangeClosed,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CancellationReason {
    TraderRequested,
    ExchangeClosed,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToCancelReason {
    OrderHasNotBeenSubmitted,
    OrderAlreadyExecuted,
    ExchangeClosed,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum SubscriptionUpdate {
    ExchangeOpen,
    OrderBook(OrderBookSnapshot),
    TradeInfo(Vec<OrderBookDiff>),
    ExchangeClosed,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum SubscriptionSchedule {
    OrderBook,
    TradeInfo,
}