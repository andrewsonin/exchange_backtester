use std::collections::{HashMap, HashSet, LinkedList};

use chrono::NaiveDateTime;

use crate::cli::InputInterface;
use crate::exchange::types::{EventQueue, OrderBookLevel};
use crate::history::parser::HistoryParser;
use crate::order::MarketOrder;
use crate::trader::Trader;
use crate::types::{OrderDirection, OrderID, Price};

pub(crate) mod interface;
pub(crate) mod types;

pub struct Exchange<'a, T, TradingTimeCriterion, NewSessionCriterion, ParsingInfo>
    where T: Trader,
          TradingTimeCriterion: Fn(NaiveDateTime) -> bool,
          NewSessionCriterion: Fn(NaiveDateTime, NaiveDateTime) -> bool,
          ParsingInfo: InputInterface
{
    event_queue: EventQueue,
    history_reader: HistoryParser<'a, ParsingInfo>,
    history_order_ids: HashSet<OrderID>,

    bids: LinkedList<OrderBookLevel>,
    asks: LinkedList<OrderBookLevel>,

    trader: &'a mut T,
    trader_pending_market_orders: LinkedList<MarketOrder>,
    trader_pending_limit_orders: HashMap<OrderID, (Price, OrderDirection)>,
    trader_submitted_orders: HashSet<OrderID>,

    current_time: NaiveDateTime,
    _is_next_session: NewSessionCriterion,
    _is_trading_time: TradingTimeCriterion,
}