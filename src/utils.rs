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
