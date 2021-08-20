#![feature(const_option, const_trait_impl, const_mut_refs, nonzero_ops)]

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use exchange_backtester::prelude::*;

struct CustomTrader {
    price_step: f64,
    file_to_write: BufWriter<File>,
}

impl HandleSubscriptionUpdates for CustomTrader {
    fn handle_order_book_snapshot(&mut self,
                                  exchange_ts: Timestamp,
                                  _: Timestamp,
                                  ob_snapshot: OrderBookSnapshot) -> Vec<TraderRequest>
    {
        let mid_price = match (ob_snapshot.bids.first(), ob_snapshot.asks.first())
        {
            (Some((bid_price, _)), Some((ask_price, _))) => {
                (bid_price.to_f64(self.price_step) + ask_price.to_f64(self.price_step)) * 0.5
            }
            (Some((bid_price, _)), _) => { bid_price.to_f64(self.price_step) }
            (_, Some((ask_price, _))) => { ask_price.to_f64(self.price_step) }
            _ => { return vec![]; }
        };
        write!(self.file_to_write, "{},{:.5}\n", exchange_ts, mid_price);
        vec![]
    }
    fn handle_trade_info_update(&mut self, _: Timestamp, _: Timestamp, _: Option<TradeInfo>) -> Vec<TraderRequest> {
        vec![]
    }
    fn handle_wakeup(&mut self, _: Timestamp) -> Vec<TraderRequest> {
        vec![]
    }
}

impl const Trader for CustomTrader {
    fn exchange_to_trader_latency(_: &mut StdRng, _: Timestamp) -> u64 { 0 }
    fn trader_to_exchange_latency(_: &mut StdRng, _: Timestamp) -> u64 { 0 }
    fn handle_exchange_reply(&mut self, _: Timestamp, _: Timestamp, _: ExchangeReply) -> Vec<TraderRequest> {
        vec![]
    }
    fn set_new_trading_period(&mut self, _: Timestamp) {}
}

fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("integration")
        .join("one_day");
    let input = StaticInput::new()
        .with_prl_files(path.join("PRL.txt").to_str().unwrap())
        .with_trd_files(path.join("TRD.txt").to_str().unwrap())
        .with_datetime_format("%d/%m/%Y %H:%M:%S%.f");
    let history_parser = HistoryParser::new(&input);

    let mut buffer = BufWriter::new(File::create(path.join("output.csv")).unwrap());
    write!(buffer, "Timestamp,MidPrice\n");

    let mut trader = CustomTrader {
        price_step: input.get_price_step(),
        file_to_write: buffer,
    };

    let is_trading_time = |timestamp: Timestamp| {
        match timestamp.hour() {
            7..=22 => { true }
            23 => { timestamp.minute() < 50 }
            _ => { false }
        }
    };

    ExchangeBuilder::new::<false>(
        history_parser,
        &mut trader,
        is_trading_time,
    )
        .ob_level_subscription_depth(lags::constant::ONE_HOUR, 1)
        .run_trades()
}