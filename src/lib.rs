#![feature(const_generics, const_trait_impl, const_mut_refs, const_option, linked_list_cursors)]

mod utils;
mod types;
mod exchange;
mod order;
mod history;

pub mod trader;
pub mod message;
pub mod input;

pub mod prelude {
    pub use crate::exchange::Exchange;
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
    use std::path::Path;

    use crate::prelude::*;
    use crate::types::Timelike;
    use crate::utils::{ExpectWith, SOURCE_DIR};

    fn prepare_testing(test_name: &str) -> StaticInput {
        let test_dir = Path::new(SOURCE_DIR)
            .join("tests")
            .join("integration")
            .join(test_name);
        let prl_files = test_dir.join("PRL.txt");
        let trd_files = test_dir.join("TRD.txt");

        let mut f = File::create(&prl_files)
            .expect_with(|| format!("Unable to create file: {:?}", prl_files));
        f.write_all(format!("{}\n{}",
                            test_dir.join("PRL_01.csv").to_str().unwrap(),
                            test_dir.join("PRL_02.csv").to_str().unwrap()).as_ref())
            .expect_with(|| format!("Unable to write to {:?}", prl_files));

        let mut f = File::create(&trd_files)
            .expect_with(|| format!("Unable to create file: {:?}", prl_files));
        f.write_all(format!("{}\n{}",
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
        let is_trading_time = |timestamp: Timestamp| {
            timestamp.hour() < 13 && timestamp.minute() < 12 && timestamp.second() < 20
        };
        let is_next_session = |_: Timestamp, _: Timestamp| { false };
        let mut exchange = Exchange::new_debug(
            &input,
            &mut trader,
            is_trading_time,
            is_next_session,
        );
        exchange.run_trades()
    }
}