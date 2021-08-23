use crate::exchange::trades::history::OrderBookDiff;
use crate::message::{ExchangeReply, TraderRequest};
use crate::trader::{subscriptions::{HandleSubscriptionUpdates, OrderBookSnapshot}, Trader};
use crate::types::{DateTime, StdRng};

pub struct VoidTrader;

impl HandleSubscriptionUpdates for VoidTrader {
    fn handle_order_book_snapshot(&mut self, _: DateTime, _: DateTime, _: OrderBookSnapshot) -> Vec<TraderRequest> {
        vec![]
    }
    fn handle_trade_info_update(&mut self, _: DateTime, _: DateTime, _: Vec<OrderBookDiff>) -> Vec<TraderRequest> {
        vec![]
    }
    fn handle_wakeup(&mut self, _: DateTime) -> Vec<TraderRequest> {
        vec![]
    }
}

impl const Trader for VoidTrader {
    fn exchange_to_trader_latency(_: &mut StdRng, _: DateTime) -> u64 { 0 }
    fn trader_to_exchange_latency(_: &mut StdRng, _: DateTime) -> u64 { 0 }
    fn handle_exchange_reply(&mut self, _: DateTime, _: DateTime, _: ExchangeReply) -> Vec<TraderRequest> { vec![] }
    fn set_new_trading_period(&mut self, _: DateTime) {}
}