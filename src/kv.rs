use std::collections::HashMap;

/// kv store
pub struct KvStore {
    db: HashMap<String, String>,
}

/// A k-v store based on memory
impl KvStore {
    /// create memory kv store
    pub fn new() -> Self {
        KvStore { db: HashMap::new() }
    }

    /// set k-v to memory
    pub fn set(&mut self, key: String, value: String) {
        println!("store.set: {}, {}", key, value);
        self.db.insert(key, value);
    }

    /// 
    pub fn get(&self, key: String) -> Option<String> {
        self.db.get(&key).map(|value| value.to_owned())
        // match self.db.get(&key) {
        //     Some(value) => Some(value.to_owned()),
        //     None => None,
        // };
    }

    /// 
    pub fn remove(&mut self, key: String) -> Option<String> {
        self.db.remove(&key).map(|value| value)
        // match self.db.remove(&key) {
        //     Some(value) => Some(value),
        //     None => None,
        // }
    }
}
