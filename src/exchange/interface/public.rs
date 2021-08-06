use crate::exchange::{
    Exchange,
    trades::history::TradesHistory,
    types::EventBody::{ExchangeReply, HistoryEvent, SubscriptionSchedule, SubscriptionUpdate, TraderRequest},
};
use crate::history::parser::HistoryParser;
use crate::input::InputInterface;
use crate::trader::{subscriptions::SubscriptionConfig, Trader};
use crate::types::Timestamp;

pub struct ExchangeBuilder<T, TTC, PInfo> {
    _dummy_a: T,
    _dummy_b: TTC,
    _dummy_c: PInfo,
}

impl<'a, T, TTC, PInfo> ExchangeBuilder<T, TTC, PInfo>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          PInfo: InputInterface
{
    pub
    fn new<const SUBSCRIPTIONS: SubscriptionConfig>(
        args: &'a PInfo,
        trader: &'a mut T,
        is_trading_time: TTC,
    ) -> Exchange<'a, T, TTC, PInfo, false, SUBSCRIPTIONS>
    {
        Exchange::build(args, trader, is_trading_time)
    }

    pub
    fn new_debug<const SUBSCRIPTIONS: SubscriptionConfig>(
        args: &'a PInfo,
        trader: &'a mut T,
        is_trading_time: TTC,
    ) -> Exchange<'a, T, TTC, PInfo, true, SUBSCRIPTIONS>
        where T: Trader,
              TTC: Fn(Timestamp) -> bool,
              PInfo: InputInterface
    {
        Exchange::build(args, trader, is_trading_time)
    }
}

impl<'a, T, TTC, PInfo, const DEBUG: bool, const SUBSCRIPTIONS: SubscriptionConfig>
Exchange<'a, T, TTC, PInfo, DEBUG, SUBSCRIPTIONS>
    where T: Trader,
          TTC: Fn(Timestamp) -> bool,
          PInfo: InputInterface
{
    fn build(args: &'a PInfo,
             trader: &'a mut T,
             is_trading_time: TTC) -> Exchange<'a, T, TTC, PInfo, DEBUG, SUBSCRIPTIONS>
    {
        let mut history_reader = HistoryParser::new(args);
        let first_event = match history_reader.yield_next_event() {
            Some(event) => { event }
            None => { panic!("Does not have any history events") }
        };

        let mut exchange = Exchange {
            event_queue: Default::default(),
            history_reader,
            history_order_ids: Default::default(),
            bids: Default::default(),
            asks: Default::default(),
            trader,
            trader_pending_market_orders: Default::default(),
            trader_pending_limit_orders: Default::default(),
            trader_submitted_orders: Default::default(),
            executed_trades: TradesHistory::new(),
            current_time: first_event.timestamp,
            is_trading_time,
        };
        exchange.event_queue.schedule_history_event(first_event);
        if DEBUG {
            eprintln!("{} :: build :: BEGIN", first_event.timestamp)
        }
        exchange
    }

    pub
    fn run_trades(&mut self)
    {
        let mut exchange_closed = true;
        while let Some(event) = self.event_queue.pop()
        {
            let is_trading_time = (self.is_trading_time)(event.timestamp);
            self.current_time = event.timestamp;
            if exchange_closed {
                if is_trading_time {
                    if DEBUG {
                        eprintln!("{} :: run_trades :: CLEANUP", event.timestamp)
                    }
                    self.cleanup();
                    self.set_new_trading_period();
                    exchange_closed = false;
                }
            } else if !is_trading_time {
                exchange_closed = true
            }
            if DEBUG {
                eprintln!("{} :: run_trades :: EVENT :: {:?}", event.timestamp, event.body)
            }
            match event.body {
                HistoryEvent(event) => { self.handle_history_event(event) }
                TraderRequest(request) => { self.handle_trader_request(request) }
                ExchangeReply(reply) => { self.handle_exchange_reply(reply) }
                SubscriptionUpdate(update) => { self.handle_subscription_update(update) }
                SubscriptionSchedule(subscription_type) => { self.handle_subscription_schedule(subscription_type) }
            }
        }
    }
}