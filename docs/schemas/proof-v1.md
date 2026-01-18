
# Proof v1

This document specifies the **SIGNIA Proof v1** format: the verification material emitted as `proof.json` inside a SIGNIA bundle.

Proofs bind a schema to:
- a deterministic leaf set derived from canonical schema content
- domain-separated leaf hashing rules
- deterministic Merkle tree construction rules

The proof root is recorded in:
- `proof.json` (this document)
- `manifest.json` (integrity linkage)
- optionally on-chain in the registry program

---

## 1) Overview

A SIGNIA bundle contains:
- `schema.json`
- `manifest.json`
- `proof.json` (this document)

Proof v1 provides:
- `proof_version` and `hash_domain`
- the proof root and hashing algorithm identifier
- leaf descriptors (optional full list) or a commitment to a leaf set
- optional inclusion proofs for selective verification
- rules to derive and verify the Merkle root

---

## 2) Hashing and domains

Proof v1 uses domain separation for all hashes.

Recommended domain tags (illustrative):
- `signia:proof-root:v1\0`
- `signia:leaf:entity:v1\0`
- `signia:leaf:edge:v1\0`
- `signia:leaf:type:v1\0`
- `signia:leaf:constraint:v1\0`
- `signia:merkle:node:v1\0`

The selected cryptographic hash function MUST be recorded in `proof.json`.

Canonicalization and hashing rules are specified in:
- `docs/determinism/canonicalization.md`
- `docs/determinism/hashing.md`

---

## 3) JSON shape (top-level)

Top-level object:

```json
{
  "proof_version": "v1",
  "hash_domain": "signia:proof:v1",
  "hash_function": "sha256|blake3",
  "root": {
    "root_hash": "<hex>",
    "root_domain": "signia:proof-root:v1",
    "tree": {
      "node_domain": "signia:merkle:node:v1",
      "odd_leaf_rule": "duplicate_last",
      "arity": 2
    }
  },
  "leaves": {
    "leaf_set": {
      "leaf_ordering": "type_then_id",
      "leaf_count": 0,
      "leaf_commitment": "<hex or null>"
    },
    "items": [ ... ]
  },
  "inclusion_proofs": [ ... ],
  "meta": { ... }
}
```

Notes:
- `leaves.items` may be omitted or truncated for large schemas.
- `leaf_commitment` can commit to the full leaf list when `items` is not fully included.

---

## 4) Root section

```json
"root": {
  "root_hash": "<hex>",
  "root_domain": "signia:proof-root:v1",
  "tree": {
    "node_domain": "signia:merkle:node:v1",
    "odd_leaf_rule": "duplicate_last",
    "arity": 2
  }
}
```

Rules:
- `arity` MUST be `2` for v1.
- `odd_leaf_rule` MUST be `"duplicate_last"` for v1.
- `root_hash` MUST match recomputation from the leaf set.

---

## 5) Leaf set definition

### 5.1 What is a leaf
In proof v1, leaves are derived from the canonical schema content:

Leaf types:
- `entity`
- `edge`
- `type`
- `constraint`

Each leaf is defined by:
- a leaf kind
- a stable ID
- canonical bytes representing that record

### 5.2 Leaf canonical bytes
Leaf canonical bytes are computed as:
- canonical JSON encoding of a minimal record projection (recommended), or
- canonical binary encoding defined by the spec

For v1, JSON projections are recommended for simplicity.

Projection rules (recommended):
- Entity leaf projection includes:
  - `id`, `kind`, `name`, `path`, `digest`, `attrs`, `tags`
- Edge leaf projection includes:
  - `id`, `relation`, `from`, `to`, `attrs`
- Type leaf projection includes:
  - `id`, `kind`, `name`, `definition`, `attrs`
- Constraint leaf projection includes:
  - `id`, `kind`, `scope`, `predicate`, `severity`, `attrs`

All projections MUST be canonical JSON bytes (key-sorted, no whitespace).

### 5.3 Leaf hash
Leaf hash definition:

- `leaf_hash = H( domain("signia:leaf:<kind>:v1") || leaf_canonical_bytes )`

Where:
- `<kind>` is one of the leaf kinds
- `H` is the selected hash function

---

## 6) Leaf ordering (deterministic)

Leaves MUST be ordered deterministically before Merkle tree construction.

v1 ordering rule:
- primary: `leaf_kind` (lexicographic)
- secondary: `stable_id` (lexicographic)

This produces a total ordering:
- `(leaf_kind, stable_id)`

Rules:
- `stable_id` MUST be unique within its leaf kind; if duplicates occur, verification MUST fail.
- If stable ID uniqueness cannot be guaranteed by a plugin, the schema is invalid for proof v1.

---

## 7) Merkle tree construction (v1)

### 7.1 Input
The Merkle tree input is the ordered list of leaf hashes as raw bytes.

### 7.2 Node hashing
Internal node hash:

- `node_hash = H( domain("signia:merkle:node:v1") || left_hash || right_hash )`

Rules:
- `left_hash` then `right_hash` order is strict.
- The domain tag is applied to node hashing only.

