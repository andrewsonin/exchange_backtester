use crate::types::{DateTime, NonZeroU64, StdRng};

pub trait NanoSecondGenerator {
    fn gen_ns(&mut self, rng: &mut StdRng, dt: DateTime) -> Option<NonZeroU64>;
}