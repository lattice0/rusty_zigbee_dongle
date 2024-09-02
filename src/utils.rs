use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

#[derive(Debug, PartialEq)]
pub struct Map<K: PartialEq + 'static, V: PartialEq + 'static>(&'static [(K, V)]);

impl<K: PartialEq + 'static, V: PartialEq + 'static> Map<K, V> {
    pub const fn new(map: &'static [(K, V)]) -> Self {
        Self(map)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.0.iter().any(|(k, _)| k == key)
    }

    pub fn contains_value(&self, value: &V) -> bool {
        self.0.iter().any(|(_, v)| v == value)
    }
}

macro_rules! log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    }
}
pub(crate) use log;

// Create a no-op waker
fn dummy_raw_waker() -> RawWaker {
    fn no_op_clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    fn no_op(_: *const ()) {}

    RawWaker::new(
        core::ptr::null(),
        &RawWakerVTable::new(no_op_clone, no_op, no_op, no_op),
    )
}

// no dependency blocking function for debugging only
pub fn debug_only_unsafe_block_on<F: Future>(mut future: F) -> F::Output {
    let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
    let mut context = Context::from_waker(&waker);

    // Pin the future on the stack
    let mut future = unsafe { Pin::new_unchecked(&mut future) };

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(val) => return val,
            Poll::Pending => {
                // In a real system, you might yield to the executor or check other tasks,
                // but in this simple case, we just keep polling.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_get() {
        let map = Map(&[("key1", 1), ("key2", 2), ("key3", 3)]);

        assert_eq!(map.get(&"key1"), Some(&1));
        assert_eq!(map.get(&"key2"), Some(&2));
        assert_eq!(map.get(&"key3"), Some(&3));
        assert_eq!(map.get(&"key4"), None);
    }

    #[test]
    fn test_map_contains_key() {
        let map = Map(&[("key1", 1), ("key2", 2), ("key3", 3)]);

        assert!(map.contains_key(&"key1"));
        assert!(map.contains_key(&"key2"));
        assert!(map.contains_key(&"key3"));
        assert!(!map.contains_key(&"key4"));
    }

    #[test]
    fn test_map_contains_value() {
        let map = Map(&[("key1", 1), ("key2", 2), ("key3", 3)]);

        assert!(map.contains_value(&1));
        assert!(map.contains_value(&2));
        assert!(map.contains_value(&3));
        assert!(!map.contains_value(&4));
    }
}
