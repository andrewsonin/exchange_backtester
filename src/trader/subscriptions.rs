use std::collections::BTreeMap;

use crate::message::TraderRequest;
use crate::types::{NonZeroU64, NonZeroUsize, Price, Size, Timestamp};

#[derive(Eq, PartialEq)]
pub struct SubscriptionConfig {
    pub(crate) ob_depth_and_interval_ns: Option<(usize, NonZeroU64)>,
    pub(crate) trade_info_interval_ns: Option<NonZeroU64>,
    pub(crate) wakeup: Option<NonZeroU64>,
}

pub trait HandleSubscriptionUpdates {
    fn handle_order_book_snapshot(&mut self,
                                  timestamp: Timestamp,
                                  ob_snapshot: OrderBookSnapshot) -> Vec<TraderRequest>;
    fn handle_trade_info_update(&mut self,
                                timestamp: Timestamp,
                                trade_info: Option<TradeInfo>) -> Vec<TraderRequest>;
    fn handle_wakeup(&mut self, timestamp: Timestamp) -> Vec<TraderRequest>;
}

impl SubscriptionConfig {
    pub const fn new() -> Self {
        SubscriptionConfig {
            ob_depth_and_interval_ns: None,
            trade_info_interval_ns: None,
            wakeup: None,
        }
    }
    pub const fn ob_level_subscription_depth(mut self, interval_ns: NonZeroU64, depth: NonZeroUsize) -> Self {
        self.ob_depth_and_interval_ns = Some((depth.get(), interval_ns));
        self
    }
    pub const fn ob_level_subscription_full(mut self, interval_ns: NonZeroU64) -> Self {
        self.ob_depth_and_interval_ns = Some((usize::MAX, interval_ns));
        self
    }
    pub const fn trade_info_subscription(mut self, interval_ns: NonZeroU64) -> Self {
        self.trade_info_interval_ns = Some(interval_ns);
        self
    }
    pub const fn with_periodic_wakeup(mut self, interval_ns: NonZeroU64) -> Self {
        self.wakeup = Some(interval_ns);
        self
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderBookSnapshot {
    pub bids: Vec<(Price, Size)>,
    pub asks: Vec<(Price, Size)>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TradeInfo {
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,

    pub buy_volume: Size,
    pub sell_volume: Size,

    pub price_to_volume_sorted: PriceToVolumeSorted,
}

pub type PriceToVolumeSorted = BTreeMap<Price, TradeVolumesBin>;

#[derive(Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct TradeVolumesBin {
    pub buy_aggressors: Size,
    pub sell_aggressors: Size,
}