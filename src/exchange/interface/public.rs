use std::num::NonZeroU64;

use crate::exchange::{
    Exchange,
    trades::history::TradesHistory,
};
use crate::history::parser::EventProcessor;
use crate::lags::interface::NanoSecondGenerator;
use crate::trader::Trader;
use crate::types::{SeedableRng, StdRng, Timestamp};

pub struct VoidNanoSecGen;

impl NanoSecondGenerator for VoidNanoSecGen {
    fn gen_ns(&mut self, _: &mut StdRng, _: Timestamp) -> NonZeroU64 { unreachable!() }
}

pub struct ExchangeBuilder<T, E> {
    _dummy_a: T,
    _dummy_b: E,
}

impl<'a, T: Trader, E: EventProcessor> ExchangeBuilder<T, E>
{
    pub
    fn new<const TRD_UPDATES_OB: bool>(
        event_processor: E,
        trader: &'a mut T,
        is_trading_time: fn(Timestamp) -> bool,
    ) -> Exchange<'a, T, E, VoidNanoSecGen, VoidNanoSecGen, VoidNanoSecGen, false, TRD_UPDATES_OB, false, false, false> {
        Exchange::build(event_processor, trader, is_trading_time)
    }

    pub
    fn new_debug<const TRD_UPDATES_OB: bool>(
        event_processor: E,
        trader: &'a mut T,
        is_trading_time: fn(Timestamp) -> bool,
    ) -> Exchange<
        'a, T, E,
        VoidNanoSecGen, VoidNanoSecGen, VoidNanoSecGen,
        true, TRD_UPDATES_OB, false, false, false
    > {
        Exchange::build(event_processor, trader, is_trading_time)
    }
}

impl<'a,
    T: Trader,
    E: EventProcessor,
    const DEBUG: bool,
    const TRD_UPDATES_OB: bool,
    const OB_SUBSCRIPTION: bool,
    const TRD_SUBSCRIPTION: bool,
    const WAKEUP_SUBSCRIPTION: bool
>
Exchange<
    'a, T, E,
    VoidNanoSecGen, VoidNanoSecGen, VoidNanoSecGen,
    DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION
