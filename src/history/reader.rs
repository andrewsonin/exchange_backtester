use std::collections::VecDeque;
use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader};

use csv::ReaderBuilder;

use crate::cli::InputInterface;
use crate::history::types::{HistoryColumnIndexInfo, HistoryEventWithTime, HistoryTickType};
use crate::utils::ExpectWith;

pub(crate)
struct HistoryReader<'a, const TICK_TYPE: HistoryTickType, ParsingInfo>
    where ParsingInfo: InputInterface
{
    files_to_parse: VecDeque<String>,
    buffered_entries: VecDeque<HistoryEventWithTime>,
    args: &'a ParsingInfo,
}

impl<const TICK_TYPE: HistoryTickType, ParsingInfo> HistoryReader<'_, TICK_TYPE, ParsingInfo>
    where ParsingInfo: InputInterface
{
    pub(crate)
    fn new<'a>(files_to_parse: &str, args: &'a ParsingInfo) -> HistoryReader<'a, TICK_TYPE, ParsingInfo>
    {
        let files_to_parse: VecDeque<String> = {
            let file = File::open(files_to_parse).expect_with(
                || format!("Cannot read the following file: {}", files_to_parse)
            );
            BufReader::new(&file).lines().filter_map(|line| line.ok()).collect()
        };
        let mut res = HistoryReader {
            files_to_parse,
            buffered_entries: VecDeque::new(),
            args,
        };
        res.buffer_next_file().expect("No history files provided");
        res
    }

    pub(crate)
    fn next(&mut self) -> Option<HistoryEventWithTime>
    {
        match self.buffered_entries.pop_front() {
            Some(value) => { Some(value) }
            None => loop {
                match self.buffer_next_file() {
                    Ok(_) => {
                        if let Some(value) = self.buffered_entries.pop_front() {
                            return Some(value);
                        }
                        // Continue loop in case when CSV-file has 0 entries
                    }
                    Err(_) => { return None; }
                }
            }
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
        let col_idx_info = HistoryColumnIndexInfo::new_for_csv(&file_to_read, self.args);
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
                        HistoryEventWithTime::parse(
                            record.expect_with(
                                || format!("Cannot parse {}-th CSV-record for the file: {}", row, file_to_read)
                            ),
                            &col_idx_info,
                            price_step,
                            datetime_format,
                            TICK_TYPE,
                        )
                )
        );
        Ok(())
    }
}