#[derive(Debug, Clone, Copy)]
pub struct BestMapNonEmpty<K: Ord, V> {
    key: K,
    value: V,
    len: usize,
}

impl<K: Ord, V> BestMapNonEmpty<K, V> {
    pub fn new(key: K, value: V) -> Self {
        BestMapNonEmpty {
            key: key,
            value: value,
            len: 0,
        }
    }

    pub fn insert_gt(&mut self, key: K, value: V) {
        if key > self.key {
            self.key = key;
            self.value = value;
        }
        self.len += 1;
    }

    pub fn insert_ge(&mut self, key: K, value: V) {
        if key >= self.key {
            self.key = key;
            self.value = value;
        }
        self.len += 1;
    }

    pub fn get(&self) -> (&K, &V) { (&self.key, &self.value) }
    pub fn key(&self) -> &K { &self.key }
    pub fn value(&self) -> &V { &self.value }

    pub fn into(self) -> (K, V) { (self.key, self.value) }
    pub fn into_key(self) -> K { self.key }
    pub fn into_value(self) -> V { self.value }

    pub fn len(&self) -> usize { self.len }
}
