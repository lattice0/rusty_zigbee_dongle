/// Map that owns the data and has a fixed size. Data is simply options of tuples, the simplest possible
/// representation of a map.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct StaticMap<const N: usize, K, V>(pub [Option<(K, V)>; N]);

impl<const N: usize, K: Copy + 'static, V: Copy + 'static> Default for StaticMap<N, K, V> {
    fn default() -> Self {
        Self([None; N])
    }
}

impl<
        const N: usize,
        K: PartialEq + Clone + Copy + 'static,
        V: PartialEq + Copy + Clone + 'static,
    > StaticMap<N, K, V>
{
    /// Create a new StaticMap from a slice of tuples
    // Ugly const implementation so no iterators
    pub const fn new(map: &[(K, V)]) -> Self {
        let mut m: [Option<(K, V)>; N] = [None; N];
        let mut i = 0;
        loop {
            m[i] = Some(map[i]);
            i += 1;
            if i >= map.len() {
                break;
            }
        }
        Self(m)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        Some(
            self.0
                .iter()
                .find_map(|x| match x {
                    Some((k, v)) if k == key => Some((k, v)),
                    _ => None,
                })?
                .1,
        )
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.0
            .iter()
            .any(|tuple| matches!(tuple, Some((k, _)) if k == key))
    }

    pub fn contains_value(&self, value: &V) -> bool {
        self.0
            .iter()
            .any(|tuple| matches!(tuple, Some((_, v)) if v == value))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0
            .iter()
            .filter_map(|x| x.as_ref().map(|(k, v)| (k, v)))
    }

    /// Inserts and returns the value previously associated with the key if it existed.
    /// Returns None if no empty slot was found.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, MapError> {
        if let Some(tuple) = self
            .0
            .iter_mut()
            .find(|x| matches!(x, Some((k, _)) if k == &key))
        {
            let old_v: Option<V> = tuple.map(|(_, v)| v);
            tuple.replace((key, value));
            Ok(old_v)
        } else if let Some(empty_space) = self.0.iter_mut().find(|x| x.is_none()) {
            empty_space.replace((key, value)).map(|(_, v)| v);
            Ok(None)
        } else {
            Err(MapError::Full)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MapError {
    Full,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_get() {
        let map: StaticMap<3, &str, i32> = StaticMap::new(&[("key1", 1), ("key2", 2), ("key3", 3)]);

        assert_eq!(map.get(&"key1"), Some(&1));
        assert_eq!(map.get(&"key2"), Some(&2));
        assert_eq!(map.get(&"key3"), Some(&3));
        assert_eq!(map.get(&"key4"), None);
    }

    #[test]
    fn test_map_contains_key() {
        let map: StaticMap<3, &str, i32> = StaticMap::new(&[("key1", 1), ("key2", 2), ("key3", 3)]);

        assert!(map.contains_key(&"key1"));
        assert!(map.contains_key(&"key2"));
        assert!(map.contains_key(&"key3"));
        assert!(!map.contains_key(&"key4"));
    }

    #[test]
    fn test_map_contains_value() {
        let map: StaticMap<3, &str, i32> = StaticMap::new(&[("key1", 1), ("key2", 2), ("key3", 3)]);

        assert!(map.contains_value(&1));
        assert!(map.contains_value(&2));
        assert!(map.contains_value(&3));
        assert!(!map.contains_value(&4));
    }

    #[test]
    fn test_map_insert_value() {
        let mut map: StaticMap<4, &str, i32> =
            StaticMap::new(&[("key1", 1), ("key2", 2), ("key3", 3)]);

        assert!(map.contains_value(&1));
        assert!(map.contains_value(&2));
        assert!(map.contains_value(&3));
        assert!(!map.contains_value(&4));

        map.insert("4", 4).unwrap();

        assert!(map.contains_value(&1));
        assert!(map.contains_value(&2));
        assert!(map.contains_value(&3));
        assert!(map.contains_value(&4));
    }
}
