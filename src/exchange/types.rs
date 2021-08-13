use std::cmp::Reverse;
use std::collections::{BinaryHeap, LinkedList};

use crate::history::types::{HistoryEvent, HistoryEventBody, OrderOrigin};
use crate::message::{ExchangeReply, SubscriptionSchedule, SubscriptionUpdate, TraderRequest};
use crate::trader::Trader;
use crate::types::{Duration, OrderID, Price, Size, Timestamp};

pub(crate) struct OrderBookLevel {
    pub(crate) price: Price,
    pub(crate) queue: LinkedList<OrderBookEntry>,
}

impl OrderBookLevel {
    pub(crate) fn get_ob_level_size(&self) -> Size {
        self.queue.iter().map(|order| order.size).sum()
    }
}

pub(crate) struct OrderBookEntry {
    pub(crate) order_id: OrderID,
    pub(crate) size: Size,
    pub(crate) from: OrderOrigin,
}

#[derive(Default)]
pub(crate) struct EventQueue(BinaryHeap<Reverse<Event>>);

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Event {
    pub(crate) timestamp: Timestamp,
    pub(crate) body: EventBody,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum EventBody {
    HistoryEvent(HistoryEventBody),
    TraderRequest(TraderRequest),
    ExchangeReply(ExchangeReply, Timestamp),
    SubscriptionUpdate(SubscriptionUpdate, Timestamp),
    SubscriptionSchedule(SubscriptionSchedule),
    TraderWakeUp,
}

impl Extend<Event> for EventQueue {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item=Event>
    {
        self.0.extend(iter.into_iter().map(Reverse))
    }
}

impl EventQueue {
    pub(crate) fn push(&mut self, item: Event) {
        self.0.push(Reverse(item))
    }

    pub(crate) fn pop(&mut self) -> Option<Event> {
        match self.0.pop() {
            Some(Reverse(event)) => { Some(event) }
            None => { None }
        }
    }

    pub(crate) fn schedule_reply_for_trader(&mut self,
                                            reply: ExchangeReply,
                                            exchange_ts: Timestamp,
                                            trader: &mut dyn Trader) {
        self.push(
            Event {
                timestamp: exchange_ts + Duration::nanoseconds(trader.exchange_to_trader_latency() as i64),
                body: EventBody::ExchangeReply(reply, exchange_ts),
            }
        )
    }

    pub(crate) fn schedule_history_event(&mut self, event: HistoryEvent) {
        self.push(
            Event {
                timestamp: event.timestamp,
                body: EventBody::HistoryEvent(event.event),
            }
        )
    }
}