use std::cmp::min;

pub use interface::HistoryEventProcessor;

use crate::history::{
    reader::{PRLReader, TRDReader},
    types::HistoryEventWithTime,
};
use crate::input::InputInterface;
use crate::types::Timestamp;

pub mod interface;

pub struct HistoryParser<'a, ParsingInfo>
    where ParsingInfo: InputInterface
{
    prl_parser: PRLReader<'a, ParsingInfo>,
    trd_parser: TRDReader<'a, ParsingInfo>,

    last_prl: Option<HistoryEventWithTime>,
    last_trd: Option<HistoryEventWithTime>,

    last_time: Timestamp,
}

impl<ParsingInfo: InputInterface> HistoryParser<'_, ParsingInfo>
{
    pub fn new(args: &ParsingInfo) -> HistoryParser<ParsingInfo>
    {
        let mut prl_parser = PRLReader::new(args.get_prl_files(), args);
        let mut trd_parser = TRDReader::new(args.get_trd_files(), args);
        let last_prl = prl_parser.next();
        let last_trd = trd_parser.next();
        let last_time = match (last_prl, last_trd) {
            (Some(prl), Some(trd)) => { min(prl.timestamp, trd.timestamp) }
            (Some(prl), _) => { prl.timestamp }
            (_, Some(trd)) => { trd.timestamp }
            _ => { unreachable!() }
        };
        HistoryParser {
            prl_parser,
            trd_parser,
            last_prl,
            last_trd,
            last_time,
        }
    }
}

impl<T: InputInterface> HistoryEventProcessor for HistoryParser<'_, T>
{
    fn yield_next_event(&mut self) -> Option<HistoryEventWithTime>
    {
        match (&self.last_trd, &self.last_prl) {
            (Some(trd), Some(prl)) => {
                if prl.timestamp < trd.timestamp {
                    let res = *prl;
                    if res.timestamp < self.last_time {
                        panic!("History file entries are not stored in ascending order by time.")
                    }
                    self.last_time = res.timestamp;
                    self.last_prl = self.prl_parser.next();
                    Some(res)
                } else {
                    let res = *trd;
                    if res.timestamp < self.last_time {
                        panic!("History file entries are not stored in ascending order by time.")
                    }
                    self.last_time = res.timestamp;
                    self.last_trd = self.trd_parser.next();
                    Some(res)
                }
            }
            (Some(trd), None) => {
                let res = *trd;
                if res.timestamp < self.last_time {
                    panic!("History file entries are not stored in ascending order by time.")
                }
                self.last_time = res.timestamp;
                self.last_trd = self.trd_parser.next();
                Some(res)
            }
            (None, Some(prl)) => {
                let res = *prl;
                if res.timestamp < self.last_time {
                    panic!("History file entries are not stored in ascending order by time.")
                }
                self.last_time = res.timestamp;
                self.last_prl = self.prl_parser.next();
                Some(res)
            }
            (None, None) => { None }
        }
    }
}