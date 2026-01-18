# Hashing

This document specifies the hashing model used by SIGNIA. Hashing is foundational to:
- content-addressed identity (schema hash)
- bundle integrity (manifest linkage)
- proof construction (Merkle roots and inclusion proofs)
- on-chain anchoring (registry records)

In SIGNIA, hashing is part of the determinism and security contract. Any ambiguity in hashing inputs or domains is treated as a security issue.

---

## 1) Goals

1. Provide stable identifiers for canonical structures.
2. Prevent substitution across contexts via domain separation.
3. Ensure hashes are computed from canonical bytes only.
4. Enable independent verification without trusting the compiler operator.
5. Support proof construction and partial verification.

---

## 2) Hash function selection

SIGNIA MUST select a single primary cryptographic hash function per major version.

Recommended choices:
- SHA-256
- BLAKE3

Rules:
- Do not mix hash functions within the same hash domain version.
- If a different hash function is introduced, bump the hash domain version and document migration.

For v1 specifications, choose one function and treat it as stable.

---

## 3) Domain separation (mandatory)

Every hash in SIGNIA MUST be domain-separated.

### 3.1 Why domain separation
Domain separation prevents reusing the same bytes in a different context to produce the same hash and “pretend” they are a different object.

### 3.2 Domain tag format
A domain tag is a canonical ASCII string prefix (UTF-8 bytes) appended/prepended to the hash input.

Recommended format:
- `signia:<object>:<version>\0`

Where:
- `<object>` identifies the hash context
- `<version>` identifies the domain version
- `\0` is a delimiter to avoid prefix ambiguity

Example tags (illustrative):
- `signia:schema:v1\0`
- `signia:manifest:v1\0`
- `signia:proof-root:v1\0`
- `signia:leaf:entity:v1\0`
- `signia:leaf:edge:v1\0`
- `signia:merkle:node:v1\0`

### 3.3 Domain tag rules
- Tags MUST be constant and documented.
- Tags MUST be included in the hashed bytes exactly as specified.
- Tags MUST NOT be configurable at runtime.

---

## 4) Canonical hash inputs

Hashes MUST be computed over canonical bytes, never over in-memory objects.

### 4.1 Canonical bytes definition
Canonical bytes are produced by:
- normalization (paths/newlines/encoding)
- canonicalization (stable ordering and canonical JSON encoding)

The canonicalization rules are specified in:
- `docs/determinism/canonicalization.md`

### 4.2 Prohibited inputs
Do not include:
- timestamps
- random IDs
- OS-dependent paths
- locale-dependent ordering
- non-deterministic serialization formats

If any such data is present, it must be excluded from hashed domains or normalized deterministically.

---

## 5) Primary identifiers

### 5.1 Schema hash
The schema hash is the canonical identifier of a compiled structure.

Definition:
- `schema_hash = H( domain("signia:schema:v1") || canonical_bytes(schema.json) )`

Where:
- `H` is the selected hash function
- `canonical_bytes(schema.json)` is canonical JSON bytes

### 5.2 Manifest hash (optional but recommended)
The manifest hash provides integrity for compilation context.

Definition:
- `manifest_hash = H( domain("signia:manifest:v1") || canonical_bytes(manifest.json_hashed_view) )`

Notes:
- The manifest may contain non-hashed metadata. If so, define a “hashed view” that excludes non-hashed fields.
- Alternatively, split manifest into `manifest.json` and `manifest.meta.json` to avoid ambiguity.

### 5.3 Proof root
Proof root is derived from the Merkle tree over leaf hashes.

Definition:
- `proof_root = MerkleRoot( ordered_leaf_hashes )`

Leaf hashing and Merkle rules are specified in Section 7.

---

## 6) Hash representation and encoding

### 6.1 Internal representation
Internally:
- hash outputs are raw bytes (e.g., 32 bytes)

### 6.2 External representation
For display and serialization:
- choose exactly one representation (recommended: hex lowercase)
- document it and keep it consistent

Rules:
- do not mix hex and base58 without labeling
- if base58 is used for Solana UX, store the canonical hex bytes in bundle and provide base58 as a derived view

Recommended:
- store as lowercase hex string with `0x` omitted

### 6.3 Comparison rules
- Compare hashes by raw bytes, not string formatting.
- When parsing, accept only the canonical string representation unless explicitly supporting multiple formats.

---

