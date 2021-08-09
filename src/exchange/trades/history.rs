use std::collections::VecDeque;

use crate::trader::subscriptions::{PriceToVolumeSorted, TradeInfo};
use crate::types::{Direction, Price, Size};

pub(crate) struct TradesHistory {
    queue: VecDeque<(Price, Size, Direction)>,
    min_price: Price,
    max_price: Price,

    buy_volume: Size,
    sell_volume: Size,
}

impl TradesHistory {
    pub(crate) fn new() -> TradesHistory
    {
        TradesHistory {
            queue: Default::default(),
            min_price: Price(u64::MAX),
            max_price: Price(u64::MIN),
            buy_volume: Size(0),
            sell_volume: Size(0),
        }
    }

    pub(crate)
    fn push(&mut self, value: (Price, Size, Direction))
    {
        self.queue.push_back(value);
        let (price, size, direction) = value;
        match direction {
            Direction::Buy => { self.buy_volume += size }
            Direction::Sell => { self.sell_volume += size }
        }
        if price > self.max_price {
            self.max_price = price
        }
        if price < self.min_price {
            self.min_price = price
        }
    }

    pub(crate)
    fn clear(&mut self) {
        self.queue.clear();
        self.min_price = Price(u64::MAX);
        self.max_price = Price(u64::MIN);
        self.buy_volume = Size(0);
        self.sell_volume = Size(0);
    }

    pub(crate)
    fn get_trade_info(&self) -> Option<TradeInfo> {
        let (open, close) = if let Some((open, _, _)) = self.queue.front() {
            (*open, self.queue.back().unwrap().0)
        } else {
            return None;
        };
        Some(
            TradeInfo {
                open,
                high: self.max_price,
                low: self.min_price,
                close,
                buy_volume: self.buy_volume,
                sell_volume: self.sell_volume,
                price_to_volume_sorted: self.get_trade_volumes(),
            }
        )
    }

    fn get_trade_volumes(&self) -> PriceToVolumeSorted {
        let mut result = PriceToVolumeSorted::new();
        for (price, size, direction) in self.queue.iter() {
            let entry = result.entry(*price).or_default();
            match direction {
                Direction::Buy => { entry.buy_aggressors += *size }
                Direction::Sell => { entry.sell_aggressors += *size }
            }
        }
        result
    }
}