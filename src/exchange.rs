use std::collections::{HashMap, HashSet, LinkedList};

use crate::exchange::trades::history::TradesHistory;
use crate::exchange::types::{EventQueue, OrderBookLevel};
use crate::history::parser::HistoryParser;
use crate::input::InputInterface;
use crate::order::MarketOrder;
use crate::trader::subscriptions::SubscriptionConfig;
use crate::trader::Trader;
use crate::types::{Direction, OrderID, Price, Timestamp};

pub(crate) mod interface;
pub(crate) mod types;
pub(crate) mod trades;

pub struct Exchange<
    'a, T, TradingTimeCriterion, ParsingInfo,
    const DEBUG: bool,
    const SUBSCRIPTIONS: SubscriptionConfig
>
    where T: Trader,
          TradingTimeCriterion: Fn(Timestamp) -> bool,
          ParsingInfo: InputInterface
{
    event_queue: EventQueue,
    history_reader: HistoryParser<'a, ParsingInfo>,
    history_order_ids: HashSet<OrderID>,

    bids: LinkedList<OrderBookLevel>,
    asks: LinkedList<OrderBookLevel>,

    trader: &'a mut T,
    trader_pending_market_orders: LinkedList<MarketOrder>,
    trader_pending_limit_orders: HashMap<OrderID, (Price, Direction)>,
    trader_submitted_orders: HashSet<OrderID>,

    executed_trades: TradesHistory,

    current_time: Timestamp,
    is_trading_time: TradingTimeCriterion,
}