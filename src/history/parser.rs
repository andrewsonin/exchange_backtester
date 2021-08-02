use crate::cli::ArgumentParser;
use crate::history::reader::HistoryReader;
use crate::history::types::{HistoryEventWithTime, HistoryTickType};

const PRL: HistoryTickType = HistoryTickType::PRL;
const TRD: HistoryTickType = HistoryTickType::TRD;

pub(crate) struct HistoryParser<'a>
{
    prl_parser: HistoryReader<'a, PRL>,
    trd_parser: HistoryReader<'a, TRD>,

    last_prl: Option<HistoryEventWithTime>,
    last_trd: Option<HistoryEventWithTime>,
}

impl HistoryParser<'_> {
    pub fn new(args: &ArgumentParser) -> HistoryParser
    {
        let mut prl_parser = HistoryReader::new(&args.prl_files, args);
        let mut trd_parser = HistoryReader::new(&args.trd_files, args);
        let last_prl = prl_parser.next();
        let last_trd = trd_parser.next();
        HistoryParser {
            prl_parser,
            trd_parser,
            last_prl,
            last_trd,
        }
    }

    pub(crate) fn yield_next_event(&mut self) -> Option<HistoryEventWithTime>
    {
        match (&self.last_trd, &self.last_prl) {
            (Some(trd), Some(prl)) => {
                if prl.timestamp < trd.timestamp {
                    let res = *prl;
                    self.last_prl = self.prl_parser.next();
                    Some(res)
                } else {
                    let res = *trd;
                    self.last_trd = self.trd_parser.next();
                    Some(res)
                }
            }
            (Some(trd), None) => {
                let res = *trd;
                self.last_trd = self.trd_parser.next();
                Some(res)
            }
            (None, Some(prl)) => {
                let res = *prl;
                self.last_prl = self.prl_parser.next();
                Some(res)
            }
            (None, None) => { None }
        }
    }
}