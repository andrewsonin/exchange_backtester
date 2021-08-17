use crate::types::{NonZeroU64, StdRng, Timestamp};

pub mod constant;

pub type NanoSecondGenerator = fn(&mut StdRng, Timestamp) -> NonZeroU64;
