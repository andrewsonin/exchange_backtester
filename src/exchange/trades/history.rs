use crate::types::{DateTime, Direction, Price, Size};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExecutedTrade {
    pub datetime: DateTime,
    pub price: Price,
    pub size: Size,
    pub direction: Direction,
}

#[derive(Default)]
pub(crate) struct TradesHistory(Vec<ExecutedTrade>);

impl TradesHistory {
    pub(crate) fn push(&mut self, trade: ExecutedTrade) { self.0.push(trade) }

    pub(crate) fn yield_trade_info(&mut self) -> Vec<ExecutedTrade> {
        let mut result = Default::default();
        std::mem::swap(&mut self.0, &mut result);
        result
    }
}