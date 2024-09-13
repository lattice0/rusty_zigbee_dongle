pub mod map;
pub mod slice_reader;
pub mod sleep;

#[cfg(not(feature="log"))]
mod log {
    macro_rules! info {
        ($($arg:tt)*) => {
            println!($($arg)*);
        }
    }

    #[allow(unused_macros)]
    macro_rules! warn {
        ($($arg:tt)*) => {
            eprintln!($($arg)*);
        }
    }

    macro_rules! error {
        ($($arg:tt)*) => {
            eprintln!($($arg)*);
        }
    }

    macro_rules! trace {
        ($($arg:tt)*) => {
            eprintln!($($arg)*);
        }
    }
}

#[cfg(feature="log")]
pub mod log {
    pub use log::info;
    pub use log::warn;
    pub use log::error;
    pub use log::trace;
}

#[allow(unused_imports)]
pub(crate) use log::info;
#[allow(unused_imports)]
pub(crate) use log::warn;
#[allow(unused_imports)]
pub(crate) use log::error;
#[allow(unused_imports)]
pub(crate) use log::trace;