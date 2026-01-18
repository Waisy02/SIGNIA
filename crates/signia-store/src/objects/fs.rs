//! Filesystem object store backend.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use sha2::{Digest, Sha256};

use super::{rooted_layout, validate_object_id, ObjectStoreImpl};

pub struct FsObjectStore {
    root: PathBuf,
}

impl FsObjectStore {
    pub fn open<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl ObjectStoreImpl for FsObjectStore {
    fn put_bytes(&self, alg: &str, bytes: &[u8]) -> Result<String> {
        let id = match alg {
            "sha256" => {
                let mut h = Sha256::new();
                h.update(bytes);
                hex::encode(h.finalize())
            }
            _ => anyhow::bail!("unsupported hash algorithm: {alg}"),
        };

        let path = rooted_layout(&self.root, alg, &id)?;
        if path.exists() {
            return Ok(id);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp = path.with_extension("tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(bytes)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &path)?;
        Ok(id)
    }

    fn get_bytes(&self, alg: &str, id: &str) -> Result<Option<Vec<u8>>> {
        validate_object_id(id)?;
        let path = rooted_layout(&self.root, alg, id)?;
        if !path.exists() {
            return Ok(None);
        }
        let mut f = fs::File::open(&path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(Some(buf))
    }

    fn exists(&self, alg: &str, id: &str) -> Result<bool> {
        validate_object_id(id)?;
        Ok(rooted_layout(&self.root, alg, id)?.exists())
    }
}
