use std::collections::BTreeMap;

use crate::message::TraderRequest;
use crate::types::{Price, Size, Timestamp};

pub trait HandleSubscriptionUpdates {
    fn handle_order_book_snapshot(&mut self,
                                  exchange_ts: Timestamp,
                                  deliver_ts: Timestamp,
                                  ob_snapshot: OrderBookSnapshot) -> Vec<TraderRequest>;
    fn handle_trade_info_update(&mut self,
                                exchange_ts: Timestamp,
                                deliver_ts: Timestamp,
                                trade_info: Option<TradeInfo>) -> Vec<TraderRequest>;
    fn handle_wakeup(&mut self, ts: Timestamp) -> Vec<TraderRequest>;
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