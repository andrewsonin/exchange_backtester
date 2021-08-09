use crate::history::types::HistoryEventWithTime;

pub trait HistoryEventProcessor {
    fn yield_next_event(&mut self) -> Option<HistoryEventWithTime>;
}