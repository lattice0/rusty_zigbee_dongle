pub mod map;
pub mod slice_reader;

macro_rules! log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    }
}
pub(crate) use log;

macro_rules! warnn {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    }
}
pub(crate) use warnn;