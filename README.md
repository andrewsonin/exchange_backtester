# Exchange backtester

Framework that allows you to replay the exchange trading history for a single ticker and test your trading strategies.

Written on the pure Rust, it uses some experimental language features such as constant generics. This is necessary in
order to instantiate (or not instantiate) machine code as efficiently as possible and calculate everything that is
possible at compile time. However, as of now (August 10, 2021), this requires compiling with the `nightly` version
of `rustc`.

## How to use (default)

1. In order to switch to the `nighly` version, type the following:

    ```sh
    rustup default nightly
    ```
2. Then create an application project that will use `exchange_backtester` as a library:

   ```sh
   cargo new <PROJECT_NAME>
   ```
   and add `exchange_backtester` to the project's `Cargo.toml` as a dependency like here:
   ```toml
   [package]
   name = "<PROJECT_NAME>"
   version = "0.1.0"
   edition = "2018"
   
   # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
   
   [dependencies]
   exchange_backtester = { git = "https://github.com/andrewsonin/exchange_backtester" }
   ```
3. Implement your custom trading agent. The following is the example of `main.rs` that implements a command line app
   that reads the trading history and prints the middle price of the spread into the standard output every trading hour
   or prints the message into the standard error in case if the order book is empty.

   ```rust
   #![feature(const_mut_refs, const_trait_impl)]
   
   use exchange_backtester::prelude::*;
   
   struct CustomTrader {
       price_step: f64,
   }
   
   impl HandleSubscriptionUpdates for CustomTrader
   {
       fn handle_order_book_snapshot(&mut self,
                                     exchange_ts: Timestamp,
                                     deliver_ts: Timestamp,
                                     ob_snapshot: OrderBookSnapshot) -> Vec<TraderRequest>
       {
           let mid_price = match (ob_snapshot.bids.first(), ob_snapshot.asks.first())
           {
               (Some((bid_price, _)), Some((ask_price, _))) => {
                   (bid_price.to_f64(self.price_step) + ask_price.to_f64(self.price_step)) * 0.5
               }
               (Some((bid_price, _)), _) => { bid_price.to_f64(self.price_step) }
               (_, Some((ask_price, _))) => { ask_price.to_f64(self.price_step) }
               _ => {
                   eprintln!("Timestamp: {}. Order book is empty", exchange_ts);
                   return vec![];
               }
           };
           println!("{},{}", exchange_ts, mid_price);
           vec![]
       }
       fn handle_trade_info_update(&mut self,
                                   exchange_ts: Timestamp,
                                   deliver_ts: Timestamp,
                                   trade_info: Option<TradeInfo>) -> Vec<TraderRequest> { vec![] }
       // Called when the time comes for the scheduled periodic trader wakeup
       fn handle_wakeup(&mut self, ts: Timestamp) -> Vec<TraderRequest> { vec![] }
   }
   
   impl const Trader for CustomTrader {
       fn exchange_to_trader_latency(rng: &mut StdRng, ts: Timestamp) -> u64 { 0 }
       fn trader_to_exchange_latency(rng: &mut StdRng, ts: Timestamp) -> u64 { 0 }
       fn handle_exchange_reply(&mut self,
                                exchange_ts: Timestamp,
                                deliver_ts: Timestamp,
                                reply: ExchangeReply) -> Vec<TraderRequest> { vec![] }
       // Called when the new trading day begins
       fn set_new_trading_period(&mut self, ts: Timestamp) {}
   }
   
   fn main() {
       let input = ArgumentParser::parse();
       let history_parser = HistoryParser::new(&input);
   
       let mut trader = CustomTrader { price_step: input.get_price_step() };
   
       let is_trading_time = |timestamp: Timestamp| {
           match timestamp.hour() {
               7..=22 => { true }
               23 => { timestamp.minute() < 50 }
               _ => { false }
           }
       };
   
       let mut exchange = ExchangeBuilder::new::<false>(
           history_parser,
           &mut trader,
           is_trading_time,
       )
           .ob_level_subscription_depth(lags::constant::ONE_HOUR, 3)
           .trade_info_subscription(lags::constant::ONE_SECOND)
           .with_periodic_wakeup(lags::constant::ONE_MINUTE);
   
       exchange.run_trades()
   }
   ```

