use std::cmp::Reverse;
use std::collections::{BinaryHeap, LinkedList};

use rand::rngs::StdRng;

use crate::history::types::{HistoryEvent, HistoryEventBody, OrderOrigin};
use crate::message::{ExchangeReply, SubscriptionSchedule, SubscriptionUpdate, TraderRequest};
use crate::trader::Trader;
use crate::types::{DateTime, Duration, OrderID, Price, Size};

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
pub(crate) struct EventQueue(pub(crate) BinaryHeap<Reverse<Event>>);

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Event {
    pub(crate) datetime: DateTime,
    pub(crate) body: EventBody,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum EventBody {
    ExchangeOpenTryout,
    HistoryEvent(HistoryEventBody),
    TraderRequest(TraderRequest),
    ExchangeReply(ExchangeReply, DateTime),
    SubscriptionUpdate(SubscriptionUpdate, DateTime),
    SubscriptionSchedule(SubscriptionSchedule),
    TraderWakeUp,
    ExchangeClosed,
}

impl Extend<Event> for EventQueue {
    fn extend<I>(&mut self, iter: I)
        where I: IntoIterator<Item=Event>
    {
        self.0.extend(iter.into_iter().map(Reverse))
    }
}

impl EventQueue {
    pub(crate) fn peek(&self) -> Option<&Event> {
        match self.0.peek() {
            Some(Reverse(event)) => { Some(event) }
            _ => { None }
        }
    }

    pub(crate) fn pop(&mut self) -> Option<Event> {
        match self.0.pop() {
            Some(Reverse(event)) => { Some(event) }
            _ => { None }
        }
    }

    pub(crate) fn push(&mut self, item: Event) {
        self.0.push(Reverse(item))
    }

    pub(crate) fn schedule_reply_for_trader<T: Trader>(&mut self,
                                                       reply: ExchangeReply,
                                                       exchange_dt: DateTime,
                                                       rng: &mut StdRng) {
        self.push(
            Event {
                datetime: exchange_dt + Duration::nanoseconds(T::exchange_to_trader_latency(rng, exchange_dt) as i64),
                body: EventBody::ExchangeReply(reply, exchange_dt),
            }
        )
    }

    pub(crate) fn schedule_history_event(&mut self, event: HistoryEvent) {
        self.push(
            Event {
                datetime: event.datetime,
                body: EventBody::HistoryEvent(event.event),
            }
        )
    }
}