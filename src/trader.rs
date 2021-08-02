use std::num::NonZeroU64;

use chrono::NaiveDateTime as Timestamp;

use crate::message::{ExchangeReply, TraderRequest};

pub trait Trader {
    fn get_latency(&self) -> u64;
    fn get_wakeup_frequency(&self) -> NonZeroU64;
    fn handle_exchange_reply(&mut self, reply: ExchangeReply);
    fn set_new_trading_period(&mut self);
    fn wakeup(&mut self, timestamp: Timestamp) -> Vec<TraderRequest>;
}