If you compile the above example and print `<resulting_exe_name> --help`, you will get the following:

```sh
USAGE:
    <resulting_exe_name> [OPTIONS] --prl <prl-files> --trd <trd-files>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --csv-sep <csv-sep>                          CSV-file separator [default: ,]
    -d, --datetime-format <datetime-format>
            Sets the datetime format to parse timestamp columns [default: %Y-%m-%d %H:%M:%S%.f]

        --bs-flag-colname <order-bs-flag-colname>
            Sets the name of the order buy-sell flag columns in the input csv files [default:
            BUY_SELL_FLAG]

        --id-colname <order-id-colname>
            Sets the name of the order ID columns in the input csv files [default: ORDER_ID]

        --price-colname <order-price-colname>
            Sets the name of the order price columns in the input csv files [default: PRICE]

        --size-colname <order-size-colname>
            Sets the name of the order size columns in the input csv files [default: SIZE]

        --ts-colname <order-timestamp-colname>
            Sets the name of the timestamp columns in the input csv files [default: Timestamp]

        --price-step <price-step>                    Price step [default: 0.0025]
    -p, --prl <prl-files>
            Sets the file each line of which should contain absolute paths to the PRL csv-files to
            use

    -t, --trd <trd-files>
            Sets the file each line of which should contain absolute paths to the TRD csv-files to
            use
```

Here, `--prl` specifies the path to the config file containing the ordered list of paths to the `PRL`-files to
use; `--trd` specifies the same but for `TRD`-files. Parameter `--price-step` specifies the price step of the ticker,
i.e. the minimal distance between two limit orders placed at different prices. The descriptions of the other parameters
are self-explanatory.

Note that it is not required that each `PRL` file should strictly correspond to a separate `TRD`-file. The only
requirement is that the lists of paths to the `PRL` and `TRD` files and the entries in them should be sorted in
ascending order by time.

## How it works (default)

Default version of the backtester simultaneously reads two types of the exchange history backups — `TRD` and `PRL` —
that should be stored in CSV-format and sorted in ascending order by time. The first one is a representation of all
trades happened, the second one is a representation of all changes in the order book.

`PRL` should have the following columns (column names can be specified differently than in the example below. This can
be achieved using command line parameters of the `ArgumentParser`):

1. `Timestamp` — `chrono::NaiveDateTime`, represents the actual time of the event of a change in the state of the order
   book.
2. `ORDER_ID` — `u64`, a number assigned to the corresponding limit order, unique within one trading session.
3. `PRICE` — `f64`, represents the limit price of the order.
4. `SIZE` — `u64`, size of the order in lots.
5. `BUY_SELL_FLAG` — `<literal>`, represents the direction of the order. Buy orders should have one of the following
   values: `0/B/b/false/False`. Sell orders should have one of the following ones: `1/S/s/true/True`.

`TRD` should have the same columns except `PRICE`. But the meaning is slightly different:

1. `Timestamp` represents the actual time of the trade happened.
2. `ORDER_ID` represents the order ID of the reference limit order with which the trade happened.
3. `SIZE` represents the size of the trade.
4. `BUY_SELL_FLAG` represents the direction of the aggressor whose action led to the trade.

Different entries in the `PRL` file can have the same ID in two cases: if they correspond to different trading sessions
or if they correspond to the same limit order within one session. In the latter case their meaning can be different. The
first entry should reflect the event of creating a limit order. The next entries should reflect the remaining price of
the limit order after trading. If the value in the `SIZE` column of the entry equals to zero, the limit order considered
fully executed or cancelled (by the market maker or by the exchange at the end of the trading period).

Let's take a look at the following examples.

### PRL-file

