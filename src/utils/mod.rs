pub mod map;
pub mod slice_reader;

macro_rules! log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    }
}
pub(crate) use log;