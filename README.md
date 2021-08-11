# Exchange backtester

Framework that allows you to replay exchange trading history and test your trading strategies.

Written on the pure Rust, it uses some experimental language features such as constant generics. This is necessary in order to instantiate (or not instantiate) machine code as efficiently as possible and calculate everything that is possible at compile time. However, as of now (August 10, 2021), this requires compiling with the `nightly` version of `rustc`.

## How to use

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
3. Implement your custom trading agent. The following is the example of `main.rs` that implements a command line app that reads the trading history and prints the middle price of the spread into the standard output every trading hour or prints the message into the standard error in the order book is empty.

   ```rust
   #![feature(const_option, const_trait_impl, const_mut_refs)]
   
   use exchange_backtester::prelude::*;
   
   struct CustomTrader {
       price_step: f64,
   }
   
   impl HandleSubscriptionUpdates for CustomTrader
   {
       fn handle_order_book_snapshot(&mut self,
                                     timestamp: Timestamp,
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
                   eprintln!("Timestamp: {}. Order book is empty", timestamp);
                   return vec![];
               }
           };
           println!("{},{}", timestamp, mid_price);
           vec![]
       }
       fn handle_trade_info_update(&mut self, _: Timestamp, _: Option<TradeInfo>) -> Vec<TraderRequest> {
           vec![]
       }
       fn handle_wakeup(&mut self, _: Timestamp) -> Vec<TraderRequest> {
           vec![]
       }
   }
   
   impl const Trader for CustomTrader {
       fn exchange_to_trader_latency(&mut self) -> u64 { 0 }
       fn trader_to_exchange_latency(&mut self) -> u64 { 0 }
       fn handle_exchange_reply(&mut self, _: ExchangeReply) -> Vec<TraderRequest> { vec![] }
       fn set_new_trading_period(&mut self) {}
   }
   
   fn main() {
       let input = ArgumentParser::parse();
       let history_parser = HistoryParser::new(&input);
   
       let mut trader = CustomTrader { price_step: input.get_price_step() };
   
       const SUBSCRIPTIONS: SubscriptionConfig = SubscriptionConfig::new()
           .ob_level_subscription_depth(constants::ONE_HOUR, NonZeroUsize::new(3).unwrap())
           .trade_info_subscription(constants::ONE_SECOND)
           .with_periodic_wakeup(constants::ONE_MINUTE);
   
       let is_trading_time = |timestamp: Timestamp| {
           match timestamp.hour() {
               7..=22 => { true }
               23 => { timestamp.minute() < 50 }
               _ => { false }
           }
       };
   
       let mut exchange = ExchangeBuilder::new::<false, SUBSCRIPTIONS>(
           history_parser,
           &mut trader,
           is_trading_time,
       );
       exchange.run_trades()
   }
   ```

## How it works

### Default version

Default version of the backtester simultaneously reads two types of the exchange history backups — `TRD` and `PRL` — that should be stored in CSV-format and sorted in ascending order by time. The first one is a representation of all trades happened, the second one is a representation of all changes in the order book.

`PRL` should have the following columns (column names can be specified differently than in the example below. This can be achieved using command line parameters of the `ArgumentParser`):
1. `Timestamp` — `chrono::NaiveDateTime`, represents the actual time of the event of a change in the state of the order book.
2. `ORDER_ID` — `u64`, a number assigned to the corresponding limit order, unique within one trading session.
3. `PRICE` — `f64`, represents the limit price of the order.
4. `SIZE` — `u64`, size of the order in lots.
5. `BUY_SELL_FLAG` — `<literal>`, represents the direction of the order. Buy orders should have one of the following values: `0/B/b/false/False`. Sell orders should have one of the following ones: `1/S/s/true/True`.

`TRD` should have the same columns except `PRICE`. But the meaning is slightly different:
1. `Timestamp` represents the actual time of the trade happened.
2. `ORDER_ID` represents the order ID of the reference limit order with which the trade happened.
3. `SIZE` represents the size of the trade.
4. `BUY_SELL_FLAG` represents the direction of the aggressor whose action led to the trade.

Different entries in the `PRL` file can have the same ID in two cases: if they correspond to different trading sessions or if they correspond to the same limit order within one session. In the latter case their meaning can be different. The first entry should reflect the event of creating a limit order. The next entries should reflect the remaining price of the limit order after trading. If the value in the `SIZE` column of the entry equals to zero, the limit order considered fully executed or cancelled (by the market maker or by the exchange at the end of the trading period).

Let's take a look at the following examples.

#### PRL-file
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

#### TRD-file
|               Timestamp | SIZE | ORDER_ID | BUY_SELL_FLAG |
| ----------------------- | ---- | -------- | ------------- |
| 2020-04-02 13:01:13.402 |   99 |        1 |             S |
| 2020-04-02 13:11:22.242 |  148 |        1 |             S |
| 2020-04-02 14:45:01.948 |    3 |        2 |             B |
| 2020-04-02 14:56:32.002 |   45 |        1 |             S |
| 2020-04-02 18:12:58.248 |   22 |        2 |             B |

What do we see here? We see that the order with `ORDER_ID = 1` was placed at `2020-04-02 13:01:11.100` and then executed at `2020-04-02 13:01:13.402`, `2020-04-02 13:11:22.242` and `2020-04-02 14:56:32.002`, but at `2020-04-02 17:00:34.123` it was cancelled. We know this because at this time this order does not have corresponding entry in the `TRD`-file.

Vice versa, the order with `ORDER_ID = 1` was placed at `2020-04-02 13:02:44.002` and then was fully executed by two subsequent trades at `2020-04-02 14:45:01.948` and `2020-04-02 14:45:01.948`.