|               Timestamp | SIZE | PRICE | ORDER_ID | BUY_SELL_FLAG |
| ----------------------- | ---- | ----- | -------- | ------------- |
| 2020-04-02 13:01:11.100 |  302 | 12.08 |        1 |             B |
| 2020-04-02 13:01:13.402 |  203 | 12.08 |        1 |             B |
| 2020-04-02 13:02:44.002 |   25 | 13.19 |        2 |             S |
| 2020-04-02 13:11:22.242 |   55 | 12.08 |        1 |             B |
| 2020-04-02 14:45:01.948 |   22 | 13.19 |        2 |             S |
| 2020-04-02 14:56:32.002 |   10 | 12.08 |        1 |             B |
| 2020-04-02 17:00:34.123 |    0 | 12.08 |        1 |             B |
| 2020-04-02 18:12:58.248 |    0 | 13.19 |        2 |             S |

### TRD-file

|               Timestamp | SIZE | ORDER_ID | BUY_SELL_FLAG |
| ----------------------- | ---- | -------- | ------------- |
| 2020-04-02 13:01:13.402 |   99 |        1 |             S |
| 2020-04-02 13:11:22.242 |  148 |        1 |             S |
| 2020-04-02 14:45:01.948 |    3 |        2 |             B |
| 2020-04-02 14:56:32.002 |   45 |        1 |             S |
| 2020-04-02 18:12:58.248 |   22 |        2 |             B |

What do we see here? We see that the order with `ORDER_ID = 1` was placed at `2020-04-02 13:01:11.100` and then executed
at `2020-04-02 13:01:13.402`, `2020-04-02 13:11:22.242` and `2020-04-02 14:56:32.002`, but at `2020-04-02 17:00:34.123`
it was cancelled. We know this because at this time this order does not have corresponding entry in the `TRD`-file.

Vice versa, the order with `ORDER_ID = 1` was placed at `2020-04-02 13:02:44.002` and then was fully executed by two
subsequent trades at `2020-04-02 14:45:01.948` and `2020-04-02 18:12:58.248`.

## Customizing the backtester

### 1. Ticker history input

In addition to using the default solution, this library allows the programmer to connect other sources of trading
histories — not just CSV files on disk. This is achieved by creating a custom structure that must implement
the `EventProcessor` trait:

```rust
pub trait EventProcessor {
    fn yield_next_event(&mut self) -> Option<HistoryEvent>;
}
```

`HistoryEvent` is an `enum` with the following definition:

```rust
pub enum HistoryEventBody {
    TRD(Size, Direction),
    PRL(Size, Direction, Price, OrderID),
}

pub struct HistoryEvent
{
    pub timestamp: Timestamp,
    pub event: HistoryEventBody,
}
```

- `OrderID`, `Price` and `Size` are just wrapper types around `u64`.
- `Direction` is the `enum` type with two possible values: `Buy` and `Sell`.
- `Price` does not wrap `f64`, but `u64` deliberately — in order to get rid of potential problems associated with the
  loss of precision in floating point calculations. Actual price `f64` value is just a `Price` inner `u64` multiplied by
  the `--price-step` in the upper example.
- `Timestamp` here is just a naming alias to the `chrono::NaiveDateTime`.

#### Example

You can implement your own trading history interface using the following backbone:

