#![feature(const_option, const_trait_impl, const_mut_refs, nonzero_ops)]

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
use std::fs::read_to_string;

use exchange_backtester::prelude::*;

struct CustomTrader<'a> {
    price_step: f64,
    file_to_write: &'a mut BufWriter<File>,
}

impl HandleSubscriptionUpdates for CustomTrader<'_> {
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

impl const Trader for CustomTrader<'_> {
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
        file_to_write: &mut buffer,
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
        .run_trades();

    drop(buffer);
    let file_content = read_to_string(path.join("output.csv")).unwrap();
    assert_eq!(
        file_content,
        "Timestamp,MidPrice\n\
        2021-06-01 08:00:00.039,73.32250\n\
        2021-06-01 09:00:00.039,73.31750\n\
        2021-06-01 10:00:00.039,73.18250\n\
        2021-06-01 11:00:00.039,73.27000\n\
        2021-06-01 12:00:00.039,73.33625\n\
        2021-06-01 13:00:00.039,73.48625\n\
        2021-06-01 14:00:00.039,73.44875\n\
        2021-06-01 15:00:00.039,73.54250\n\
        2021-06-01 16:00:00.039,73.59750\n\
        2021-06-01 17:00:00.039,73.45500\n\
        2021-06-01 18:00:00.039,73.48875\n\
        2021-06-01 19:00:00.039,73.45250\n\
        2021-06-01 20:00:00.039,73.53500\n\
        2021-06-01 21:00:00.039,73.55375\n\
        2021-06-01 22:00:00.039,73.49000\n\
        2021-06-01 23:00:00.039,73.50500\n"
    )
}