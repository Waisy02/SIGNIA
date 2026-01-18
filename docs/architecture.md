
# Architecture

This document describes the architectural design of SIGNIA: a structure-level on-chain compilation system that converts real-world artifacts into canonical, verifiable, composable on-chain forms.

The architecture is designed around three principles:

1. **Structure-first**: focus on structural relationships and constraints, not content replication or execution semantics.
2. **Determinism**: identical inputs must produce identical outputs, regardless of environment.
3. **Verifiability**: outputs must be independently verifiable, locally and (optionally) via minimal on-chain anchors.

---

## High-level system view

SIGNIA consists of an off-chain compilation stack and an optional on-chain registry.

```
               ┌───────────────────────────────┐
               │           Off-chain            │
               │                               │
Input Artifact ─┼─► Ingest ─► Parse ─► Infer ─► │
               │                  │            │
               │                  ▼            │
               │              IR / Model       │
               │                  │            │
               │                  ▼            │
               │   Canonicalize ─► Compile ─►  │──► Bundle (schema/manifest/proof)
               │                  │            │
               │                  ▼            │
               │                Verify         │
               └───────────────────────────────┘
                                 │
                                 │ (optional)
                                 ▼
               ┌───────────────────────────────┐
               │            On-chain            │
               │                               │
               │   Solana Registry Program      │
               │   - schema hash anchor         │
               │   - version links              │
               │   - minimal metadata           │
               └───────────────────────────────┘
                                 │
                                 ▼
                         Query / Fetch / Compose
```

---

## Core components

### 1) Compiler (off-chain)

The compiler is responsible for producing a deterministic bundle from a supported input.

Primary responsibilities:
- Input normalization (paths, encodings, line endings, timestamps)
- Parsing and structural extraction
- Type inference and dependency modeling
- Canonicalization rules and stable ordering
- Bundle emission (schema, manifest, proof)
- Local verification and integrity checks

Boundary:
- The compiler **never executes** untrusted code.
- Parsers and plugins must be treated as potentially hostile input handlers.

### 2) Plugin system

A plugin is an adapter that converts an input type into a common structural intermediate representation (IR).

Examples of plugins:
- Git repository structure plugin (files, modules, dependency edges)
- OpenAPI plugin (endpoints, schemas, request/response types)
- Dataset plugin (fields, constraints, schema evolution)
- Document/spec plugin (sections, references, normative constraints)
- Workflow plugin (nodes/edges, parameters, invariants)
- Config plugin (keys, types, overlays, environment resolution rules)

Plugins are **pure** functions from `(normalized input) → IR`, with explicit configuration and versioning.

### 3) Store + proofs

The store component persists bundles locally and supports deterministic verification.

Responsibilities:
- Content-addressed storage of bundles and intermediate artifacts
- Merkle tree construction for proof material
- Retrieval by hash
- Bundle integrity checks (hashes, schema format, manifest invariants)

The proof format is designed to support:
- Independent verification of bundle integrity
- Stable hash roots for on-chain anchoring
- Partial verification for large structures (where applicable)

### 4) CLI

The CLI is the primary developer entrypoint and is expected to remain stable.

Core commands (conceptual):
- `compile`: generate a bundle
- `verify`: verify a bundle against rules and schemas
- `publish`: register a bundle hash on Solana
- `fetch`: fetch a registered schema by hash
- `inspect`: render human-readable structural summaries
- `plugins`: list or validate plugin availability

### 5) API service

The API service provides automation-friendly access.

Primary use cases:
- Remote compilation in controlled environments
- CI integration
- Batch compilation and verification
- Registry operations (publish/fetch/resolve)

Design goals:
- Stateless request handling where feasible
- Idempotent operations keyed by content hash
- Explicit versioning and backwards compatibility of endpoints

### 6) Console

The Console is an operator/developer UI for:
- Exploring schemas, manifests, proofs
- Searching and browsing structural graphs
- Viewing compile status and verifying outputs
- Providing guided deployment instructions

---

## Data flow and contracts

### Input ingestion

Inputs must be normalized before parsing to preserve determinism.

Normalization must define:
- Path canonicalization rules (separator normalization, root mapping)
- Text normalization (line endings, encoding, BOM handling)
- Timestamp removal and replacement with deterministic markers
- Stable handling of symlinks (policy-driven)
- Stable file traversal ordering

Ingest output:
- A normalized artifact representation
- An `input_descriptor` containing:
  - source type (file/repo/url)
  - pinned reference (commit SHA, tag, checksum)
  - tool versions (compiler/plugin versions)
  - normalization policy version

### Parsing + inference

Parsing extracts a structural model from the normalized input.
Inference creates types, relationships, and constraints.

Parser output requirements:
- No nondeterministic ordering
- No reliance on filesystem iteration order
- No network calls unless explicitly allowed and pinned
- If remote sources exist, they must be content-addressed or pinned by immutable refs

### Intermediate Representation (IR)

The IR is the central contract between plugins and the compiler pipeline.

Design constraints:
- Stable schema with explicit versioning
- Canonical field ordering rules
- Explicit node/edge identity model
- No “free-form” semantics affecting hashing unless normalized

A typical IR contains:
- Entities: nodes with type, identity, attributes
- Edges: directed relations with stable semantics
- Types: structural types, constraints, references
- Annotations: optional metadata (non-hashed or separately hashed domains)

### Canonicalization

Canonicalization transforms IR into a canonical schema.

Rules include:
- Stable sorting for all maps/lists
- Normalization of identifiers (case rules, escaping)
- Normalization of numeric formats
- Canonical JSON serialization (exact bytes for hashing)
- Domain-separated hashing

Canonicalization outputs:
- Canonical schema document
- Canonical manifest document
- Proof material (Merkle leaves, root, inclusion data)