```rust
#![feature(const_trait_impl, const_mut_refs)]

use std::collections::VecDeque;

use exchange_backtester::input::default::DATETIME_FORMAT;
use exchange_backtester::prelude::*;

const PRICE_STEP: f64 = 0.0025;

struct CustomTrader;

impl HandleSubscriptionUpdates for CustomTrader {
   fn handle_order_book_snapshot(&mut self,
                                 exchange_ts: Timestamp,
                                 deliver_ts: Timestamp,
                                 ob_snapshot: OrderBookSnapshot) -> Vec<TraderRequest>
   {
      let mid_price = match (ob_snapshot.bids.first(), ob_snapshot.asks.first())
      {
         (Some((bid_price, _)), Some((ask_price, _))) => {
            (bid_price.to_f64(PRICE_STEP) + ask_price.to_f64(PRICE_STEP)) * 0.5
         }
         (Some((bid_price, _)), _) => { bid_price.to_f64(PRICE_STEP) }
         (_, Some((ask_price, _))) => { ask_price.to_f64(PRICE_STEP) }
         _ => { return vec![]; }
      };
      println!("{},{}", exchange_ts, mid_price);
      vec![]
   }
   fn handle_trade_info_update(&mut self,
                               exchange_ts: Timestamp,
                               deliver_ts: Timestamp,
                               trade_info: Option<TradeInfo>) -> Vec<TraderRequest> { vec![] }
   // Called when the time comes for the scheduled periodic trader wakeup
   fn handle_wakeup(&mut self, ts: Timestamp) -> Vec<TraderRequest> { vec![] }
}

impl const Trader for CustomTrader {
   fn exchange_to_trader_latency(rng: &mut StdRng, ts: Timestamp) -> u64 { 0 }
   fn trader_to_exchange_latency(rng: &mut StdRng, ts: Timestamp) -> u64 { 0 }
   fn handle_exchange_reply(&mut self,
                            exchange_ts: Timestamp,
                            deliver_ts: Timestamp,
                            reply: ExchangeReply) -> Vec<TraderRequest> { vec![] }
   // Called when the new trading day begins
   fn set_new_trading_period(&mut self, ts: Timestamp) {}
}

#[derive(Default)]
struct HistoryHolder {
   history: VecDeque<HistoryEvent>,
}

impl HistoryHolder {
   fn add_prl(&mut self, ts: &str, size: u64, dir: Direction, price: f64, order_id: u64) {
      self.history.push_back(HistoryEvent {
         timestamp: Timestamp::parse_from_str(ts, DATETIME_FORMAT).unwrap(),
         event: HistoryEventBody::PRL(Size(size), dir, Price::from_f64(price, PRICE_STEP), OrderID(order_id)),
      })
   }

   fn add_trd(&mut self, ts: &str, size: u64, dir: Direction) {
      self.history.push_back(HistoryEvent {
         timestamp: Timestamp::parse_from_str(ts, DATETIME_FORMAT).unwrap(),
         event: HistoryEventBody::TRD(Size(size), dir),
      })
   }
}

impl EventProcessor for HistoryHolder {
   fn yield_next_event(&mut self) -> Option<HistoryEvent> {
      self.history.pop_front()
   }
}

fn is_trading_time(timestamp: Timestamp) -> bool {
   match timestamp.hour() {
      7..=19 => { true }
      _ => { false }
   }
}

fn main() {
   let mut history = HistoryHolder::default();
   history.add_prl("2020-03-03 12:22:22.31",  3, Direction::Buy,  12.0025, 1);
   history.add_prl("2020-03-03 14:11:26.33", 22, Direction::Sell, 12.0075, 2);
   history.add_trd("2020-03-03 16:11:26.33",  2, Direction::Sell);
   history.add_prl("2020-03-03 16:11:26.33",  1, Direction::Buy,  12.0025, 1);
   history.add_trd("2020-03-03 18:24:00",     1, Direction::Sell);
   history.add_prl("2020-03-03 18:24:00",     0, Direction::Buy,  12.0025, 1);

   let mut trader = CustomTrader;
   let mut exchange = ExchangeBuilder::new::<false>(
      history,
      &mut trader,
      is_trading_time,
   )
           .ob_level_subscription_depth(lags::constant::ONE_HOUR, 1);

   println!("Timestamp,MidPrice");
   exchange.run_trades()
}
```

This code prints a CSV-table into the standard output representing the middle price of the order book at every hour of
the trading period after the start of trades determined by the function `is_trading_time`:

|               Timestamp | MidPrice |
| ----------------------- | ---------|
| 2020-03-03 13:22:22.310 | 12.0025  |
| 2020-03-03 14:22:22.310 | 12.005   |
| 2020-03-03 15:22:22.310 | 12.005   |
| 2020-03-03 16:22:22.310 | 12.005   |
| 2020-03-03 17:22:22.310 | 12.005   |
| 2020-03-03 18:22:22.310 | 12.005   |
| 2020-03-03 19:22:22.310 | 12.0075  |

### 2. Trader Subscription Configuration

The exchange notifies the trader about the events happened in two ways.

