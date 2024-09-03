use core::future::Future;
use core::task::{Context, Poll};
use std::io::Read;
use std::pin::pin;
use std::sync::Arc;
use std::task::Wake;

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
struct NoOpWaker;


pub struct SliceReader<'a>(pub &'a [u8]);

#[allow(unused)]
impl<'a> SliceReader<'a> {
    pub fn read_u8(&mut self) -> Result<u8, std::io::Error> {
        let mut buffer = [0u8; 1];
        self.0.read_exact(&mut buffer[..])?;
        Ok(buffer[0])
    }

    pub fn read_u16_be(&mut self) -> Result<u16, std::io::Error> {
        let mut buffer = [0u8; 2];
        self.0.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    pub fn read_u16_le(&mut self) -> Result<u16, std::io::Error> {
        let mut buffer = [0u8; 2];
        self.0.read_exact(&mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    pub fn read_exact(&mut self, output: &mut [u8]) -> Result<(), std::io::Error> {
        self.0.read_exact(output)
    }

    pub fn subslice_exact(&mut self, len: usize) -> Result<&'a [u8], std::io::Error> {
        let (left, right) = self.0.split_at_checked(len).ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "split at error",
        ))?;
        self.0 = right;
        Ok(left)
    }
}

impl Wake for NoOpWaker {
    fn wake(self: Arc<Self>) {}
}

// no dependency blocking function for debugging only
pub fn debug_only_block_on<F: Future>(mut future: F) -> F::Output {
    let waker = Arc::new(NoOpWaker).into();

    let mut context = Context::from_waker(&waker);

    let mut future = pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(val) => return val,
            Poll::Pending => {}
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
