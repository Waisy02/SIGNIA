//! Object storage backends.

mod layout;
mod fs;

#[cfg(feature = "s3")]
mod s3;

use std::path::Path;

use anyhow::{anyhow, Result};

pub use fs::FsObjectStore;
pub use layout::{ObjectKey, ObjectLayout};

#[cfg(feature = "s3")]
pub use s3::S3ObjectStore;

#[derive(Debug, Clone)]
pub enum ObjectStoreBackend {
    Fs { dir: String },
    #[cfg(feature = "s3")]
    S3 { bucket: String, prefix: String, region: Option<String> },
}

impl Default for ObjectStoreBackend {
    fn default() -> Self {
        ObjectStoreBackend::Fs { dir: "objects".to_string() }
    }
}

pub struct ObjectStore {
    inner: Box<dyn ObjectStoreImpl + Send + Sync>,
}

impl ObjectStore {
    pub fn open<P: AsRef<Path>>(root: P, backend: ObjectStoreBackend) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root)?;

        let inner: Box<dyn ObjectStoreImpl + Send + Sync> = match backend {
            ObjectStoreBackend::Fs { dir } => Box::new(FsObjectStore::open(root.join(dir))?),
            #[cfg(feature = "s3")]
            ObjectStoreBackend::S3 { bucket, prefix, region } => Box::new(S3ObjectStore::new(bucket, prefix, region)?),
        };

        Ok(Self { inner })
    }

    pub fn put_bytes(&self, alg: &str, bytes: &[u8]) -> Result<String> {
        self.inner.put_bytes(alg, bytes)
    }

    pub fn get_bytes(&self, alg: &str, id: &str) -> Result<Option<Vec<u8>>> {
        self.inner.get_bytes(alg, id)
    }

    pub fn exists(&self, alg: &str, id: &str) -> Result<bool> {
        self.inner.exists(alg, id)
    }
}

pub trait ObjectStoreImpl {
    fn put_bytes(&self, alg: &str, bytes: &[u8]) -> Result<String>;
    fn get_bytes(&self, alg: &str, id: &str) -> Result<Option<Vec<u8>>>;
    fn exists(&self, alg: &str, id: &str) -> Result<bool>;
}

pub fn validate_object_id(id: &str) -> Result<()> {
    if id.len() < 16 || id.len() > 128 {
        return Err(anyhow!("object id length must be 16..=128"));
    }
    if !id.is_ascii() {
        return Err(anyhow!("object id must be ASCII")); 
    }
    for c in id.bytes() {
        if !matches!(c, b'0'..=b'9' | b'a'..=b'f') {
            return Err(anyhow!("object id must be lowercase hex")); 
        }
    }
    Ok(())
}

fn rooted_layout(root: &std::path::Path, alg: &str, id: &str) -> Result<std::path::PathBuf> {
    validate_object_id(id)?;
    Ok(ObjectLayout::new(root.to_path_buf()).path_for(ObjectKey::new(alg, id)?))
}
