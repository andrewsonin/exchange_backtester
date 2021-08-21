use std::cmp::{min, Ordering};

pub use interface::EventProcessor;

use crate::history::{
    reader::{PRLReader, TRDReader},
    types::{HistoryEvent, HistoryEventBody},
};
use crate::input::InputInterface;
use crate::types::{DateTime, Direction, OrderID, Price, Size};

pub mod interface;

pub struct HistoryParser<'a, ParsingInfo>
    where ParsingInfo: InputInterface
{
    prl_parser: PRLReader<'a, ParsingInfo>,
    trd_parser: TRDReader<'a, ParsingInfo>,

    last_prl: Option<(DateTime, Size, Direction, Price, OrderID)>,
    last_trd: Option<(DateTime, Size, Direction, OrderID)>,

    last_dt: DateTime,
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
            (Some(prl), Some(trd)) => { min(prl.0, trd.0) }
            (Some(prl), _) => { prl.0 }
            (_, Some(trd)) => { trd.0 }
            _ => { unreachable!() }
        };
        HistoryParser {
            prl_parser,
            trd_parser,
            last_prl,
            last_trd,
            last_dt: last_time,
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
                            datetime: *prl_ts,
                            event: HistoryEventBody::PRL(*prl_size, *prl_dir, *prl_price, *prl_id),
                        };
                        if res.datetime < self.last_dt {
                            panic!("History file entries are not stored in ascending order by time")
                        }
                        self.last_dt = res.datetime;
                        self.last_prl = self.prl_parser.next();
                        Some(res)
                    }
                    (Ordering::Greater, _) | (Ordering::Equal, false) => {
                        let res = HistoryEvent { datetime: *trd_ts, event: HistoryEventBody::TRD(*trd_size, *trd_dir) };
                        if res.datetime < self.last_dt {
                            panic!("History file entries are not stored in ascending order by time")
                        }
                        self.last_dt = res.datetime;
                        self.last_trd = self.trd_parser.next();
                        Some(res)
                    }
                }
            }
            (Some((trd_ts, trd_size, trd_dir, _)), None) => {
                let res = HistoryEvent { datetime: *trd_ts, event: HistoryEventBody::TRD(*trd_size, *trd_dir) };
                if res.datetime < self.last_dt {
                    panic!("History file entries are not stored in ascending order by time")
                }
                self.last_dt = res.datetime;
                self.last_trd = self.trd_parser.next();
                Some(res)
            }
            (None, Some((prl_ts, prl_size, prl_dir, prl_price, prl_id))) => {
                let res = HistoryEvent {
                    datetime: *prl_ts,
                    event: HistoryEventBody::PRL(*prl_size, *prl_dir, *prl_price, *prl_id),
                };
                if res.datetime < self.last_dt {
                    panic!("History file entries are not stored in ascending order by time")
                }
                self.last_dt = res.datetime;
                self.last_prl = self.prl_parser.next();
                Some(res)
            }
            (None, None) => { None }
        }
    }
}