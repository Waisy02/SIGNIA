//! KV storage backends.

mod memory;

#[cfg(feature = "sqlite")]
mod sqlite;

use std::path::Path;

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};

pub use memory::MemoryKv;

#[cfg(feature = "sqlite")]
pub use sqlite::SqliteKv;

#[derive(Debug, Clone)]
pub enum KvBackend {
    Memory,
    #[cfg(feature = "sqlite")]
    Sqlite { path: String },
}

impl Default for KvBackend {
    fn default() -> Self {
        #[cfg(feature = "sqlite")]
        {
            return KvBackend::Sqlite { path: "kv.sqlite3".to_string() };
        }
        #[cfg(not(feature = "sqlite"))]
        {
            KvBackend::Memory
        }
    }
}

pub struct Kv {
    inner: RwLock<Box<dyn KvStore + Send + Sync>>,
}

impl Kv {
    pub fn open<P: AsRef<Path>>(dir: P, backend: KvBackend) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;

        let store: Box<dyn KvStore + Send + Sync> = match backend {
            KvBackend::Memory => Box::new(MemoryKv::default()),
            #[cfg(feature = "sqlite")]
            KvBackend::Sqlite { path } => Box::new(SqliteKv::open(dir.join(path))?),
        };

        Ok(Self { inner: RwLock::new(store) })
    }

    pub fn put_bytes(&self, key: &str, value: Vec<u8>) -> Result<()> {
        validate_key(key)?;
        self.inner.write().put(key, value)
    }

    pub fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>> {
        validate_key(key)?;
        self.inner.read().get(key)
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        validate_key(key)?;
        self.inner.write().delete(key)
    }

    pub fn put_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.put_bytes(key, serde_json::to_vec(value)?)
    }

    pub fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let Some(bytes) = self.get_bytes(key)? else { return Ok(None); };
        Ok(Some(serde_json::from_slice(&bytes)?))
    }

    pub fn list_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        validate_key(prefix)?;
        self.inner.read().list_prefix(prefix)
    }
}

pub trait KvStore {
    fn put(&mut self, key: &str, value: Vec<u8>) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn delete(&mut self, key: &str) -> Result<()>;
    fn list_prefix(&self, prefix: &str) -> Result<Vec<String>>;
}

pub fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() || key.len() > 256 {
        return Err(anyhow!("kv key must be 1..=256 chars"));
    }
    if !key.is_ascii() {
        return Err(anyhow!("kv key must be ASCII"));
    }
    for b in key.bytes() {
        let ok = matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b'/' | b':');
        if !ok {
            return Err(anyhow!("kv key contains invalid char"));
        }
    }
    Ok(())
}
