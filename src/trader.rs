use crate::message::{ExchangeReply, TraderRequest};
use crate::trader::subscriptions::HandleSubscriptionUpdates;

pub mod examples;
pub mod subscriptions;

pub trait Trader: HandleSubscriptionUpdates {
    fn exchange_to_trader_latency(&self) -> u64;
    fn trader_to_exchange_latency(&self) -> u64;
    fn handle_exchange_reply(&mut self, reply: ExchangeReply) -> Vec<TraderRequest>;
    fn set_new_trading_period(&mut self);
}