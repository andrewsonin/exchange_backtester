use crate::exchange::trades::history::OrderBookDiff;
use crate::message::TraderRequest;
use crate::types::{DateTime, Price, Size};

pub trait HandleSubscriptionUpdates {
    fn handle_order_book_snapshot(&mut self,
                                  exchange_dt: DateTime,
                                  delivery_dt: DateTime,
                                  ob_snapshot: OrderBookSnapshot) -> Vec<TraderRequest>;
    fn handle_trade_info_update(&mut self,
                                exchange_dt: DateTime,
                                delivery_dt: DateTime,
                                trade_info: Vec<OrderBookDiff>) -> Vec<TraderRequest>;
    fn handle_wakeup(&mut self, dt: DateTime) -> Vec<TraderRequest>;
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct OrderBookSnapshot {
    pub bids: Vec<(Price, Size)>,
    pub asks: Vec<(Price, Size)>,
}