>
{
    fn build(mut event_processor: E, trader: &'a mut T, is_trading_time: fn(Timestamp) -> bool) -> Exchange<
        'a, T, E,
        VoidNanoSecGen, VoidNanoSecGen, VoidNanoSecGen,
        DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION
    > {
        let first_event = match event_processor.yield_next_event() {
            Some(event) => { event }
            None => { panic!("Does not have any history events") }
        };

        let mut exchange = Exchange {
            event_queue: Default::default(),
            event_processor,
            history_order_ids: Default::default(),
            bids: Default::default(),
            asks: Default::default(),
            trader,
            trader_pending_market_orders: Default::default(),
            trader_pending_limit_orders: Default::default(),
            trader_submitted_orders: Default::default(),
            executed_trades: TradesHistory::new(),
            current_time: first_event.timestamp,
            exchange_closed: true,
            is_trading_time,
            rng: StdRng::from_entropy(),
            ob_depth_and_interval_ns: (0, VoidNanoSecGen),
            trade_info_interval_ns: VoidNanoSecGen,
            wakeup: VoidNanoSecGen,
        };
        exchange.event_queue.schedule_history_event(first_event);
        if DEBUG {
            eprintln!("{} :: build :: BEGIN", first_event.timestamp)
        }
        exchange
    }
}

impl<'a,
    T: Trader,
    E: EventProcessor,
    ObLagGen: NanoSecondGenerator,
    TrdLagGen: NanoSecondGenerator,
    WkpLagGen: NanoSecondGenerator,
    const DEBUG: bool,
    const TRD_UPDATES_OB: bool,
    const OB_SUBSCRIPTION: bool,
    const TRD_SUBSCRIPTION: bool,
    const WAKEUP_SUBSCRIPTION: bool
>
Exchange<
    'a, T, E,
    ObLagGen, TrdLagGen, WkpLagGen,
    DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION
>
{
    pub
    fn run_trades(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            self.process_next_event(event)
        }
    }

    pub
    fn ob_level_subscription_depth<G: NanoSecondGenerator>(self, ns_gen: G, depth: usize) -> Exchange<
        'a, T, E,
        G, TrdLagGen, WkpLagGen,
        DEBUG, TRD_UPDATES_OB, true, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION
    > {
        let Exchange {
            event_queue,
            event_processor,
            history_order_ids,
            bids,
            asks,
            trader,
            trader_pending_market_orders,
            trader_pending_limit_orders,
            trader_submitted_orders,
            executed_trades,
            current_time,
            exchange_closed,
            is_trading_time,
            rng,
            trade_info_interval_ns,
            wakeup,
            ..
        } = self;
        Exchange {
            event_queue,
            event_processor,
            history_order_ids,
            bids,
            asks,
            trader,
            trader_pending_market_orders,
            trader_pending_limit_orders,
            trader_submitted_orders,
            executed_trades,
            current_time,
            exchange_closed,
            is_trading_time,
            rng,
            ob_depth_and_interval_ns: (depth, ns_gen),
            trade_info_interval_ns,
            wakeup,
        }
    }

    pub
    fn ob_level_subscription_full<G: NanoSecondGenerator>(self, ns_gen: G) -> Exchange<
        'a, T, E,
        G, TrdLagGen, WkpLagGen,
        DEBUG, TRD_UPDATES_OB, true, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION
    > {
        let Exchange {
            event_queue,
            event_processor,
            history_order_ids,
            bids,
            asks,
            trader,
            trader_pending_market_orders,
            trader_pending_limit_orders,
            trader_submitted_orders,
            executed_trades,
            current_time,
            exchange_closed,
            is_trading_time,
            rng,
            trade_info_interval_ns,
            wakeup,
            ..
        } = self;
        Exchange {
            event_queue,
            event_processor,
            history_order_ids,
            bids,
            asks,
            trader,
            trader_pending_market_orders,
            trader_pending_limit_orders,
            trader_submitted_orders,
            executed_trades,
            current_time,
            exchange_closed,
            is_trading_time,
            rng,
            ob_depth_and_interval_ns: (usize::MAX, ns_gen),
            trade_info_interval_ns,
            wakeup,
        }
    }

    pub
    fn trade_info_subscription<G: NanoSecondGenerator>(self, ns_gen: G) -> Exchange<
        'a, T, E,
        ObLagGen, G, WkpLagGen,
        DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, true, WAKEUP_SUBSCRIPTION
    > {
        let Exchange {
            event_queue,
            event_processor,
            history_order_ids,
            bids,
            asks,
            trader,
            trader_pending_market_orders,
            trader_pending_limit_orders,
            trader_submitted_orders,
            executed_trades,
            current_time,
            exchange_closed,
            is_trading_time,
            rng,
            ob_depth_and_interval_ns,
            wakeup,
            ..
        } = self;
        Exchange {
            event_queue,
            event_processor,
            history_order_ids,
            bids,
            asks,
            trader,
            trader_pending_market_orders,
            trader_pending_limit_orders,
            trader_submitted_orders,
            executed_trades,
            current_time,
            exchange_closed,
            is_trading_time,
            rng,
            ob_depth_and_interval_ns,
            trade_info_interval_ns: ns_gen,
            wakeup,
        }
    }

    pub
    fn with_periodic_wakeup<G: NanoSecondGenerator>(self, ns_gen: G) -> Exchange<
        'a, T, E,
        ObLagGen, TrdLagGen, G,
        DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, TRD_SUBSCRIPTION, true
    > {
        let Exchange {
            event_queue,
            event_processor,
            history_order_ids,
            bids,
            asks,
            trader,
            trader_pending_market_orders,
            trader_pending_limit_orders,
            trader_submitted_orders,
            executed_trades,
            current_time,
            exchange_closed,
            is_trading_time,
            rng,
            ob_depth_and_interval_ns,
            trade_info_interval_ns,
            ..
        } = self;
        Exchange {
            event_queue,
            event_processor,
            history_order_ids,
            bids,
            asks,
            trader,
            trader_pending_market_orders,
            trader_pending_limit_orders,
            trader_submitted_orders,
            executed_trades,
            current_time,
            exchange_closed,
            is_trading_time,
            rng,
            ob_depth_and_interval_ns,
            trade_info_interval_ns,
            wakeup: ns_gen,
        }
    }

    pub fn seed_rng(&mut self, seed: u64) { self.rng = StdRng::seed_from_u64(seed) }
}