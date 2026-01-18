# signia-store

`signia-store` provides deterministic local persistence and artifact storage primitives for SIGNIA.

It supports:
- Content-addressed artifacts (objects keyed by stable digests)
- Deterministic object layout (stable paths for the same object id)
- Simple KV metadata (fingerprints, manifests, execution metadata)
- Merkle proofs (roots and inclusion proofs)
- A predictable content-addressed in-memory cache

By default, it is local and simple:
- SQLite KV store
- Filesystem object store

## Features

- `sqlite` (default): SQLite-backed KV store
- `fs` (default): filesystem object store
- `s3` (optional): S3 object store backend (requires async runtime)

## Quickstart

```rust
use signia_store::{Store, StoreConfig};

fn main() -> anyhow::Result<()> {
    let cfg = StoreConfig::local_dev("./.signia")?;
    let store = Store::open(cfg)?;

    let obj = b"hello world".to_vec();
    let id = store.put_object_bytes(&obj)?;

    store.kv().put_json("example.object_id", &id)?;
    let roundtrip: String = store.kv().get_json("example.object_id")?.unwrap();

    assert_eq!(roundtrip, id);
    Ok(())
}
```

## Layout

Objects are stored under:

```
objects/<alg>/<aa>/<bb>/<digest>
```

Where `<aa>` and `<bb>` are the first two and next two hex characters of the digest.

## License

MIT OR Apache-2.0
