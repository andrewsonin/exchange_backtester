use std::cmp::{min, Ordering};

pub use interface::EventProcessor;

use crate::history::{
    reader::{OBDiffHistoryReader, TradeHistoryReader},
    types::{HistoryEvent, HistoryEventBody},
};
use crate::input::InputInterface;
use crate::types::{DateTime, Direction, OrderID, Price, Size};

pub mod interface;

pub struct HistoryParser<'a, ParsingInfo>
    where ParsingInfo: InputInterface
{
    ob_diff_history_parser: OBDiffHistoryReader<'a, ParsingInfo>,
    trade_history_parser: TradeHistoryReader<'a, ParsingInfo>,

    last_ob_diff: Option<(DateTime, Size, Direction, Price, OrderID)>,
    last_trd: Option<(DateTime, Size, Direction, OrderID)>,

    last_dt: DateTime,
}

impl<ParsingInfo: InputInterface> HistoryParser<'_, ParsingInfo>
{
    pub fn new(args: &ParsingInfo) -> HistoryParser<ParsingInfo>
    {
        let mut ob_diff_history_parser = OBDiffHistoryReader::new(args.get_ob_diff_history_files(), args);
        let mut trade_history_parser = TradeHistoryReader::new(args.get_trade_history_files(), args);
        let last_ob_diff = ob_diff_history_parser.next();
        let last_trd = trade_history_parser.next();
        let last_dt = match (last_ob_diff, last_trd) {
            (Some(ob_diff), Some(trd)) => { min(ob_diff.0, trd.0) }
            (Some(ob_diff), _) => { ob_diff.0 }
            (_, Some(trd)) => { trd.0 }
            _ => { unreachable!() }
        };
        HistoryParser {
            ob_diff_history_parser,
            trade_history_parser,
            last_ob_diff,
            last_trd,
            last_dt,
        }
    }
}

impl<T: InputInterface> EventProcessor for HistoryParser<'_, T>
{
    fn yield_next_event(&mut self) -> Option<HistoryEvent>
    {
        match (&self.last_trd, &self.last_ob_diff) {
            (
                Some((trd_dt, trd_size, trd_dir, trd_id)),
                Some((ob_diff_dt, ob_diff_size, ob_diff_dir, ob_diff_price, ob_diff_id))
            ) => {
                match (ob_diff_dt.cmp(trd_dt), trd_id > ob_diff_id) {
                    (Ordering::Less, _) | (Ordering::Equal, true) => {
                        let res = HistoryEvent {
                            datetime: *ob_diff_dt,
                            event: HistoryEventBody::OrderBookDiff(*ob_diff_size, *ob_diff_dir, *ob_diff_price, *ob_diff_id),
                        };
                        if res.datetime < self.last_dt {
                            panic!("History file entries are not stored in ascending order by time")
                        }
                        self.last_dt = res.datetime;
                        self.last_ob_diff = self.ob_diff_history_parser.next();
                        Some(res)
                    }
                    (Ordering::Greater, _) | (Ordering::Equal, false) => {
                        let res = HistoryEvent { datetime: *trd_dt, event: HistoryEventBody::Trade(*trd_size, *trd_dir) };
                        if res.datetime < self.last_dt {
                            panic!("History file entries are not stored in ascending order by time")
                        }
                        self.last_dt = res.datetime;
                        self.last_trd = self.trade_history_parser.next();
                        Some(res)
                    }
                }
            }
            (Some((trd_dt, trd_size, trd_dir, _)), None) => {
                let res = HistoryEvent { datetime: *trd_dt, event: HistoryEventBody::Trade(*trd_size, *trd_dir) };
                if res.datetime < self.last_dt {
                    panic!("History file entries are not stored in ascending order by time")
                }
                self.last_dt = res.datetime;
                self.last_trd = self.trade_history_parser.next();
                Some(res)
            }
            (None, Some((ob_diff_dt, ob_diff_size, ob_diff_dir, ob_diff_price, ob_diff_id))) => {
                let res = HistoryEvent {
                    datetime: *ob_diff_dt,
                    event: HistoryEventBody::OrderBookDiff(*ob_diff_size, *ob_diff_dir, *ob_diff_price, *ob_diff_id),
                };
                if res.datetime < self.last_dt {
                    panic!("History file entries are not stored in ascending order by time")
                }
                self.last_dt = res.datetime;
                self.last_ob_diff = self.ob_diff_history_parser.next();
                Some(res)
            }
            (None, None) => { None }
        }
    }
}