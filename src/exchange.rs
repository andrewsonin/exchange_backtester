use std::collections::{HashMap, HashSet, LinkedList};

use crate::exchange::{trades::history::TradesHistory, types::{EventQueue, OrderBookLevel}};
use crate::history::parser::EventProcessor;
use crate::lags::interface::NanoSecondGenerator;
use crate::order::MarketOrder;
use crate::trader::Trader;
use crate::types::{DateTime, Direction, OrderID, Price, StdRng};

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
    has_history_events_in_queue: bool,
    history_order_ids: HashSet<OrderID>,

    bids: LinkedList<OrderBookLevel>,
    asks: LinkedList<OrderBookLevel>,

    trader: &'a mut T,
    trader_pending_market_orders: LinkedList<MarketOrder>,
    trader_pending_limit_orders: HashMap<OrderID, (Price, Direction)>,
    trader_submitted_orders: HashSet<OrderID>,

    executed_trades: TradesHistory,

    current_dt: DateTime,
    exchange_closed: bool,
    get_next_open_dt: fn(DateTime) -> DateTime,
    get_next_close_dt: fn(DateTime) -> DateTime,
    rng: StdRng,

    // Subscriptions
    ob_depth_and_interval_ns: (usize, ObLagGen),
    trade_info_interval_ns: TrdLagGen,
    wakeup: WkpLagGen,
}