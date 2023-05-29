use std::{
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
};

use crate::error::{KvsError, Result};

/// kv store
pub struct KvStore {
    db: HashMap<String, String>,
}

/// A k-v store based on memory
impl KvStore {
    /// create memory kv store
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        todo!();
    }

    /// set k-v to memory
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        println!("store.set: {}, {}", key, value);
        self.db.insert(key, value);
        todo!();
    }

    ///
    pub fn get(&self, key: String) -> Result<Option<String>> {
        self.db.get(&key).map(|value| value.to_owned());
        // match self.db.get(&key) {
        //     Some(value) => Some(value.to_owned()),
        //     None => None,
        // };
        todo!();
    }

    ///
    pub fn remove(&mut self, key: String) -> Result<Option<String>> {
        self.db.remove(&key).map(|value| value);
        // match self.db.remove(&key) {
        //     Some(value) => Some(value),
        //     None => None,
        // }
        todo!();
    }
}