### Bundle format

A bundle is a directory or archive containing:

- `schema.json` (canonical schema, versioned)
- `manifest.json` (inputs, dependencies, hashes, tool versions)
- `proof.json` (root hash and verification material)
- Optional: `artifacts/` (non-hashed helper outputs, debugging artifacts)
- Optional: `meta/` (human-friendly summaries)

Bundle invariants:
- Schema hash must match canonical bytes of `schema.json` (by spec)
- Manifest must reference schema hash and proof root
- Proof root must be derived from defined leaf set and ordering rules
- Any non-hashed content must be explicitly marked and excluded from hash computation

---

## On-chain registry design (Solana)

The on-chain registry is optional but central to on-chain referencing.

### Goals
- Minimal on-chain footprint
- Stable addressability by hash
- Immutable anchoring of structure identifiers
- Version link graph (optional) without storing large content

### Non-goals
- On-chain execution of compilation
- On-chain storage of full schemas
- On-chain indexing of all fields for arbitrary search

### Suggested account model

Conceptual accounts (actual implementation may vary):

- **SchemaRecord** (PDA by `schema_hash`)
  - `schema_hash` (32 bytes)
  - `schema_version`
  - `manifest_hash` (optional)
  - `proof_root` (optional)
  - `created_at_slot` (optional)
  - `publisher` (pubkey)
  - `status` flags (active/deprecated)
  - `links` (optional pointer to version links)

- **VersionLink** (optional)
  - from_hash → to_hash
  - relation type (supersedes, compatible_with, forks_from)
  - rationale hash (optional)

### Registry operations

- **register_schema(schema_hash, metadata)**  
  Stores a minimal record keyed by hash.

- **link_versions(from_hash, to_hash, relation)**  
  Adds a relationship between schema hashes.

- **deprecate(schema_hash)**  
  Marks a hash as deprecated (soft state), without removing history.

### Trust model

Publishing a hash does not imply truth, only referential anchoring:
- It proves that someone registered a particular structure identifier.
- Off-chain verification proves the hash matches a deterministic compilation of an input.

---

## Determinism strategy

Determinism is not an emergent property; it is enforced by design.

### Deterministic inputs
- Prefer immutable refs (commit SHA, checksum)
- Avoid time-dependent sources
- If network access is allowed, require pinned content addressing

### Deterministic pipeline
- No random IDs
- No timestamps in hashed domains
- Stable sorting rules everywhere
- Canonical serialization and hashing domains

### Deterministic verification
- `verify` must recompute hashes from canonical bytes
- Proof verification must be independent and reproducible
- Golden fixtures must test byte-for-byte stability across runs

---

## Security model

### Threats considered
- Malicious inputs designed to crash parsers
- Path traversal and symlink attacks
- Dependency confusion and remote fetch manipulation
- Resource exhaustion (zip bombs, giant files, cyclic graphs)
- On-chain metadata poisoning (misleading titles/links)

### Mitigations
- Strict input normalization and sandbox policies
- Size/time limits per plugin and per compilation stage
- Content-type restrictions and safe parsers
- Explicit allowlists for network access (default deny)
- Separate hashed vs non-hashed metadata domains
- CI hardening: pinned tool versions, dependency audits, CodeQL

---

## Scalability and performance

### Workload characteristics
- Many small compiles in CI (fast path)
- Occasional large compiles (large repos or datasets)
- Verification should be cheaper than compilation

### Scaling strategies
- Content-addressed caching by input descriptor hash
- Incremental compilation (reuse IR fragments where input is partitionable)
- Parallel parsing across independent subtrees
- Streaming Merkle construction for large structures
- API service supports job deduplication by content hash

---

## Deployment topology

SIGNIA can be used in three common modes.

### Mode A: Local developer tooling
- CLI only
- Local bundle store
- Optional registry publish

### Mode B: CI pipeline integration
- CLI in CI
- Bundles uploaded as artifacts
- Verification gates
- Optional publish on tagged builds

### Mode C: Hosted compilation service
- API service + store
- Console UI for inspection
- Optional indexing layer (off-chain) for fast search
- Registry interactions via Solana RPC

---

## Operational boundaries

### Stable public interfaces
- Bundle format (schema/manifest/proof)
- CLI flags and output contracts
- Registry instruction semantics

### Internal evolution
- Plugin implementations
- IR internal representation (must be versioned and migrated)
- Store layout (content-addressed contracts should remain stable)

---

## Implementation map (recommended repository layout)

This document assumes a repository layout similar to:

- `crates/signia-core`: pipeline, IR, canonicalization, hashing
- `crates/signia-plugins`: plugin interfaces and built-ins
- `crates/signia-store`: bundle store and proof primitives
- `crates/signia-api`: API service
- `crates/signia-cli`: CLI entrypoint
- `crates/signia-solana-client`: registry client and RPC helpers
- `programs/signia-registry`: Solana registry program (Anchor)
- `schemas/`: JSON schema specs for bundle formats
- `docs/`: documentation
- `examples/`: runnable examples and test fixtures

---

## Appendix: Invariants checklist

A change is architectural-impacting if it affects:

- Hashing domains or canonical JSON encoding
- Ordering rules for entities/edges/types
- Bundle file structure or schema versions
- Registry account layouts or instruction parameters
- Plugin input normalization policies

Such changes must:
- bump versions where required
- update docs and schemas
- update golden fixtures
- maintain a migration story

---

## Next documents

- `docs/data-model/schema-v1.md`
- `docs/data-model/manifest.md`
- `docs/data-model/proof.md`
- `docs/data-model/determinism-rules.md`
- `docs/onchain/registry-program.md`
- `docs/cli/usage.md`
