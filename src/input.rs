pub mod cli;
pub mod inline;
pub mod default;

pub trait InputInterface {
    fn get_ob_diff_history_files(&self) -> &str;
    fn get_trade_history_files(&self) -> &str;
    fn get_order_datetime_colname(&self) -> &str;
    fn get_order_id_colname(&self) -> &str;
    fn get_order_price_colname(&self) -> &str;
    fn get_order_size_colname(&self) -> &str;
    fn get_order_bs_flag_colname(&self) -> &str;
    fn get_datetime_format(&self) -> &str;
    fn get_csv_sep(&self) -> char;
    fn get_price_step(&self) -> f64;
}