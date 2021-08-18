use std::collections::{HashMap, HashSet, LinkedList};

use crate::exchange::{trades::history::TradesHistory, types::{EventQueue, OrderBookLevel}};
use crate::history::parser::EventProcessor;
use crate::lags::interface::NanoSecondGenerator;
use crate::order::MarketOrder;
use crate::trader::Trader;
use crate::types::{Direction, OrderID, Price, StdRng, Timestamp};

pub(crate) mod interface;
pub(crate) mod types;
pub(crate) mod trades;

pub struct Exchange<
    'a,
    T: Trader,
    E: EventProcessor,
    ObLagGen: NanoSecondGenerator,
    TrdLagGen: NanoSecondGenerator,
    WkpLagGen: NanoSecondGenerator,
    const DEBUG: bool,
    const TRD_UPDATES_OB: bool,
    const OB_SUBSCRIPTION: bool,
    const TRD_SUBSCRIPTION: bool,
    const WAKEUP_SUBSCRIPTION: bool,
>
{
    event_queue: EventQueue,
    event_processor: E,
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
    is_trading_time: fn(Timestamp) -> bool,
    rng: StdRng,

    // Subscriptions
    ob_depth_and_interval_ns: (usize, ObLagGen),
    trade_info_interval_ns: TrdLagGen,
    wakeup: WkpLagGen,
}