//! Content-addressed in-memory cache.

use std::collections::{BTreeMap, VecDeque};

use anyhow::{anyhow, Result};
use parking_lot::Mutex;

use crate::objects::validate_object_id;

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_items: usize,
    pub max_bytes: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self { max_items: 1024, max_bytes: 64 * 1024 * 1024 }
    }
}

pub struct ContentAddressedCache {
    cfg: CacheConfig,
    inner: Mutex<Inner>,
}

struct Inner {
    map: BTreeMap<String, Vec<u8>>,
    order: VecDeque<String>,
    bytes: usize,
}

impl ContentAddressedCache {
    pub fn new(cfg: CacheConfig) -> Self {
        Self {
            cfg,
            inner: Mutex::new(Inner { map: BTreeMap::new(), order: VecDeque::new(), bytes: 0 }),
        }
    }

    pub fn get(&self, id: &str) -> Result<Option<Vec<u8>>> {
        validate_object_id(id)?;
        Ok(self.inner.lock().map.get(id).cloned())
    }

    pub fn put(&self, id: &str, bytes: Vec<u8>) -> Result<()> {
        validate_object_id(id)?;
        if bytes.len() > self.cfg.max_bytes {
            return Err(anyhow!("item too large for cache"));
        }

        let mut inner = self.inner.lock();
        if let Some(prev) = inner.map.insert(id.to_string(), bytes) {
            inner.bytes = inner.bytes.saturating_sub(prev.len());
        } else {
            inner.order.push_back(id.to_string());
        }
        inner.bytes += inner.map.get(id).unwrap().len();

        while inner.map.len() > self.cfg.max_items || inner.bytes > self.cfg.max_bytes {
            if let Some(old) = inner.order.pop_front() {
                if let Some(v) = inner.map.remove(&old) {
                    inner.bytes = inner.bytes.saturating_sub(v.len());
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn stats(&self) -> (usize, usize) {
        let inner = self.inner.lock();
        (inner.map.len(), inner.bytes)
    }
}