## 7) Merkle tree hashing (proofs)

Proofs provide integrity for large structures with partial verification support.

### 7.1 Leaf definition
Leaves are the canonical elements included in proof construction.

Typical leaves (illustrative):
- entity records
- edge records
- type records
- constraint records

Each leaf must be encoded deterministically (canonical JSON or a binary canonical encoding).

### 7.2 Leaf hashing
Leaf hash definition:
- `leaf_hash = H( domain("signia:leaf:<kind>:v1") || leaf_canonical_bytes )`

Where `<kind>` is a leaf type:
- `entity`
- `edge`
- `type`
- `constraint`

### 7.3 Leaf ordering
Leaves MUST be ordered deterministically before tree construction.

Recommended total ordering:
- `(leaf_kind, stable_id)` with stable tie-breakers.

Rules:
- define stable_id generation in the schema spec
- prohibit ties; if ties occur, define a deterministic tiebreaker or treat as invalid

### 7.4 Node hashing
Merkle internal nodes are hashed with a dedicated domain.

Definition:
- `node_hash = H( domain("signia:merkle:node:v1") || left_hash || right_hash )`

Rules:
- left_hash and right_hash are raw bytes
- concatenation order is strict and must be documented

### 7.5 Odd leaf handling
Choose one policy and standardize it:
- Duplicate last leaf
- Promote last leaf
- Pad with a fixed constant

Recommended:
- duplicate last leaf for simplicity and wide compatibility

If a different policy is chosen, document it as part of the proof version.

### 7.6 Root representation
The Merkle root is represented as:
- raw bytes internally
- canonical hex string in JSON outputs

---

## 8) Hashing for on-chain anchoring (Solana)

The registry program anchors the schema hash (and optionally manifest hash/proof root).

### 8.1 Stored values
Recommended stored values:
- schema_hash (raw 32 bytes)
- schema_version (small integer or enum)
- optional proof_root (raw bytes)
- optional manifest_hash (raw bytes)
- publisher pubkey
- status flags

### 8.2 PDA derivation
PDAs MUST include:
- a static domain seed
- schema hash bytes
- optional version seed

Example (illustrative):
- seeds: `["signia-registry", schema_hash_bytes]`

Rules:
- avoid variable-length, ambiguous seeds
- document seeds and versioning
- ensure collision resistance by including the hash bytes

---

## 9) Multi-hash and future upgrades

### 9.1 Upgrading hash functions
If changing the hash function:
- bump hash domain version (e.g., `v2`)
- store the domain version in schema/manifest/proof
- provide migration tooling:
  - recompute hashes from canonical bytes
  - link versions on-chain if needed

### 9.2 Supporting multiple algorithms
If multiple algorithms are supported:
- do not allow ambiguity in verification
- require an explicit algorithm identifier in the manifest
- keep domain tags algorithm-specific or include algorithm in the domain

---

## 10) Common pitfalls and forbidden patterns

Forbidden:
- hashing non-canonical JSON with arbitrary whitespace
- hashing map iteration order
- hashing stringified debug dumps
- hashing floats without strict canonical rules
- mixing hashed and non-hashed fields without a “hashed view”
- allowing runtime-configured domain tags

Pitfalls:
- inconsistent unicode normalization
- platform-dependent path handling
- concurrency causing nondeterministic merge ordering
- caches that change results (network fetches)

---

## 11) Verification requirements

Verification MUST:
- recompute schema hash from canonical schema bytes
- recompute manifest hash (if used) from canonical hashed view
- recompute leaf hashes and Merkle root per spec
- ensure manifest references match computed hashes
- fail closed on any mismatch

Verification MUST NOT:
- trust hashes provided without recomputation
- accept multiple encodings without explicit rules

---

## 12) Test requirements

Minimum required tests:
- golden fixtures for schema hash stability
- proof root stability tests
- negative tests:
  - schema mutation changes hash
  - leaf mutation changes root
  - manifest mismatch fails verification
- cross-run determinism tests:
  - compile twice yields identical hashes

Recommended:
- cross-platform determinism tests (Linux/macOS/Windows)

---

## 13) Summary

SIGNIA hashing rules ensure:
- stable identity for compiled structures
- strong separation between contexts via domain tags
- reproducible verification through canonical bytes
- safe on-chain anchoring without storing large content

Any change to hashing inputs, domains, or algorithms requires versioning, documentation, and fixture updates.
