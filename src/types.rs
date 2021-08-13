pub use std::num::{NonZeroU64, NonZeroUsize};
use std::str::FromStr;

pub use chrono::{Duration, NaiveDateTime as Timestamp, Timelike};
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};

use crate::utils::ExpectWith;

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy, Add, Sub, AddAssign, SubAssign)]
pub struct Price(pub u64);

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy, Add, Sub, AddAssign, SubAssign)]
pub struct OrderID(pub u64);

#[derive(Debug, Default, PartialOrd, PartialEq, Ord, Eq, Hash, Clone, Copy, Add, Sum, Sub, AddAssign, SubAssign)]
pub struct Size(pub u64);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Direction {
    Buy,
    Sell,
}

impl Price
{
    pub
    fn from_decimal_str(string: &str, price_step: f64) -> Self
    {
        let parsed_f64 = f64::from_str(string).expect_with(
            || format!("Cannot parse to f64: {}", string)
        );
        Self::from_f64(parsed_f64, price_step)
    }

    pub
    fn from_f64(value: f64, price_step: f64) -> Self {
        let price_steps = value / price_step;
        let rounded_price_steps = price_steps.round();
        if (rounded_price_steps - price_steps).abs() > 10e-12 {
            panic!(
                "Cannot convert f64 {} to Price without loss of precision \
                with the following price step: {}",
                value,
                price_step
            )
        }
        Price(rounded_price_steps as u64)
    }

    pub
    fn to_f64(&self, price_step: f64) -> f64 {
        self.0 as f64 * price_step
    }
}

impl const Into<u64> for Price {
    fn into(self) -> u64 { self.0 }
}