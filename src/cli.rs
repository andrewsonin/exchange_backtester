pub use clap::{AppSettings, Clap};

/// Exchange backtesting framework
#[derive(Clap)]
#[clap(version = "0.0.1", author = "Andrew Sonin <sonin.cel@yandex.ru>")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct ArgumentParser {
    /// Sets the file each line of which should contain absolute paths to the PRL csv-files to use
    #[clap(short = 'p', long = "--prl", required = true)]
    pub(crate) prl_files: String,
    /// Sets the file each line of which should contain absolute paths to the TRD csv-files to use
    #[clap(short = 't', long = "--trd", required = true)]
    pub(crate) trd_files: String,
    /// Sets the name of the timestamp columns in the input csv files
    #[clap(long = "--ts-colname", default_value = "Timestamp")]
    pub(crate) order_timestamp_colname: String,
    /// Sets the name of the order ID columns in the input csv files
    #[clap(long = "--id-colname", default_value = "ORDER_ID")]
    pub(crate) order_id_colname: String,
    /// Sets the name of the order price columns in the input csv files
    #[clap(long = "--price-colname", default_value = "PRICE")]
    pub(crate) order_price_colname: String,
    /// Sets the name of the order size columns in the input csv files
    #[clap(long = "--size-colname", default_value = "SIZE")]
    pub(crate) order_size_colname: String,
    /// Sets the name of the order buy-sell flag columns in the input csv files
    #[clap(long = "--bs-flag-colname", default_value = "BUY_SELL_FLAG")]
    pub(crate) order_bs_flag_colname: String,
    /// Sets the datetime format to parse timestamp columns
    #[clap(short, long, default_value = "%Y-%m-%d %H:%M:%S%.f")]
    pub(crate) datetime_format: String,
    /// CSV-file separator
    #[clap(long, default_value = ",")]
    pub(crate) csv_sep: char,
    /// Price step
    #[clap(long, default_value = "0.0025")]
    pub(crate) price_step: f64,
}