#![feature(const_generics, const_trait_impl, const_mut_refs, const_option, linked_list_cursors)]

mod utils;
mod types;
mod exchange;
mod order;
mod history;

pub mod trader;
pub mod message;
pub mod input;
pub mod constants;

pub mod prelude {
    pub use crate::constants;
    pub use crate::exchange::Exchange;
    pub use crate::exchange::interface::public::ExchangeBuilder;
    pub use crate::input;
    pub use crate::input::{cli::ArgumentParser, cli::Clap, inline::StaticInput};
    pub use crate::message::{
        CancellationReason,
        DiscardingReason,
        ExchangeReply,
        InabilityToCancelReason,
        TraderRequest,
    };
    pub use crate::trader::{examples, Trader};
    pub use crate::trader::subscriptions;
    pub use crate::types::{
        Duration,
        NonZeroU64,
        OrderDirection,
        OrderID,
        OrderSize,
        Price,
        Timestamp,
    };
}

#[cfg(test)]
mod integration {
    use std::fs::File;
    use std::io::Write;
    use std::num::NonZeroUsize;
    use std::path::Path;

    use subscriptions::SubscriptionConfig;

    use crate::prelude::*;
    use crate::utils::{ExpectWith, SOURCE_DIR};

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
        let mut trader = examples::VoidTrader;

        let end_of_trades = Timestamp::parse_from_str("2019-03-04 12:11:12", "%Y-%m-%d %H:%M:%S")
            .unwrap();
        let is_trading_time = |timestamp: Timestamp| {
            timestamp < end_of_trades
        };
        const SUBSCRIPTIONS: SubscriptionConfig = SubscriptionConfig::new()
            .ob_level_subscription_depth(constants::ONE_SECOND, NonZeroUsize::new(10).unwrap())
            .trade_info_subscription(constants::ONE_SECOND);

        let mut exchange = ExchangeBuilder::new_debug::<SUBSCRIPTIONS>(
            &input,
            &mut trader,
            is_trading_time,
        );
        exchange.run_trades()
    }
}