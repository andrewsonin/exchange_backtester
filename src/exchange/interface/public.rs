use std::num::NonZeroU64;

use crate::exchange::{
    Exchange,
    trades::history::TradesHistory,
};
use crate::history::parser::EventProcessor;
use crate::lags::NanoSecondGenerator;
use crate::trader::Trader;
use crate::types::{SeedableRng, StdRng, Timestamp};

pub struct ExchangeBuilder<T, EP> {
    _dummy_a: T,
    _dummy_b: EP,
}

impl<'a, T, EP> ExchangeBuilder<T, EP>
    where T: Trader,
          EP: EventProcessor
{
    pub
    fn new<const TRD_UPDATES_OB: bool>(
        event_processor: EP,
        trader: &'a mut T,
        is_trading_time: fn(Timestamp) -> bool,
    ) -> Exchange<'a, T, EP, false, TRD_UPDATES_OB, false, false, false>
    {
        Exchange::build(event_processor, trader, is_trading_time)
    }

    pub
    fn new_debug<const TRD_UPDATES_OB: bool>(
        event_processor: EP,
        trader: &'a mut T,
        is_trading_time: fn(Timestamp) -> bool,
    ) -> Exchange<'a, T, EP, true, TRD_UPDATES_OB, false, false, false>
    {
        Exchange::build(event_processor, trader, is_trading_time)
    }
}

impl<'a,
    T: Trader,
    EP: EventProcessor,
    const DEBUG: bool,
    const TRD_UPDATES_OB: bool,
    const OB_SUBSCRIPTION: bool,
    const TRD_SUBSCRIPTION: bool,
    const WAKEUP_SUBSCRIPTION: bool
>
Exchange<'a, T, EP, DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION>
{
    fn build(mut event_processor: EP,
             trader: &'a mut T,
             is_trading_time: fn(Timestamp) -> bool) -> Exchange<'a, T, EP, DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION>
    {
        let first_event = match event_processor.yield_next_event() {
            Some(event) => { event }
            None => { panic!("Does not have any history events") }
        };

        let nano_sec_gen_plug = |_: &mut StdRng, _: Timestamp| -> NonZeroU64 {
            unreachable!()
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
            ob_depth_and_interval_ns: (0, nano_sec_gen_plug),
            trade_info_interval_ns: nano_sec_gen_plug,
            wakeup: nano_sec_gen_plug,
        };
        exchange.event_queue.schedule_history_event(first_event);
        if DEBUG {
            eprintln!("{} :: build :: BEGIN", first_event.timestamp)
        }
        exchange
    }

    pub
    fn run_trades(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            self.process_next_event(event)
        }
    }

    pub
    fn ob_level_subscription_depth(self, ns_gen: NanoSecondGenerator, depth: usize) -> Exchange<'a, T, EP, DEBUG, TRD_UPDATES_OB, true, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION> {
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
    fn ob_level_subscription_full(self, ns_gen: NanoSecondGenerator) -> Exchange<'a, T, EP, DEBUG, TRD_UPDATES_OB, true, TRD_SUBSCRIPTION, WAKEUP_SUBSCRIPTION> {
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
    fn trade_info_subscription(self, ns_gen: NanoSecondGenerator) -> Exchange<'a, T, EP, DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, true, WAKEUP_SUBSCRIPTION> {
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
    fn with_periodic_wakeup(self, ns_gen: NanoSecondGenerator) -> Exchange<'a, T, EP, DEBUG, TRD_UPDATES_OB, OB_SUBSCRIPTION, TRD_SUBSCRIPTION, true> {
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