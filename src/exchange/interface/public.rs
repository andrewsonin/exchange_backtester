use crate::exchange::{
    Exchange,
    trades::history::TradesHistory,
};
use crate::history::parser::EventProcessor;
use crate::trader::{subscriptions::SubscriptionConfig, Trader};
use crate::types::Timestamp;

pub struct ExchangeBuilder<T, TTC, EP> {
    _dummy_a: T,
    _dummy_b: TTC,
    _dummy_c: EP,
}

impl<'a, T, TTC, EP> ExchangeBuilder<T, TTC, EP>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          EP: EventProcessor
{
    pub
    fn new<const TRD_UPDATES_OB: bool, const SUBSCRIPTIONS: SubscriptionConfig>(
        event_processor: EP,
        trader: &'a mut T,
        is_trading_time: TTC,
    ) -> Exchange<'a, T, TTC, EP, false, TRD_UPDATES_OB, SUBSCRIPTIONS>
    {
        Exchange::build(event_processor, trader, is_trading_time)
    }

    pub
    fn new_debug<const TRD_UPDATES_OB: bool, const SUBSCRIPTIONS: SubscriptionConfig>(
        event_processor: EP,
        trader: &'a mut T,
        is_trading_time: TTC,
    ) -> Exchange<'a, T, TTC, EP, true, TRD_UPDATES_OB, SUBSCRIPTIONS>
    {
        Exchange::build(event_processor, trader, is_trading_time)
    }
}

impl<'a, T, TTC, EP, const DEBUG: bool, const TRD_UPDATES_OB: bool, const SUBSCRIPTIONS: SubscriptionConfig>
Exchange<'a, T, TTC, EP, DEBUG, TRD_UPDATES_OB, SUBSCRIPTIONS>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          EP: EventProcessor
{
    fn build(mut event_processor: EP,
             trader: &'a mut T,
             is_trading_time: TTC) -> Exchange<'a, T, TTC, EP, DEBUG, TRD_UPDATES_OB, SUBSCRIPTIONS>
    {
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
}