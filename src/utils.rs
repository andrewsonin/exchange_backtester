use std::panic::panic_any;

pub(crate) trait ExpectWith<T, F>
    where F: Fn() -> String {
    fn expect_with(self, f: F) -> T;
}

impl<T, F> ExpectWith<T, F> for Option<T>
    where F: Fn() -> String {
    fn expect_with(self, f: F) -> T {
        match self {
            Some(v) => { v }
            None => { panic_any(f()) }
        }
    }
}

impl<T, F, E> ExpectWith<T, F> for Result<T, E>
    where F: Fn() -> String {
    fn expect_with(self, f: F) -> T {
        match self {
            Ok(v) => { v }
            Err(_) => { panic_any(f()) }
        }
    }
}