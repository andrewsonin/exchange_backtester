#![feature(const_fn_fn_ptr_basics, const_panic, const_generics, const_trait_impl, const_fn_trait_bound, const_mut_refs, const_option, linked_list_cursors)]

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
    pub use crate::exchange::{Exchange, interface::public::ExchangeBuilder};
    pub use crate::history::{parser::{HistoryParser, interface::EventProcessor}, types::*};
    pub use crate::input;
    pub use crate::input::{cli::{ArgumentParser, Clap}, inline::StaticInput, InputInterface};
    pub use crate::lags;
    pub use crate::message::{
        CancellationReason,
        DiscardingReason,
        ExchangeReply,
        InabilityToCancelReason,
        TraderRequest,
    };
    pub use crate::order::*;
    pub use crate::trader::{
        examples,
        subscriptions::{HandleSubscriptionUpdates, OrderBookSnapshot, TradeInfo},
        Trader,
    };
    pub use crate::types::{
        Date,
        Direction,
        Duration,
        NonZeroU64,
        NonZeroUsize,
        OrderID,
        Price,
        Size,
        Time,
        Timelike,
        Timestamp,
    };
    pub use crate::utils::ExpectWith;
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

        let is_trading_time = |timestamp: Timestamp| {
            match timestamp.date().cmp(&Date::from_ymd(2019, 3, 4)) {
                Ordering::Less => { true }
                Ordering::Equal => { timestamp.time() < Time::from_hms(12, 11, 12) }
                Ordering::Greater => { false }
            }
        };

        let exchange = ExchangeBuilder::new_debug::<false>(
            history_parser,
            &mut trader,
            is_trading_time,
        );
        let mut exchange = exchange
            .ob_level_subscription_depth(lags::constant::one_second, 10)
            .trade_info_subscription(lags::constant::one_second)
            .with_periodic_wakeup(lags::constant::one_minute);

        exchange.run_trades()
    }
}