### 7.3 Odd leaf handling
If the number of leaves at a level is odd:
- duplicate the last leaf hash to make a pair.

This is applied at every level until a root is produced.

### 7.4 Root hash domain
The proof root is computed as:

- `root_hash = H( domain("signia:proof-root:v1") || merkle_root_node_hash )`

Where:
- `merkle_root_node_hash` is the final node hash resulting from Section 7.2 rules.
- If there is exactly one leaf, the leaf hash is treated as the root node hash (still wrapped with root domain).

This final wrap ensures root hashes cannot be confused with node hashes.

---

## 8) Leaves section (serialization)

The proof may include a leaf list for transparency and debugging.

### 8.1 Leaf set header

```json
"leaf_set": {
  "leaf_ordering": "type_then_id",
  "leaf_count": 123,
  "leaf_commitment": "<hex or null>"
}
```

Rules:
- `leaf_ordering` MUST be `"type_then_id"` for v1.
- `leaf_count` MUST match the actual leaf count derived from schema.
- `leaf_commitment` MAY be:
  - null if `items` includes all leaves, or
  - a commitment hash to the complete leaf list if not fully included

### 8.2 Leaf commitment (optional)
If `items` is not complete, `leaf_commitment` provides a compact commitment to the full leaf list.

Recommended commitment:
- canonical JSON array of leaf descriptors:
  - each descriptor: `{ "kind": "...", "id": "...", "hash": "..." }`
- commitment hash:
  - `H( domain("signia:leaf-set:v1") || canonical_bytes(descriptor_array) )`

If used, the domain must be documented and stable.

### 8.3 Leaf items
Leaf item object:

```json
{
  "kind": "entity|edge|type|constraint",
  "id": "<stable id>",
  "hash": "<hex>",
  "projection": { ... }
}
```

Rules:
- `items` MUST be sorted by `(kind, id)`.
- `projection` is optional; if present it MUST match the canonical projection used for hashing.

For large schemas:
- `projection` may be omitted to reduce file size.
- `items` may be omitted entirely if leaf commitment is present.

---

## 9) Inclusion proofs (optional)

Inclusion proofs allow verifying a specific leaf against the root without requiring the full leaf list.

Inclusion proof object:

```json
{
  "kind": "<leaf kind>",
  "id": "<stable id>",
  "leaf_hash": "<hex>",
  "path": [
    { "side": "left", "hash": "<hex>" },
    { "side": "right", "hash": "<hex>" }
  ]
}
```

Rules:
- `path` is ordered from leaf level upward.
- `side` indicates whether the sibling is left or right relative to the current hash.
- Verification recomputes node hashes along the path using the node hashing domain.

Notes:
- Inclusion proofs are optional and may be generated on demand.

---

## 10) Meta section (optional)

`meta` may include:
- generation notes
- file size stats
- timing info

Rules:
- `meta` must not affect verification.
- treat as non-hashed metadata.

---

## 11) Canonicalization requirements (normative)

Proof v1 must be canonicalizable.

Normative rules:
- JSON key ordering and canonical JSON encoding (see canonicalization doc)
- `leaves.items` sorted by `(kind, id)`
- `inclusion_proofs` sorted by `(kind, id)` if included
- no non-deterministic data in hashed fields (`root_hash`, leaf hashes)

---

## 12) Validation rules (normative)

A proof MUST fail validation if:
- `proof_version` is not `"v1"`
- `hash_domain` is incorrect
- `hash_function` is missing or unsupported for this bundle
- `root.root_hash` is missing
- leaf ordering is violated
- any `leaf_hash` does not match recomputation from schema projections
- the recomputed root does not match `root.root_hash`

---

## 13) Example (minimal)

```json
{
  "proof_version": "v1",
  "hash_domain": "signia:proof:v1",
  "hash_function": "sha256",
  "root": {
    "root_hash": "4567...cdef",
    "root_domain": "signia:proof-root:v1",
    "tree": {
      "node_domain": "signia:merkle:node:v1",
      "odd_leaf_rule": "duplicate_last",
      "arity": 2
    }
  },
  "leaves": {
    "leaf_set": {
      "leaf_ordering": "type_then_id",
      "leaf_count": 1,
      "leaf_commitment": null
    },
    "items": [
      {
        "kind": "entity",
        "id": "ent:endpoint:GET_/health",
        "hash": "abcd...0123",
        "projection": {
          "id": "ent:endpoint:GET_/health",
          "kind": "endpoint",
          "name": "GET /health",
          "path": null,
          "digest": null,
          "attrs": { "method": "GET", "route": "/health" },
          "tags": ["public"]
        }
      }
    ]
  },
  "inclusion_proofs": [],
  "meta": {}
}
```

---

## 14) Related documents

- Schema spec: `docs/schemas/schema-v1.md`
- Manifest spec: `docs/schemas/manifest-v1.md`
- Canonicalization: `docs/determinism/canonicalization.md`
- Hashing: `docs/determinism/hashing.md`
- Determinism contract: `docs/determinism/determinism-contract.md`
