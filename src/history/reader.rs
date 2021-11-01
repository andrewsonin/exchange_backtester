use std::collections::VecDeque;
use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader};

use csv::ReaderBuilder;

use crate::history::types::{HistoryEvent, OBDiffHistoryColumnIndexInfo, TradeHistoryColumnIndexInfo};
use crate::input::InputInterface;
use crate::types::{DateTime, Direction, OrderID, Price, Size};
use crate::utils::ExpectWith;

pub(crate)
struct OBDiffHistoryReader<'a, ParsingInfo: InputInterface>
{
    files_to_parse: VecDeque<String>,
    buffered_entries: VecDeque<(DateTime, Size, Direction, Price, OrderID)>,
    args: &'a ParsingInfo,
}

impl<ParsingInfo: InputInterface> OBDiffHistoryReader<'_, ParsingInfo>
{
    pub(crate)
    fn new<'a>(files_to_parse: &str, args: &'a ParsingInfo) -> OBDiffHistoryReader<'a, ParsingInfo>
    {
        let files_to_parse: VecDeque<String> = {
            let file = File::open(files_to_parse).expect_with(
                || format!("Cannot read the following file: {}", files_to_parse)
            );
            BufReader::new(&file).lines().filter_map(|line| line.ok()).collect()
        };
        let mut res = OBDiffHistoryReader {
            files_to_parse,
            buffered_entries: VecDeque::new(),
            args,
        };
        res.buffer_next_file().expect("No history files provided");
        res
    }

    pub(crate)
    fn next(&mut self) -> Option<(DateTime, Size, Direction, Price, OrderID)>
    {
        match self.buffered_entries.pop_front() {
            None => loop {
                match self.buffer_next_file() {
                    Ok(_) => {
                        let res = self.buffered_entries.pop_front();
                        if res.is_some() {
                            return res;
                        }
                        // Continue loop in case when CSV-file has 0 entries
                    }
                    Err(_) => { return None; }
                }
            }
            res => { res }
        }
    }

    pub(crate)
    fn buffer_next_file(&mut self) -> Result<(), ()>
    {
        let file_to_read = match self.files_to_parse.pop_front() {
            Some(file_to_read) => { file_to_read }
            None => { return Err(()); }
        };
        let cur_file_content = read_to_string(&file_to_read).expect_with(
            || format!("Cannot read the following file: {}", file_to_read)
        );
        let col_idx_info = OBDiffHistoryColumnIndexInfo::new_for_csv(&file_to_read, self.args);
        let price_step = self.args.get_price_step();
        let datetime_format = self.args.get_datetime_format();
        self.buffered_entries.extend(
            ReaderBuilder::new()
                .delimiter(self.args.get_csv_sep() as u8)
                .from_reader(cur_file_content.as_bytes())
                .records()
                .zip(2..)
                .map(
                    |(record, row)|
                        HistoryEvent::parse_ob_diff(
                            record.expect_with(
                                || format!("Cannot parse {}-th CSV-record for the file: {}", row, file_to_read)
                            ),
                            &col_idx_info,
                            price_step,
                            datetime_format,
                        )
                )
        );
        Ok(())
    }
}

pub(crate)
struct TradeHistoryReader<'a, ParsingInfo: InputInterface>
{
    files_to_parse: VecDeque<String>,
    buffered_entries: VecDeque<(DateTime, Size, Direction, OrderID)>,
    args: &'a ParsingInfo,
}

impl<ParsingInfo: InputInterface> TradeHistoryReader<'_, ParsingInfo>
{
    pub(crate)
    fn new<'a>(files_to_parse: &str, args: &'a ParsingInfo) -> TradeHistoryReader<'a, ParsingInfo>
    {
        let files_to_parse: VecDeque<String> = {
            let file = File::open(files_to_parse).expect_with(
                || format!("Cannot read the following file: {}", files_to_parse)
            );
            BufReader::new(&file).lines().filter_map(|line| line.ok()).collect()
        };
        let mut res = TradeHistoryReader {
            files_to_parse,
            buffered_entries: VecDeque::new(),
            args,
        };
        res.buffer_next_file().expect("No history files provided");
        res
    }

    pub(crate)
    fn next(&mut self) -> Option<(DateTime, Size, Direction, OrderID)>
    {
        match self.buffered_entries.pop_front() {
            None => loop {
                match self.buffer_next_file() {
                    Ok(_) => {
                        let res = self.buffered_entries.pop_front();
                        if res.is_some() {
                            return res;
                        }
                        // Continue loop in case when CSV-file has 0 entries
                    }
                    Err(_) => { return None; }
                }
            }
            res => { res }
        }
    }

    pub(crate)
    fn buffer_next_file(&mut self) -> Result<(), ()>
    {
        let file_to_read = match self.files_to_parse.pop_front() {
            Some(file_to_read) => { file_to_read }
            None => { return Err(()); }
        };
        let cur_file_content = read_to_string(&file_to_read).expect_with(
            || format!("Cannot read the following file: {}", file_to_read)
        );
        let col_idx_info = TradeHistoryColumnIndexInfo::new_for_csv(&file_to_read, self.args);
        let datetime_format = self.args.get_datetime_format();
        self.buffered_entries.extend(
            ReaderBuilder::new()
                .delimiter(self.args.get_csv_sep() as u8)
                .from_reader(cur_file_content.as_bytes())
                .records()
                .zip(2..)
                .map(
                    |(record, row)|
                        HistoryEvent::parser_trade(
                            record.expect_with(
                                || format!("Cannot parse {}-th CSV-record for the file: {}", row, file_to_read)
                            ),
                            &col_idx_info,
                            datetime_format,
                        )
                )
        );
        Ok(())
    }
}