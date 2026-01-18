//! Deterministic object layout.

use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::objects::validate_object_id;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectKey {
    pub alg: String,
    pub id: String,
}

impl ObjectKey {
    pub fn new(alg: &str, id: &str) -> Result<Self> {
        if alg.trim().is_empty() {
            return Err(anyhow!("hash algorithm must not be empty"));
        }
        if !alg.is_ascii() {
            return Err(anyhow!("hash algorithm must be ASCII"));
        }
        validate_object_id(id)?;
        Ok(Self { alg: alg.to_string(), id: id.to_string() })
    }

    pub fn prefix2(&self) -> (&str, &str) {
        (&self.id[0..2], &self.id[2..4])
    }
}

#[derive(Debug, Clone)]
pub struct ObjectLayout {
    root: PathBuf,
}

impl ObjectLayout {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn path_for(&self, key: ObjectKey) -> PathBuf {
        let (aa, bb) = key.prefix2();
        self.root.join(key.alg).join(aa).join(bb).join(key.id)
    }
}
