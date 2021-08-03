use crate::message::{ExchangeReply, TraderRequest};
use crate::types::{NonZeroU64, Timestamp};

pub mod examples;

pub trait Trader {
    fn get_latency(&self) -> u64;
    fn get_wakeup_frequency_ns(&self) -> NonZeroU64;
    fn handle_exchange_reply(&mut self, reply: ExchangeReply);
    fn set_new_trading_period(&mut self);
    fn wakeup(&mut self, timestamp: Timestamp) -> Vec<TraderRequest>;
}