pub use clap::{AppSettings, Parser};

use crate::input::{default::*, InputInterface};

/// Exchange backtesting framework
#[derive(Parser)]
#[clap(version = "0.0.1", author = "Andrew Sonin <sonin.cel@yandex.ru>")]
pub struct ArgumentParser {
    /// Sets the file each line of which should contain absolute paths to the PRL csv-files to use
    #[clap(short = 'o', long = "--obd", required = true)]
    ob_diff_history_files: String,
    /// Sets the file each line of which should contain absolute paths to the TRD csv-files to use
    #[clap(short = 't', long = "--trd", required = true)]
    trade_history_files: String,
    /// Sets the name of the datetime columns in the input csv files
    #[clap(long = "--dt-colname", default_value = ORDER_DATETIME_COLNAME)]
    order_datetime_colname: String,
    /// Sets the name of the order ID columns in the input csv files
    #[clap(long = "--id-colname", default_value = ORDER_ID_COLNAME)]
    order_id_colname: String,
    /// Sets the name of the order price columns in the input csv files
    #[clap(long = "--price-colname", default_value = ORDER_PRICE_COLNAME)]
    order_price_colname: String,
    /// Sets the name of the order size columns in the input csv files
    #[clap(long = "--size-colname", default_value = ORDER_SIZE_COLNAME)]
    order_size_colname: String,
    /// Sets the name of the order buy-sell flag columns in the input csv files
    #[clap(long = "--bs-flag-colname", default_value = ORDER_BS_FLAG_COLNAME)]
    order_bs_flag_colname: String,
    /// Sets the datetime format to parse timestamp columns
    #[clap(short, long, default_value = DATETIME_FORMAT)]
    datetime_format: String,
    /// CSV-file separator
    #[clap(long, default_value = CSV_SEP)]
    csv_sep: char,
    /// Price step
    #[clap(long, default_value = PRICE_STEP)]
    price_step: f64,
}

impl InputInterface for ArgumentParser {
    fn get_ob_diff_history_files(&self) -> &str { self.ob_diff_history_files.as_str() }
    fn get_trade_history_files(&self) -> &str { self.trade_history_files.as_str() }
    fn get_order_datetime_colname(&self) -> &str { self.order_datetime_colname.as_str() }
    fn get_order_id_colname(&self) -> &str { self.order_id_colname.as_str() }
    fn get_order_price_colname(&self) -> &str { self.order_price_colname.as_str() }
    fn get_order_size_colname(&self) -> &str { self.order_size_colname.as_str() }
    fn get_order_bs_flag_colname(&self) -> &str { self.order_bs_flag_colname.as_str() }
    fn get_datetime_format(&self) -> &str { self.datetime_format.as_str() }
    fn get_csv_sep(&self) -> char { self.csv_sep }
    fn get_price_step(&self) -> f64 { self.price_step }
}