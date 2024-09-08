pub mod map;
pub mod slice_reader;

macro_rules! log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    }
}
use futures::executor::block_on;
pub(crate) use log;

#[allow(unused_macros)]
macro_rules! warnn {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    }
}
#[allow(unused_imports)]
pub(crate) use warnn;

macro_rules! err {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    }
}
pub(crate) use err;