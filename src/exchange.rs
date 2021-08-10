use std::collections::{HashMap, HashSet, LinkedList};

use crate::exchange::{trades::history::TradesHistory, types::{EventQueue, OrderBookLevel}};
use crate::history::parser::EventProcessor;
use crate::order::MarketOrder;
use crate::trader::{subscriptions::SubscriptionConfig, Trader};
use crate::types::{Direction, OrderID, Price, Timestamp};

pub(crate) mod interface;
pub(crate) mod types;
pub(crate) mod trades;

pub struct Exchange<
    'a, T, TradingTimeCriterion, EP,
    const DEBUG: bool,
    const TRD_UPDATES_OB: bool,
    const SUBSCRIPTIONS: SubscriptionConfig
>
    where T: Trader,
          TradingTimeCriterion: Fn(Timestamp) -> bool,
          EP: EventProcessor
{
    event_queue: EventQueue,
    event_processor: EP,
    history_order_ids: HashSet<OrderID>,

    bids: LinkedList<OrderBookLevel>,
    asks: LinkedList<OrderBookLevel>,

    trader: &'a mut T,
    trader_pending_market_orders: LinkedList<MarketOrder>,
    trader_pending_limit_orders: HashMap<OrderID, (Price, Direction)>,
    trader_submitted_orders: HashSet<OrderID>,

    executed_trades: TradesHistory,

    current_time: Timestamp,
    exchange_closed: bool,
    is_trading_time: TradingTimeCriterion,
}