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