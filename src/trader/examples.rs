use crate::message::{ExchangeReply, TraderRequest};
use crate::trader::{subscriptions::{HandleSubscriptionUpdates, OrderBookSnapshot, TradeInfo}, Trader};
use crate::types::Timestamp;

pub struct VoidTrader;

impl HandleSubscriptionUpdates for VoidTrader {
    fn handle_order_book_snapshot(&mut self, _: Timestamp, _: OrderBookSnapshot) -> Vec<TraderRequest> {
        vec![]
    }
    fn handle_trade_info_update(&mut self, _: Timestamp, _: Option<TradeInfo>) -> Vec<TraderRequest> {
        vec![]
    }
    fn handle_wakeup(&mut self, _: Timestamp) -> Vec<TraderRequest> {
        vec![]
    }
}

impl const Trader for VoidTrader {
    fn exchange_to_trader_latency(&self) -> u64 { 0 }
    fn trader_to_exchange_latency(&self) -> u64 { 0 }
    fn handle_exchange_reply(&mut self, _: ExchangeReply) -> Vec<TraderRequest> { vec![] }
    fn set_new_trading_period(&mut self) {}
}