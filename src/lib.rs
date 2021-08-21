#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn_trait_bound)]
#![feature(const_generics)]
#![feature(const_mut_refs)]
#![feature(const_option)]
#![feature(const_panic)]
#![feature(const_trait_impl)]
#![feature(linked_list_cursors)]

mod utils;
mod types;
mod exchange;
mod order;
mod history;

pub mod trader;
pub mod message;
pub mod input;
pub mod lags;

pub mod prelude {
    pub use crate::{
        exchange::{Exchange, interface::public::ExchangeBuilder, trades::history::ExecutedTrade},
        history::{
            parser::{HistoryParser, interface::EventProcessor},
            types::{HistoryEvent, HistoryEventBody},
        },
        input,
        input::{cli::{ArgumentParser, Clap}, inline::StaticInput, InputInterface},
        lags,
        lags::interface::NanoSecondGenerator,
        message::{
            CancellationReason,
            DiscardingReason,
            ExchangeReply,
            InabilityToCancelReason,
            TraderRequest,
        },
        order::{LimitOrder, MarketOrder},
        trader::{
            examples,
            subscriptions::{HandleSubscriptionUpdates, OrderBookSnapshot, TradeInfo},
            Trader,
        },
        types::{
            Date,
            DateTime,
            Direction,
            Duration,
            NonZeroU64,
            NonZeroUsize,
            OrderID,
            Price,
            Rng,
            SeedableRng,
            Size,
            StdRng,
            Time,
            Timelike,
        },
        utils::ExpectWith,
    };
}

#[cfg(test)]
mod integration {
    use std::cmp::Ordering;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use crate::prelude::*;

    const SOURCE_DIR: &str = env!("CARGO_MANIFEST_DIR");

    fn prepare_testing(test_name: &str) -> StaticInput {
        let test_dir = Path::new(SOURCE_DIR)
            .join("tests")
            .join("data")
            .join("integration")
            .join(test_name);
        let prl_files = test_dir.join("PRL.txt");
        let trd_files = test_dir.join("TRD.txt");

        File::create(&prl_files)
            .expect_with(|| format!("Unable to create file: {:?}", prl_files))
            .write_all(format!("{}\n{}",
                               test_dir.join("PRL_01.csv").to_str().unwrap(),
                               test_dir.join("PRL_02.csv").to_str().unwrap()).as_ref())
            .expect_with(|| format!("Unable to write to {:?}", prl_files));

        File::create(&trd_files)
            .expect_with(|| format!("Unable to create file: {:?}", prl_files))
            .write_all(format!("{}\n{}",
                               test_dir.join("TRD_01.csv").to_str().unwrap(),
                               test_dir.join("TRD_02.csv").to_str().unwrap()).as_ref())
            .expect_with(|| format!("Unable to write to {:?}", prl_files));

        StaticInput::new()
            .with_prl_files(prl_files.to_str().unwrap())
            .with_trd_files(trd_files.to_str().unwrap())
    }

    #[test]
    fn test_01() {
        let input = prepare_testing("test_01");
        let history_parser = HistoryParser::new(&input);
        let mut trader = examples::VoidTrader;

        let is_trading_dt = |datetime: DateTime| {
            match datetime.date().cmp(&Date::from_ymd(2019, 3, 4)) {
                Ordering::Less => { true }
                Ordering::Equal => { datetime.time() < Time::from_hms(12, 11, 12) }
                Ordering::Greater => { false }
            }
        };

        let exchange = ExchangeBuilder::new_debug::<false>(
            history_parser,
            &mut trader,
            is_trading_dt,
        );
        let mut exchange = exchange
            .ob_level_subscription_depth(lags::constant::ONE_SECOND, 10)
            .trade_info_subscription(lags::constant::ONE_SECOND)
            .with_periodic_wakeup(lags::constant::ONE_MINUTE);

        exchange.run_trades()
    }
}