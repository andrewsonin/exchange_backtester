use crate::message::{ExchangeReply, TraderRequest};
use crate::trader::Trader;
use crate::types::{NonZeroU64, Timestamp};

const ONE_BILLION: NonZeroU64 = NonZeroU64::new(1_000_000_000).unwrap();

pub struct VoidTrader;

impl const Trader for VoidTrader {
    fn get_latency(&self) -> u64 { 0 }
    fn get_wakeup_frequency_ns(&self) -> NonZeroU64 { ONE_BILLION }
    fn handle_exchange_reply(&mut self, _: ExchangeReply) {}
    fn set_new_trading_period(&mut self) {}
    fn wakeup(&mut self, _: Timestamp) -> Vec<TraderRequest> { vec![] }
}