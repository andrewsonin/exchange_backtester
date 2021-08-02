#![feature(const_generics, const_trait_impl, const_mut_refs, const_option, linked_list_cursors)]

mod utils;
mod types;
mod exchange;
mod order;
mod history;

pub mod cli;
pub mod trader;
pub mod message;

pub mod prelude {
    pub use chrono::NaiveDateTime as Timestamp;

    pub use crate::cli::*;
    pub use crate::exchange::Exchange;
    pub use crate::message::*;
    pub use crate::trader::*;
}