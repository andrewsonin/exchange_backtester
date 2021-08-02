use std::str::FromStr;

use chrono::NaiveDateTime;
use csv::{ReaderBuilder, StringRecord};

use crate::cli::InputInterface;
use crate::order::{Order, OrderInfo, PricedOrder};
use crate::types::{OrderDirection, OrderID, OrderSize, Price};
use crate::utils::ExpectWith;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum OrderOrigin {
    History,
    Trader,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub(crate) struct HistoryEvent {
    pub(crate) tick_type: HistoryTickType,
    pub(crate) price: Price,
    pub(crate) order_info: OrderInfo,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub(crate) enum HistoryTickType {
    TRD,
    PRL,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) struct HistoryEventWithTime
{
    pub(crate) timestamp: NaiveDateTime,
    pub(crate) event: HistoryEvent,
}

pub(crate) struct HistoryColumnIndexInfo
{
    price_idx: usize,
    size_idx: usize,
    timestamp_idx: usize,
    buy_sell_flag_idx: usize,
    order_id_idx: usize,
}

impl const Order for HistoryEvent {
    fn get_order_id(&self) -> OrderID { self.order_info.order_id }
    fn get_order_size(&self) -> OrderSize { self.order_info.size }
    fn mut_order_size(&mut self) -> &mut OrderSize { &mut self.order_info.size }
    fn get_order_direction(&self) -> OrderDirection { self.order_info.direction }
    fn extract_body(self) -> OrderInfo { self.order_info }
}

impl const PricedOrder for HistoryEvent {
    fn get_price(&self) -> Price { self.price }
}

impl HistoryEventWithTime
{
    pub(crate) fn parse(record: StringRecord,
                        col_idx_info: &HistoryColumnIndexInfo,
                        price_step: f64,
                        dt_format: &str,
                        tick_type: HistoryTickType) -> HistoryEventWithTime
    {
        let timestamp = &record[col_idx_info.timestamp_idx];
        let order_id = &record[col_idx_info.order_id_idx];
        let price = &record[col_idx_info.price_idx];
        let size = &record[col_idx_info.size_idx];
        let bs_flag = &record[col_idx_info.buy_sell_flag_idx];
        HistoryEventWithTime {
            timestamp: NaiveDateTime::parse_from_str(timestamp, dt_format).expect_with(
                || format!("Cannot parse to NaiveDateTime: {}. Datetime format used: {}", timestamp, dt_format)
            ),
            event: HistoryEvent {
                price: Price::from_decimal_str(price, price_step),
                tick_type,
                order_info: OrderInfo {
                    order_id: OrderID(
                        u64::from_str(order_id).expect_with(
                            || format!("Cannot parse to u64: {}", order_id)
                        )
                    ),
                    size: OrderSize(
                        u64::from_str(size).expect_with(
                            || format!("Cannot parse to u64: {}", size)
                        )
                    ),
                    direction: match bs_flag {
                        "0" | "B" | "b" | "False" | "false" => { OrderDirection::Buy }
                        "1" | "S" | "s" | "True" | "true" => { OrderDirection::Sell }
                        _ => { panic!("Cannot parse buy-sell flag: {}", bs_flag) }
                    },
                },
            },
        }
    }
}

impl HistoryColumnIndexInfo
{
    pub(crate)
    fn new_for_csv<ParsingInfo>(path: &str, args: &ParsingInfo) -> HistoryColumnIndexInfo
        where ParsingInfo: InputInterface
    {
        let mut order_id_idx: Option<usize> = None;
        let mut timestamp_idx: Option<usize> = None;
        let mut size_idx: Option<usize> = None;
        let mut price_idx: Option<usize> = None;
        let mut buy_sell_flag_idx: Option<usize> = None;

        let order_id_colname = args.get_order_id_colname();
        let timestamp_colname = args.get_order_timestamp_colname();
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
            } else if header == timestamp_colname {
                if let Some(_) = timestamp_idx {
                    panic!("Duplicate column {} in the file: {}", timestamp_colname, path)
                }
                timestamp_idx = Some(i)
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
        HistoryColumnIndexInfo {
            price_idx: price_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", price_colname, path)
            ),
            size_idx: size_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", size_colname, path)
            ),
            timestamp_idx: timestamp_idx.expect_with(
                || format!("Cannot find {} column in the CSV-file: {}", timestamp_colname, path)
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
