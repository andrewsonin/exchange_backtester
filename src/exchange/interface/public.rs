use chrono::NaiveDateTime;

use crate::cli::InputInterface;
use crate::exchange::Exchange;
use crate::exchange::types::EventBody;
use crate::history::parser::HistoryParser;
use crate::message::TraderRequest;
use crate::trader::Trader;

impl<'a, T, TTC, NSC, PInfo> Exchange<'a, T, TTC, NSC, PInfo>
    where T: Trader,
          TTC: Fn(NaiveDateTime) -> bool,
          NSC: Fn(NaiveDateTime, NaiveDateTime) -> bool,
          PInfo: InputInterface
{
    pub
    fn new(args: &'a PInfo,
           trader: &'a mut T,
           is_trading_time: TTC,
           is_next_session: NSC) -> Exchange<'a, T, TTC, NSC, PInfo>
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
            current_time: first_event.timestamp,
            _is_next_session: is_next_session,
            _is_trading_time: is_trading_time,
        };
        exchange.schedule_history_event(first_event);
        exchange.set_new_trading_period(first_event.timestamp);
        exchange
    }

    pub
    fn run_trades(&mut self)
    {
        while let Some(event) = self.event_queue.pop()
        {
            let event = event.0;
            if self.is_next_session(event.timestamp) {
                self.set_new_trading_period(event.timestamp);
                self.cleanup()
            }
            self.current_time = event.timestamp;
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
                EventBody::ExchangeReply(reply) => { self.trader.handle_exchange_reply(reply) }
                EventBody::WakeUp => { self.handle_wakeup() }
            }
        }
    }
}