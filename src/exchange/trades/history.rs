use crate::types::{Direction, Price, Size, Timestamp};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HistoryTrade {
    pub datetime: Timestamp,
    pub price: Price,
    pub size: Size,
    pub direction: Direction,
}

#[derive(Default)]
pub(crate) struct TradesHistory(Vec<HistoryTrade>);

impl TradesHistory {
    pub(crate) fn push(&mut self, trade: HistoryTrade) { self.0.push(trade) }

    pub(crate) fn yield_trade_info(&mut self) -> Vec<HistoryTrade> {
        let mut result = Default::default();
        std::mem::swap(&mut self.0, &mut result);
        result
    }
}