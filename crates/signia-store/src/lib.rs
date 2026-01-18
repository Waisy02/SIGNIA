//! Deterministic storage primitives for SIGNIA.

pub mod cache;
pub mod kv;
pub mod objects;
pub mod proofs;

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::kv::{Kv, KvBackend};
use crate::objects::{ObjectStore, ObjectStoreBackend};

#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub root_dir: PathBuf,
    pub kv_backend: KvBackend,
    pub object_backend: ObjectStoreBackend,
    pub hash_alg: String,
}

impl StoreConfig {
    pub fn local_dev<P: AsRef<Path>>(root_dir: P) -> Result<Self> {
        let root = root_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&root)?;
        Ok(Self {
            root_dir: root,
            kv_backend: KvBackend::default(),
            object_backend: ObjectStoreBackend::default(),
            hash_alg: "sha256".to_string(),
        })
    }
}

pub struct Store {
    cfg: StoreConfig,
    kv: Kv,
    objects: ObjectStore,
}

impl Store {
    pub fn open(cfg: StoreConfig) -> Result<Self> {
        let kv = Kv::open(cfg.root_dir.join("kv"), cfg.kv_backend.clone())?;
        let objects = ObjectStore::open(cfg.root_dir.join("objects"), cfg.object_backend.clone())?;
        Ok(Self { cfg, kv, objects })
    }

    pub fn config(&self) -> &StoreConfig {
        &self.cfg
    }

    pub fn kv(&self) -> &Kv {
        &self.kv
    }

    pub fn objects(&self) -> &ObjectStore {
        &self.objects
    }

    pub fn put_object_bytes(&self, bytes: &[u8]) -> Result<String> {
        self.objects.put_bytes(&self.cfg.hash_alg, bytes)
    }

    pub fn get_object_bytes(&self, id: &str) -> Result<Option<Vec<u8>>> {
        self.objects.get_bytes(&self.cfg.hash_alg, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn store_roundtrip() {
        let td = TempDir::new().unwrap();
        let cfg = StoreConfig::local_dev(td.path()).unwrap();
        let store = Store::open(cfg).unwrap();

        let id = store.put_object_bytes(b"abc").unwrap();
        let got = store.get_object_bytes(&id).unwrap().unwrap();
        assert_eq!(got, b"abc");

        store.kv().put_json("k", &id).unwrap();
        let got_id: String = store.kv().get_json("k").unwrap().unwrap();
        assert_eq!(got_id, id);
    }
}
