use std::str::FromStr;

use csv::{ReaderBuilder, StringRecord};

use crate::input::InputInterface;
use crate::types::{DateTime, Direction, OrderID, Price, Size};
use crate::utils::ExpectWith;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum OrderOrigin {
    History,
    Trader,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum HistoryEventBody {
    TRD(Size, Direction),
    PRL(Size, Direction, Price, OrderID),
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct HistoryEvent
{
    pub datetime: DateTime,
    pub event: HistoryEventBody,
}

pub(crate) struct PRLColumnIndexInfo
{
    price_idx: usize,
    size_idx: usize,
    datetime_idx: usize,
    buy_sell_flag_idx: usize,
    order_id_idx: usize,
}

pub(crate) struct TRDColumnIndexInfo
{
    size_idx: usize,
    datetime_idx: usize,
    buy_sell_flag_idx: usize,
    order_id_idx: usize,
}

impl HistoryEvent
{
    pub(crate) fn parse_prl(record: StringRecord,
                            col_idx_info: &PRLColumnIndexInfo,
                            price_step: f64,
                            dt_format: &str) -> (DateTime, Size, Direction, Price, OrderID)
    {
        let datetime = &record[col_idx_info.datetime_idx];
        let order_id = &record[col_idx_info.order_id_idx];
        let price = &record[col_idx_info.price_idx];
        let size = &record[col_idx_info.size_idx];
        let bs_flag = &record[col_idx_info.buy_sell_flag_idx];
        (
            DateTime::parse_from_str(datetime, dt_format).expect_with(
                || format!("Cannot parse to NaiveDateTime: {}. Datetime format used: {}", datetime, dt_format)
            ),
            Size(
                u64::from_str(size).expect_with(
                    || format!("Cannot parse to u64: {}", size)
                )
            ),
            match bs_flag {
                "0" | "B" | "b" | "False" | "false" => { Direction::Buy }
                "1" | "S" | "s" | "True" | "true" => { Direction::Sell }
                _ => { panic!("Cannot parse buy-sell flag: {}", bs_flag) }
            },
            Price::from_decimal_str(price, price_step),
            OrderID(
                u64::from_str(order_id).expect_with(
                    || format!("Cannot parse to u64: {}", order_id)
                )
            )
        )
    }

    pub(crate) fn parse_trd(record: StringRecord,
                            col_idx_info: &TRDColumnIndexInfo,
                            dt_format: &str) -> (DateTime, Size, Direction, OrderID)
    {
        let datetime = &record[col_idx_info.datetime_idx];
        let order_id = &record[col_idx_info.order_id_idx];
        let size = &record[col_idx_info.size_idx];
        let bs_flag = &record[col_idx_info.buy_sell_flag_idx];
        (
            DateTime::parse_from_str(datetime, dt_format).expect_with(
                || format!("Cannot parse to NaiveDateTime: {}. Datetime format used: {}", datetime, dt_format)
            ),
            Size(
                u64::from_str(size).expect_with(
                    || format!("Cannot parse to u64: {}", size)
                )
            ),
            match bs_flag {
                "0" | "B" | "b" | "False" | "false" => { Direction::Buy }
                "1" | "S" | "s" | "True" | "true" => { Direction::Sell }
                _ => { panic!("Cannot parse buy-sell flag: {}", bs_flag) }
            },
            OrderID(
                u64::from_str(order_id).expect_with(
                    || format!("Cannot parse to u64: {}", order_id)
                )
            )
        )
    }
}

impl PRLColumnIndexInfo
{
    pub(crate)
    fn new_for_csv<ParsingInfo>(path: &str, args: &ParsingInfo) -> PRLColumnIndexInfo
        where ParsingInfo: InputInterface
    {
        let mut order_id_idx: Option<usize> = None;
        let mut datetime_idx: Option<usize> = None;
        let mut size_idx: Option<usize> = None;
        let mut price_idx: Option<usize> = None;
        let mut buy_sell_flag_idx: Option<usize> = None;

        let order_id_colname = args.get_order_id_colname();
        let datetime_colname = args.get_order_datetime_colname();
        let size_colname = args.get_order_size_colname();
        let price_colname = args.get_order_price_colname();
        let bs_flag_colname = args.get_order_bs_flag_colname();

        for (i, header) in ReaderBuilder::new()
            .delimiter(args.get_csv_sep() as u8)
            .from_path(path)
            .expect_with(|| format!("Cannot read the following file: {}", path))
            .headers()
            .expect_with(|| format!("Cannot parse header of the CSV-file: {}", path))
            .iter()
            .enumerate()
        {
            if header == order_id_colname {
                if let Some(_) = order_id_idx {
                    panic!("Duplicate column {} in the file: {}", order_id_colname, path)
                }
                order_id_idx = Some(i)
            } else if header == datetime_colname {
                if let Some(_) = datetime_idx {
                    panic!("Duplicate column {} in the file: {}", datetime_colname, path)
                }
                datetime_idx = Some(i)
            } else if header == size_colname {
                if let Some(_) = size_idx {
                    panic!("Duplicate column {} in the file: {}", size_colname, path)
                }
                size_idx = Some(i)
            } else if header == price_colname {
                if let Some(_) = price_idx {
                    panic!("Duplicate column {} in the file: {}", price_colname, path)
                }
                price_idx = Some(i)
            } else if header == bs_flag_colname {
                if let Some(_) = buy_sell_flag_idx {
                    panic!("Duplicate column {} in the file: {}", bs_flag_colname, path)
                }
                buy_sell_flag_idx = Some(i)
            }
        };
        PRLColumnIndexInfo {
            price_idx: price_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", price_colname, path)
            ),
            size_idx: size_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", size_colname, path)
            ),
            datetime_idx: datetime_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", datetime_colname, path)
            ),
            buy_sell_flag_idx: buy_sell_flag_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", bs_flag_colname, path)
            ),
            order_id_idx: order_id_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", order_id_colname, path)
            ),
        }
    }
}

