use crate::types::{NonZeroU64, StdRng, Timestamp};

const ONE_MICROSECOND: NonZeroU64 = NonZeroU64::new(1000).unwrap();
const ONE_MILLISECOND: NonZeroU64 = NonZeroU64::new(ONE_MICROSECOND.get() * 1000).unwrap();
const ONE_SECOND: NonZeroU64 = NonZeroU64::new(ONE_MILLISECOND.get() * 1000).unwrap();
const ONE_MINUTE: NonZeroU64 = NonZeroU64::new(ONE_SECOND.get() * 60).unwrap();
const ONE_HOUR: NonZeroU64 = NonZeroU64::new(ONE_MINUTE.get() * 60).unwrap();
const ONE_DAY: NonZeroU64 = NonZeroU64::new(ONE_HOUR.get() * 24).unwrap();

pub const fn one_microsecond(_: &mut StdRng, _: Timestamp) -> NonZeroU64 { ONE_MICROSECOND }

pub const fn one_millisecond(_: &mut StdRng, _: Timestamp) -> NonZeroU64 { ONE_MILLISECOND }

pub const fn one_second(_: &mut StdRng, _: Timestamp) -> NonZeroU64 { ONE_SECOND }

pub const fn one_minute(_: &mut StdRng, _: Timestamp) -> NonZeroU64 { ONE_MINUTE }

pub const fn one_hour(_: &mut StdRng, _: Timestamp) -> NonZeroU64 { ONE_HOUR }

pub const fn one_day(_: &mut StdRng, _: Timestamp) -> NonZeroU64 { ONE_DAY }