1. By sending `ExchangeReply` that contain an information about order placement, discarding and execution. It is
   directly related to the trading agent. Agent may react to this notifications by sending a list of instances
   of `TraderRequest`. This can be achieved via the method `handle_exchange_reply` in the `Trader` trait. Below you can
   see the full list of useful structures involved in such request-reply process:

   ```rust
   pub enum TraderRequest {
      CancelLimitOrder(OrderID),
      CancelMarketOrder(OrderID),
      PlaceLimitOrder(LimitOrder),
      PlaceMarketOrder(MarketOrder),
   }
   
   pub enum ExchangeReply {
      OrderAccepted(OrderID),
      OrderPlacementDiscarded(OrderID, DiscardingReason),
      OrderPartiallyExecuted(OrderID, Size, Price),
      OrderExecuted(OrderID, Size, Price),
      OrderCancelled(OrderID, CancellationReason),
      CannotCancelOrder(OrderID, InabilityToCancelReason),
   }
   
   pub enum DiscardingReason {
      OrderWithSuchIDAlreadySubmitted,
      ZeroSize,
      ExchangeClosed,
   }
   
   pub enum CancellationReason {
      TraderRequested,
      ExchangeClosed,
   }
   
   pub enum InabilityToCancelReason {
      OrderHasNotBeenSubmitted,
      OrderAlreadyExecuted,
      ExchangeClosed,
   }
   ```
2. By sending subscription updates. This information refers to the state of the market as a whole. Trader can subscribe
   to this information using special chained initialization methods of the `Exchange`. Here
   they are:

   ```rust
   fn ob_level_subscription_depth<G: NanoSecondGenerator>(self, ns_gen: G, depth: usize);
   
   fn ob_level_subscription_full<G: NanoSecondGenerator>(self, ns_gen: G);
   
   fn trade_info_subscription<G: NanoSecondGenerator>(self, ns_gen: G);
   
   fn with_periodic_wakeup<G: NanoSecondGenerator>(self, ns_gen: G);
   ```
    - `ob_level_subscription_depth` subscribe the trader to the order book snapshots which depth is limited to
      the `depth` parameter. Information comes at intervals, the duration of which is determined by the `ns_gen` structure.
    - `ob_level_subscription_full` does the same, but the depth is not limited.
    - `trade_info_subscription` subscribe the trader to the candles that contain the information of the executed trades
       after the last `trade_info_subscription` call.
    - `with_periodic_wakeup` just ping the trader and allow him to send the list of instances of `TraderRequest`
      every time interval, the duration of which is determined by the `ns_gen` structure.

Exchange replies, subscription updates and trader requests does not come immediately after sending. The lag in
nanoseconds is set by `exchange_to_trader_latency` (for exchange replies and subscription updates)
and `trader_to_exchange_latency` (for trader requests) methods in the `Trader` trait. Note that they can use the exchange random number generator `rng` and can return
different values each time which makes it possible to simulate a latency noise. Also note that trader wakeups do not suffer
from `exchange_to_trader_latency`.

As you can see, the `ns_gen` structure must implement `NanoSecondGenerator` trait that looks like the following:

```rust
pub trait NanoSecondGenerator {
    fn gen_ns(&mut self, rng: &mut StdRng, ts: Timestamp) -> NonZeroU64;
}
```

`gen_ns` method here receives the exchange random number generator `rng` and the timestamp of the event `ts`.

### 3. ExchangeBuilder

`ExchangeBuilder::new` and `ExchangeBuilder::new_debug` are used to initialize the exchange. They have identical
signatures except that the latter one create an exchange that prints to the standard error some debug messages while
running. These functions have the following signature (consider an example with `ExchangeBuilder::new`):

```rust
fn new<const TRD_UPDATES_OB: bool>(
    event_processor: EP,
    trader: &'a mut T,
    is_trading_time: TradingTimeCriterion,
)
    where EP: EventProcessor,
          T: Trader,
          TradingTimeCriterion: Fn(Timestamp) -> bool
```

As you can see, the first argument should implement the `EventProcessor` processor trait, the second one should
implement the `Trader` trait and the last one should be a function that says whether the given `Timestamp` is a trading
time.

`TRD_UPDATES_OB` template parameter is responsible for the behavior of the order book after receiving `TRD` events. If
it is set to `false` the order book will change or delete traded limit order only if the `PRL` entry corresponding to
this `TRD` exists. If it is set to `true` the order book will change or delete traded limit order immediately after
receiving the `TRD` event, so the existence of the corresponding `PRL` event is unnecessary.
