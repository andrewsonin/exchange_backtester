use crate::lags::interface::NanoSecondGenerator;
use crate::types::{DateTime, NonZeroU64, StdRng};

pub struct ConstNanoSecondGenerator(pub NonZeroU64);

impl const NanoSecondGenerator for ConstNanoSecondGenerator {
    fn gen_ns(&mut self, _: &mut StdRng, _: DateTime) -> Option<NonZeroU64> { Some(self.0) }
}

const _ONE_MICROSECOND: NonZeroU64 = NonZeroU64::new(1000).unwrap();
const _ONE_MILLISECOND: NonZeroU64 = NonZeroU64::new(_ONE_MICROSECOND.get() * 1000).unwrap();
const _ONE_SECOND: NonZeroU64 = NonZeroU64::new(_ONE_MILLISECOND.get() * 1000).unwrap();
const _ONE_MINUTE: NonZeroU64 = NonZeroU64::new(_ONE_SECOND.get() * 60).unwrap();
const _ONE_HOUR: NonZeroU64 = NonZeroU64::new(_ONE_MINUTE.get() * 60).unwrap();
const _ONE_DAY: NonZeroU64 = NonZeroU64::new(_ONE_HOUR.get() * 24).unwrap();

pub const ONE_MICROSECOND: ConstNanoSecondGenerator = ConstNanoSecondGenerator(_ONE_MICROSECOND);
pub const ONE_MILLISECOND: ConstNanoSecondGenerator = ConstNanoSecondGenerator(_ONE_MILLISECOND);
pub const ONE_SECOND: ConstNanoSecondGenerator = ConstNanoSecondGenerator(_ONE_SECOND);
pub const ONE_MINUTE: ConstNanoSecondGenerator = ConstNanoSecondGenerator(_ONE_MINUTE);
pub const ONE_HOUR: ConstNanoSecondGenerator = ConstNanoSecondGenerator(_ONE_HOUR);
pub const ONE_DAY: ConstNanoSecondGenerator = ConstNanoSecondGenerator(_ONE_DAY);