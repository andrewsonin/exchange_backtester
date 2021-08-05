use crate::types::NonZeroU64;

pub const ONE_MICROSECOND: NonZeroU64 = NonZeroU64::new(1000).unwrap();
pub const ONE_MILLISECOND: NonZeroU64 = NonZeroU64::new(ONE_MICROSECOND.get() * 1000).unwrap();
pub const ONE_SECOND: NonZeroU64 = NonZeroU64::new(ONE_MILLISECOND.get() * 1000).unwrap();
pub const ONE_MINUTE: NonZeroU64 = NonZeroU64::new(ONE_SECOND.get() * 60).unwrap();
pub const ONE_HOUR: NonZeroU64 = NonZeroU64::new(ONE_MINUTE.get() * 60).unwrap();
pub const ONE_DAY: NonZeroU64 = NonZeroU64::new(ONE_HOUR.get() * 24).unwrap();
