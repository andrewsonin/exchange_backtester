use std::collections::VecDeque;

use crate::trader::subscriptions::{PriceToVolumeSorted, TradeInfo};
use crate::types::{OrderDirection, OrderSize, Price, Timestamp};

pub(crate) struct TradesHistory {
    queue: VecDeque<(Price, OrderSize, OrderDirection)>,
    min_price: Price,
    max_price: Price,

    total_volume: OrderSize,
    buy_volume: OrderSize,
    sell_volume: OrderSize,

    begin_time: Timestamp,
}

impl TradesHistory {
    pub(crate)
    fn new(begin_time: Timestamp) -> TradesHistory
    {
        TradesHistory {
            queue: Default::default(),
            min_price: Price(u64::MAX),
            max_price: Price(u64::MIN),
            total_volume: OrderSize(0),
            buy_volume: OrderSize(0),
            sell_volume: OrderSize(0),
            begin_time,
        }
    }

    pub(crate)
    fn push(&mut self, value: (Price, OrderSize, OrderDirection))
    {
        self.queue.push_back(value);
        let (price, size, direction) = value;
        self.total_volume += size;
        match direction {
            OrderDirection::Buy => { self.buy_volume += size }
            OrderDirection::Sell => { self.sell_volume += size }
        }
        if price > self.max_price {
            self.max_price = price
        }
        if price < self.min_price {
            self.min_price = price
        }
    }

    pub(crate)
    fn clear(&mut self, new_begin_time: Timestamp) {
        self.begin_time = new_begin_time;
        self.queue.clear();
        self.min_price = Price(u64::MAX);
        self.max_price = Price(u64::MIN);
        self.total_volume = OrderSize(0);
        self.buy_volume = OrderSize(0);
        self.sell_volume = OrderSize(0);
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
                volume: self.total_volume,
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
                OrderDirection::Buy => { entry.buy_aggressors += *size }
                OrderDirection::Sell => { entry.sell_aggressors += *size }
            }
        }
        result
    }
}