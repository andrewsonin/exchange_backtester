use crate::types::{NonZeroU64, StdRng, Timestamp};

pub trait NanoSecondGenerator {
    fn gen_ns(&mut self, rng: &mut StdRng, ts: Timestamp) -> NonZeroU64;
}