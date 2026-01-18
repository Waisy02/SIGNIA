
# signia-core

`signia-core` is the foundational Rust crate for SIGNIA. It provides the types and deterministic primitives shared across:
- SIGNIA CLI
- SIGNIA API service
- SIGNIA Console integrations (via API contracts)
- plugin implementations

This crate focuses on:
- schema / manifest / proof data structures
- canonical JSON encoding for stable hashing
- hashing utilities (SHA-256, BLAKE3)
- Merkle root construction for proofs
- deterministic path normalization helpers

If you are building custom plugins or integrating SIGNIA in Rust, this is the crate you import.

---

## What this crate contains

### Data model
- `SchemaV1` and supporting `Entity` / `Edge` structures
- `ManifestV1` describing inputs, policies, outputs, and computed hashes
- `ProofV1` describing Merkle roots and optional inclusion proofs

### Determinism utilities
- canonical JSON encoding that produces stable bytes
- stable hashing with domain separation
- stable ordering for entities, edges, and leaf keys

### Hashing
- SHA-256 (feature: `sha256`)
- BLAKE3 (feature: `blake3`)

### Proofs
- Merkle root over canonical leaves
- optional inclusion proof generation/verification

---

## Installation

Add the crate to your Cargo workspace or dependency list:

```toml
[dependencies]
signia-core = { path = "../crates/signia-core" }
```

Or, if published:
```toml
signia-core = "0.1.0"
```

---

## Example: canonical hash of schema

```rust
use signia_core::canonical::canonical_json_bytes;
use signia_core::hash::{HashAlg, hash_with_domain};

fn main() -> anyhow::Result<()> {
    let schema = serde_json::json!({
        "version": "v1",
        "kind": "repo",
        "meta": { "name": "demo", "createdAt": "1970-01-01T00:00:00Z",
          "source": { "type": "path", "locator": "artifact:/demo" },
          "normalization": { "policyVersion": "v1", "pathRoot": "artifact:/", "newline": "lf", "encoding": "utf-8", "symlinks": "deny", "network": "deny" }
        },
        "entities": [],
        "edges": []
    });

    let bytes = canonical_json_bytes(&schema)?;
    let digest = hash_with_domain(HashAlg::Sha256, "signia.v1.schema", &bytes)?;
    println!("{}", hex::encode(digest));
    Ok(())
}
```

---

## Feature flags

- `sha256` (default) — enables SHA-256 hashing
- `blake3` (default) — enables BLAKE3 hashing
- `canonical-json` (default) — enables canonical JSON encoding helpers
- `parallel` — enables parallel hashing and sorting for large artifacts

Disable defaults if you need a minimal build:
```toml
signia-core = { path = "../crates/signia-core", default-features = false, features = ["sha256"] }
```

---

## Determinism rules (high level)

The core determinism contract is:
- canonicalize JSON to stable bytes before hashing
- avoid host-specific absolute paths
- normalize newlines as LF
- stable sorting of:
  - entity ids
  - edge ids
  - proof leaf keys

See:
- `docs/determinism/determinism-contract.md`
- `docs/determinism/canonicalization.md`
- `docs/determinism/hashing.md`

---

## Crate layout

- `src/`
  - `lib.rs` — public exports
  - `model/` — schema/manifest/proof structures
  - `canonical/` — canonical JSON encoding
  - `hash/` — hashing utilities with domain separation
  - `merkle/` — Merkle tree logic and proofs
  - `path/` — path normalization and artifact path rules
  - `errors.rs` — error types

---

## Testing

Run unit tests:
```bash
cargo test -p signia-core
```

Run property-based tests:
```bash
cargo test -p signia-core proptest
```

---

## License

Apache-2.0
