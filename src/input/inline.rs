use crate::input::{default::*, InputInterface};

pub struct StaticInput {
    prl_files: String,
    trd_files: String,
    order_datetime_colname: String,
    order_id_colname: String,
    order_price_colname: String,
    order_size_colname: String,
    order_bs_flag_colname: String,
    datetime_format: String,
    csv_sep: char,
    price_step: f64,
}

impl Default for StaticInput {
    fn default() -> Self {
        StaticInput {
            prl_files: String::new(),
            trd_files: String::new(),
            order_datetime_colname: ORDER_DATETIME_COLNAME.to_string(),
            order_id_colname: ORDER_ID_COLNAME.to_string(),
            order_price_colname: ORDER_PRICE_COLNAME.to_string(),
            order_size_colname: ORDER_SIZE_COLNAME.to_string(),
            order_bs_flag_colname: ORDER_BS_FLAG_COLNAME.to_string(),
            datetime_format: DATETIME_FORMAT.to_string(),
            csv_sep: CSV_SEP.parse().unwrap(),
            price_step: PRICE_STEP.parse().unwrap(),
        }
    }
}

impl StaticInput {
    pub fn new() -> Self { Default::default() }

    pub fn with_prl_files(mut self, prl_files: &str) -> Self {
        self.prl_files = prl_files.to_string();
        self
    }
    pub fn with_trd_files(mut self, trd_files: &str) -> Self {
        self.trd_files = trd_files.to_string();
        self
    }
    pub fn with_dt_colname(mut self, order_datetime_colname: &str) -> Self {
        self.order_datetime_colname = order_datetime_colname.to_string();
        self
    }
    pub fn with_id_colname(mut self, order_id_colname: &str) -> Self {
        self.order_id_colname = order_id_colname.to_string();
        self
    }
    pub fn with_price_colname(mut self, order_price_colname: &str) -> Self {
        self.order_price_colname = order_price_colname.to_string();
        self
    }
    pub fn with_size_colname(mut self, order_size_colname: &str) -> Self {
        self.order_size_colname = order_size_colname.to_string();
        self
    }
    pub fn with_bs_flag_colname(mut self, order_bs_flag_colname: &str) -> Self {
        self.order_bs_flag_colname = order_bs_flag_colname.to_string();
        self
    }
    pub fn with_datetime_format(mut self, datetime_format: &str) -> Self {
        self.datetime_format = datetime_format.to_string();
        self
    }
    pub const fn with_csv_sep(mut self, csv_sep: char) -> Self {
        self.csv_sep = csv_sep;
        self
    }
    pub const fn with_price_step(mut self, price_step: f64) -> Self {
        self.price_step = price_step;
        self
    }
}

impl InputInterface for StaticInput {
    fn get_prl_files(&self) -> &str {
        if self.prl_files.is_empty() {
            panic!("get_prl_files returned an empty string. Consider setting PRL files with the method 'with_prl_files' before usage")
        }
        self.prl_files.as_str()
    }
    fn get_trd_files(&self) -> &str {
        if self.trd_files.is_empty() {
            panic!("get_trd_files returned an empty string. Consider setting TRD files with the method 'with_trd_files' before usage")
        }
        self.trd_files.as_str()
    }
    fn get_order_datetime_colname(&self) -> &str { self.order_datetime_colname.as_str() }
    fn get_order_id_colname(&self) -> &str { self.order_id_colname.as_str() }
    fn get_order_price_colname(&self) -> &str { self.order_price_colname.as_str() }
    fn get_order_size_colname(&self) -> &str { self.order_size_colname.as_str() }
    fn get_order_bs_flag_colname(&self) -> &str { self.order_bs_flag_colname.as_str() }
    fn get_datetime_format(&self) -> &str { self.datetime_format.as_str() }
    fn get_csv_sep(&self) -> char { self.csv_sep }
    fn get_price_step(&self) -> f64 { self.price_step }
}