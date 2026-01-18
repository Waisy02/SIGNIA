//! In-memory KV backend.

use std::collections::BTreeMap;

use anyhow::Result;

use super::KvStore;

#[derive(Default)]
pub struct MemoryKv {
    map: BTreeMap<String, Vec<u8>>,
}

impl KvStore for MemoryKv {
    fn put(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.map.insert(key.to_string(), value);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.map.get(key).cloned())
    }

    fn delete(&mut self, key: &str) -> Result<()> {
        self.map.remove(key);
        Ok(())
    }

    fn list_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        Ok(self.map.keys().filter(|k| k.starts_with(prefix)).cloned().collect())
    }
}
