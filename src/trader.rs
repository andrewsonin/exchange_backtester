use crate::message::{ExchangeReply, TraderRequest};
use crate::trader::subscriptions::HandleSubscriptionUpdates;
use crate::types::{StdRng, Timestamp};

pub mod examples;
pub mod subscriptions;

pub trait Trader: HandleSubscriptionUpdates {
    fn exchange_to_trader_latency(&mut self, rng: &mut StdRng, ts: Timestamp) -> u64;
    fn trader_to_exchange_latency(&mut self, rng: &mut StdRng, ts: Timestamp) -> u64;
    fn handle_exchange_reply(&mut self,
                             exchange_ts: Timestamp,
                             deliver_ts: Timestamp,
                             reply: ExchangeReply) -> Vec<TraderRequest>;
    fn set_new_trading_period(&mut self);
}