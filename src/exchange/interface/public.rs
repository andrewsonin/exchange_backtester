use crate::exchange::Exchange;
use crate::exchange::trades::history::TradesHistory;
use crate::exchange::types::EventBody;
use crate::history::parser::HistoryParser;
use crate::input::InputInterface;
use crate::message::TraderRequest;
use crate::trader::subscriptions::SubscriptionConfig;
use crate::trader::Trader;
use crate::types::Timestamp;

pub struct ExchangeBuilder<'a, T, TTC, PInfo> {
    _dummy_a: &'a T,
    _dummy_b: TTC,
    _dummy_c: PInfo,
}

impl<'a, T, TTC, PInfo> ExchangeBuilder<'_, T, TTC, PInfo>
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
            executed_trades: TradesHistory::new(first_event.timestamp),
            current_time: first_event.timestamp,
            is_trading_time: is_trading_time,
        };
        exchange.event_queue.schedule_history_event(first_event);
        // exchange.set_new_trading_period(first_event.timestamp);
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
                    self.set_new_trading_period(event.timestamp);
                    exchange_closed = false;
                }
            } else {
                if !is_trading_time { exchange_closed = true }
            }
            if DEBUG {
                eprintln!("{} :: run_trades :: EVENT :: {:?}", event.timestamp, event.body)
            }
            match event.body {
                EventBody::HistoryEvent(event) => { self.handle_history_event(event) }
                EventBody::TraderRequest(request) => {
                    match request {
                        TraderRequest::PlaceLimitOrder(order) => { self.submit_limit_order(order) }
                        TraderRequest::PlaceMarketOrder(order) => { self.submit_market_order(order) }
                        TraderRequest::CancelLimitOrder(order_id) => { self.cancel_limit_order(order_id) }
                        TraderRequest::CancelMarketOrder(order_id) => { self.cancel_market_order(order_id) }
                    }
                }
                EventBody::ExchangeReply(reply) => { self.trader.handle_exchange_reply(reply); }
                EventBody::SubscriptionUpdate(update) => { self.handle_subscription_update(update) }
                EventBody::SubscriptionSchedule(subscription_type) => {
                    self.handle_subscription_schedule(subscription_type)
                }
            }
        }
    }
}