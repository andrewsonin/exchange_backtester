use std::collections::BTreeMap;

use crate::exchange::trades::history::ExecutedTrade;
use crate::message::TraderRequest;
use crate::types::{DateTime, Price, Size};

pub trait HandleSubscriptionUpdates {
    fn handle_order_book_snapshot(&mut self,
                                  exchange_dt: DateTime,
                                  deliver_dt: DateTime,
                                  ob_snapshot: OrderBookSnapshot) -> Vec<TraderRequest>;
    fn handle_trade_info_update(&mut self,
                                exchange_dt: DateTime,
                                deliver_dt: DateTime,
                                trade_info: Vec<ExecutedTrade>) -> Vec<TraderRequest>;
    fn handle_wakeup(&mut self, dt: DateTime) -> Vec<TraderRequest>;
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