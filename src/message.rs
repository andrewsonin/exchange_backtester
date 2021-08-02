use crate::order::{LimitOrder, MarketOrder};
use crate::types::{OrderID, OrderSize, Price};

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum TraderRequest {
    CancelLimitOrder(OrderID),
    CancelMarketOrder(OrderID),
    PlaceLimitOrder(LimitOrder),
    PlaceMarketOrder(MarketOrder),
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum ExchangeReply {
    OrderAccepted(OrderID),
    OrderPlacementDiscarded(OrderID, DiscardingReason),
    OrderPartiallyExecuted(OrderID, OrderSize, Price),
    OrderExecuted(OrderID, OrderSize, Price),
    OrderCancelled(OrderID, CancellationReason),
    CannotCancelOrder(OrderID, InabilityToCancelReason),
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum DiscardingReason {
    OrderWithSuchIDAlreadySubmitted,
    ZeroSize,
    ExchangeClosed,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum CancellationReason {
    TraderRequested,
    ExchangeClosed,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum InabilityToCancelReason {
    OrderHasNotBeenSubmitted,
    OrderAlreadyExecuted,
    ExchangeClosed,
}