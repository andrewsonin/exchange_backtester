use std::cmp::Reverse;
use std::collections::{BinaryHeap, LinkedList};

use crate::history::types::{HistoryEvent, OrderOrigin};
use crate::message::{ExchangeReply, TraderRequest};
use crate::types::{OrderID, OrderSize, Price, Timestamp};

pub(crate) struct OrderBookLevel {
    pub(crate) price: Price,
    pub(crate) queue: LinkedList<OrderBookEntry>,
}

pub(crate) struct OrderBookEntry {
    pub(crate) order_id: OrderID,
    pub(crate) size: OrderSize,
    pub(crate) from: OrderOrigin,
}

pub(crate) type EventQueue = BinaryHeap<Reverse<Event>>;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Event {
    pub(crate) timestamp: Timestamp,
    pub(crate) body: EventBody,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum EventBody {
    HistoryEvent(HistoryEvent),
    TraderRequest(TraderRequest),
    ExchangeReply(ExchangeReply),
    WakeUp,
}