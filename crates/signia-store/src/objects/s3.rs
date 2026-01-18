//! S3 object store backend (optional).

#![cfg(feature = "s3")]

use std::sync::OnceLock;

use anyhow::Result;
use aws_config::Region;
use aws_sdk_s3::{primitives::ByteStream, Client};
use bytes::Bytes;
use sha2::{Digest, Sha256};

use super::{ObjectStoreImpl, validate_object_id};

static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn rt() -> &'static tokio::runtime::Runtime {
    RT.get_or_init(|| tokio::runtime::Runtime::new().expect("tokio runtime"))
}

pub struct S3ObjectStore {
    bucket: String,
    prefix: String,
    client: Client,
}

impl S3ObjectStore {
    pub fn new(bucket: String, prefix: String, region: Option<String>) -> Result<Self> {
        let client = rt().block_on(async move {
            let mut loader = aws_config::from_env();
            if let Some(r) = region {
                loader = loader.region(Region::new(r));
            }
            let conf = loader.load().await;
            Ok::<Client, anyhow::Error>(Client::new(&conf))
        })?;

        Ok(Self { bucket, prefix: prefix.trim_matches('/').to_string(), client })
    }

    fn key(&self, alg: &str, id: &str) -> String {
        if self.prefix.is_empty() {
            format!("{alg}/{id}")
        } else {
            format!("{}/{alg}/{id}", self.prefix)
        }
    }
}

impl ObjectStoreImpl for S3ObjectStore {
    fn put_bytes(&self, alg: &str, bytes: &[u8]) -> Result<String> {
        let id = match alg {
            "sha256" => {
                let mut h = Sha256::new();
                h.update(bytes);
                hex::encode(h.finalize())
            }
            _ => anyhow::bail!("unsupported hash algorithm: {alg}"),
        };

        let key = self.key(alg, &id);
        let bucket = self.bucket.clone();
        let client = self.client.clone();
        let body = ByteStream::from(Bytes::copy_from_slice(bytes));

        rt().block_on(async move {
            client.put_object().bucket(bucket).key(key).body(body).send().await?;
            Ok::<(), anyhow::Error>(())
        })?;

        Ok(id)
    }

    fn get_bytes(&self, alg: &str, id: &str) -> Result<Option<Vec<u8>>> {
        validate_object_id(id)?;
        let key = self.key(alg, id);
        let bucket = self.bucket.clone();
        let client = self.client.clone();

        let out = rt().block_on(async move {
            let resp = client.get_object().bucket(bucket).key(key).send().await;
            match resp {
                Ok(r) => Ok::<Option<Vec<u8>>, anyhow::Error>(Some(r.body.collect().await?.into_bytes().to_vec())),
                Err(e) => {
                    let msg = format!("{e}");
                    if msg.contains("NotFound") || msg.contains("NoSuchKey") {
                        Ok(None)
                    } else {
                        Err(anyhow::anyhow!(e))
                    }
                }
            }
        })?;
        Ok(out)
    }

    fn exists(&self, alg: &str, id: &str) -> Result<bool> {
        validate_object_id(id)?;
        let key = self.key(alg, id);
        let bucket = self.bucket.clone();
        let client = self.client.clone();

        let ok = rt().block_on(async move {
            let resp = client.head_object().bucket(bucket).key(key).send().await;
            match resp {
                Ok(_) => Ok::<bool, anyhow::Error>(true),
                Err(e) => {
                    let msg = format!("{e}");
                    if msg.contains("NotFound") || msg.contains("NoSuchKey") {
                        Ok(false)
                    } else {
                        Err(anyhow::anyhow!(e))
                    }
                }
            }
        })?;
        Ok(ok)
    }
}
