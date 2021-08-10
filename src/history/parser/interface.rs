use crate::history::types::HistoryEvent;

pub trait EventProcessor {
    fn yield_next_event(&mut self) -> Option<HistoryEvent>;
}