impl TRDColumnIndexInfo
{
    pub(crate)
    fn new_for_csv<ParsingInfo>(path: &str, args: &ParsingInfo) -> TRDColumnIndexInfo
        where ParsingInfo: InputInterface
    {
        let mut order_id_idx: Option<usize> = None;
        let mut datetime_idx: Option<usize> = None;
        let mut size_idx: Option<usize> = None;
        let mut buy_sell_flag_idx: Option<usize> = None;

        let order_id_colname = args.get_order_id_colname();
        let datetime_colname = args.get_order_datetime_colname();
        let size_colname = args.get_order_size_colname();
        let bs_flag_colname = args.get_order_bs_flag_colname();

        for (i, header) in ReaderBuilder::new()
            .delimiter(args.get_csv_sep() as u8)
            .from_path(path)
            .expect_with(|| format!("Cannot read the following file: {}", path))
            .headers()
            .expect_with(|| format!("Cannot parse header of the CSV-file: {}", path))
            .iter()
            .enumerate()
        {
            if header == order_id_colname {
                if let Some(_) = order_id_idx {
                    panic!("Duplicate column {} in the file: {}", order_id_colname, path)
                }
                order_id_idx = Some(i)
            } else if header == datetime_colname {
                if let Some(_) = datetime_idx {
                    panic!("Duplicate column {} in the file: {}", datetime_colname, path)
                }
                datetime_idx = Some(i)
            } else if header == size_colname {
                if let Some(_) = size_idx {
                    panic!("Duplicate column {} in the file: {}", size_colname, path)
                }
                size_idx = Some(i)
            } else if header == bs_flag_colname {
                if let Some(_) = buy_sell_flag_idx {
                    panic!("Duplicate column {} in the file: {}", bs_flag_colname, path)
                }
                buy_sell_flag_idx = Some(i)
            }
        };
        TRDColumnIndexInfo {
            size_idx: size_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", size_colname, path)
            ),
            datetime_idx: datetime_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", datetime_colname, path)
            ),
            buy_sell_flag_idx: buy_sell_flag_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", bs_flag_colname, path)
            ),
            order_id_idx: order_id_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", order_id_colname, path)
            ),
        }
    }
}