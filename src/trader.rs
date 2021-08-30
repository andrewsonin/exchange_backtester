use crate::message::{ExchangeReply, TraderRequest};
use crate::trader::subscriptions::HandleSubscriptionUpdates;
use crate::types::{DateTime, StdRng};

pub mod examples;
pub mod subscriptions;

pub trait Trader: HandleSubscriptionUpdates {
    fn exchange_to_trader_latency(rng: &mut StdRng, dt: DateTime) -> u64;
    fn trader_to_exchange_latency(rng: &mut StdRng, dt: DateTime) -> u64;
    fn handle_exchange_reply(&mut self,
                             exchange_dt: DateTime,
                             deliver_dt: DateTime,
                             reply: ExchangeReply) -> Vec<TraderRequest>;
    fn exchange_open(&mut self, dt: DateTime);
    fn exchange_closed(&mut self, dt: DateTime);
}