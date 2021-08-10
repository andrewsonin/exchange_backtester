use std::cmp::{min, Ordering};

pub use interface::EventProcessor;

use crate::history::{
    reader::{PRLReader, TRDReader},
    types::{HistoryEvent, HistoryEventBody},
};
use crate::input::InputInterface;
use crate::types::{Direction, OrderID, Price, Size, Timestamp};

pub mod interface;

pub struct HistoryParser<'a, ParsingInfo>
    where ParsingInfo: InputInterface
{
    prl_parser: PRLReader<'a, ParsingInfo>,
    trd_parser: TRDReader<'a, ParsingInfo>,

    last_prl: Option<(Timestamp, Size, Direction, Price, OrderID)>,
    last_trd: Option<(Timestamp, Size, Direction, OrderID)>,

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
            (Some((prl_ts, _, _, _, _)), Some((trd_ts, _, _, _))) => {
                min(prl_ts, trd_ts)
            }
            (Some((prl_ts, _, _, _, _)), _) => { prl_ts }
            (_, Some((trd_ts, _, _, _))) => { trd_ts }
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

impl<T: InputInterface> EventProcessor for HistoryParser<'_, T>
{
    fn yield_next_event(&mut self) -> Option<HistoryEvent>
    {
        match (&self.last_trd, &self.last_prl) {
            (
                Some((trd_ts, trd_size, trd_dir, trd_id)),
                Some((prl_ts, prl_size, prl_dir, prl_price, prl_id))
            ) => {
                match (prl_ts.cmp(trd_ts), trd_id > prl_id) {
                    (Ordering::Less, _) | (Ordering::Equal, true) => {
                        let res = HistoryEvent {
                            timestamp: *prl_ts,
                            event: HistoryEventBody::PRL((*prl_size, *prl_dir, *prl_price, *prl_id)),
                        };
                        if res.timestamp < self.last_time {
                            panic!("History file entries are not stored in ascending order by time.")
                        }
                        self.last_time = res.timestamp;
                        self.last_prl = self.prl_parser.next();
                        Some(res)
                    }
                    (Ordering::Greater, _) | (Ordering::Equal, false) => {
                        let res = HistoryEvent { timestamp: *trd_ts, event: HistoryEventBody::TRD((*trd_size, *trd_dir)) };
                        if res.timestamp < self.last_time {
                            panic!("History file entries are not stored in ascending order by time.")
                        }
                        self.last_time = res.timestamp;
                        self.last_trd = self.trd_parser.next();
                        Some(res)
                    }
                }
            }
            (Some((trd_ts, trd_size, trd_dir, _)), None) => {
                let res = HistoryEvent { timestamp: *trd_ts, event: HistoryEventBody::TRD((*trd_size, *trd_dir)) };
                if res.timestamp < self.last_time {
                    panic!("History file entries are not stored in ascending order by time.")
                }
                self.last_time = res.timestamp;
                self.last_trd = self.trd_parser.next();
                Some(res)
            }
            (None, Some((prl_ts, prl_size, prl_dir, prl_price, prl_id))) => {
                let res = HistoryEvent {
                    timestamp: *prl_ts,
                    event: HistoryEventBody::PRL((*prl_size, *prl_dir, *prl_price, *prl_id)),
                };
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