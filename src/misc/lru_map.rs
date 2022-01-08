use std::collections::HashMap;
use std::hash::Hash;

/// Hash map that implements a least recently used cache.
/// Each item in the hash map is a tuple of the key and a counter which indicates when it was last used.
/// Every time a key is accessed, the counter is set to the current global counter value, thus indicating
/// when this key was accessed for the last time. If a new item is inserted into the mapand the map has reached
/// a given size, the map is checked for the item with the lowest counter value and this item is discarded.
pub struct LruMap<T, K, const S: usize> {
    /// Actual inner map from key to value and counter tuple.
    map: HashMap<K, (T, u32)>,
    /// Current access counter value
    counter: u32,
}

impl<T, K, const S: usize> LruMap<T, K, S>
where
    K: Eq + Hash + Clone,
{
    /// Create a new LruMap
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            counter: 0,
        }
    }

    /// Gets a value from the map. If the key is not present, None is returned.
    /// Note that self has to be mutable to increase the counter of the key.
    pub fn get(&mut self, key: K) -> Option<&T> {
        let val = self.map.get_mut(&key);
        if let Some((t, counter)) = val {
            self.counter += 1;
            *counter = self.counter;
            return Some(t);
        }
        None
    }

    /// Check if the map contains a given key.
    pub fn contains(&self, key: K) -> bool {
        self.map.contains_key(&key)
    }

    /// Insert a new value into the map. If the map is full, the least recently used item is discarded.
    pub fn put(&mut self, key: K, t: T) {
        if self.map.len() == S {
            let lru_key = self.get_lru_key();
            if let Some(lru_key) = lru_key {
                self.map.remove(&lru_key);
            }
        }
        self.counter += 1;
        self.map.insert(key, (t, self.counter));
    }

    /// Clear the map.
    pub fn clear(&mut self) {
        self.map.clear();
        self.counter = 0;
    }

    /// Get the key of the least recently used item.
    fn get_lru_key(&self) -> Option<K> {
        let mut lru_key: Option<K> = None;
        let mut lru_counter = u32::MAX;
        for (k, val) in self.map.iter() {
            if val.1 < lru_counter {
                lru_key = Some(k.clone());
                lru_counter = val.1;
            }
        }
        lru_key
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_lru() {
        use super::LruMap;
        let mut list: LruMap<u32, u32, 3> = LruMap::new();

        assert!(list.get(3).is_none());
        list.put(3, 6);
        assert_eq!(*list.get(3).unwrap(), 6);

        list.put(4, 8);
        list.put(5, 10);

        assert_eq!(*list.get(4).unwrap(), 8);
        assert_eq!(*list.get(5).unwrap(), 10);

        list.put(6, 12);

        assert_eq!(*list.get(6).unwrap(), 12);

        assert!(list.get(3).is_none());

        list.put(4, 4);
        list.put(7, 14);

        assert!(list.get(5).is_none());
        assert_eq!(*list.get(4).unwrap(), 4);

        list.clear();
        assert!(list.get(4).is_none());
    }
}
