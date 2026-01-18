
# Plugin Specification

This document defines the SIGNIA plugin specification. Plugins are responsible for turning real-world inputs into a shared intermediate representation (IR) that SIGNIA can validate, canonicalize, hash, and prove.

Plugins must be:
- deterministic (same normalized input → same IR)
- bounded (limits and failure modes are deterministic)
- isolated (no unsafe side effects by default)
- versioned (config and behavior changes must be explicit)

This spec covers:
- plugin lifecycle
- input and output contracts
- IR shapes and constraints
- configuration and hashing
- sandbox permissions
- testing and determinism requirements

---

## 1) Plugin architecture overview

SIGNIA uses a multi-stage pipeline:

1. Ingest input (untrusted)
2. Normalize (paths/newlines/encoding)
3. Execute plugin(s) to produce IR
4. Validate IR invariants
5. Canonicalize schema + compute hashes
6. Build proof
7. Emit bundle

Plugins run in stage (3). Everything they produce is untrusted until validated.

---

## 2) Plugin responsibilities

A plugin:
- declares supported input kinds (repo, file, url, stdin)
- parses or analyzes input
- emits deterministic IR:
  - entities (nodes)
  - edges (relationships)
  - types (optional)
  - constraints (optional)
- declares configuration schema and default config
- declares required permissions (filesystem/network)
- implements bounded execution with deterministic failures

Non-responsibilities:
- do not write to chain directly
- do not embed execution logic
- do not include secrets
- do not produce nondeterministic IDs

---

## 3) Plugin identity and versioning

### 3.1 Plugin ID
Each plugin has a stable ID:
- `name`: e.g., `openapi`, `repo`, `dataset`, `markdown`
- `vendor`: optional namespace (e.g., `signia`)

Canonical plugin identifier:
- `plugin_id = "<vendor>:<name>"` if vendor exists
- otherwise `plugin_id = "<name>"`

### 3.2 Versioning
Plugins MUST use semantic versioning.

Behavioral changes that affect output IR MUST:
- bump plugin version
- update fixtures and expected hashes
- document changes

---

## 4) Configuration contract

Plugins MUST define:
- configuration schema (JSON schema or equivalent)
- default configuration
- canonical config encoding for hashing

### 4.1 Config hashing
The plugin config hash is recorded in `manifest.json`.

Definition:
- `config_hash = H( domain("signia:plugin-config:v1") || canonical_bytes(config_object) )`

Rules:
- config object must be canonical JSON
- default values must be explicit in canonical form (avoid implicit defaults)
- `config_hash` changes when any output-affecting config changes

### 4.2 Config validation
Plugin must validate config:
- reject unknown fields (recommended) to avoid silent behavior changes
- enforce type constraints and bounds

---

## 5) Permissions and sandbox

Plugins operate under a permission model.

### 5.1 Filesystem
Default:
- read-only access inside input root
- no write access outside output directory

Symlinks:
- default deny
- if allowed, resolve within root only

### 5.2 Network
Default:
- deny network access

If enabled:
- allow only pinned/immutable fetches
- enforce timeouts and size limits
- record resolved immutable identifiers in manifest

### 5.3 Process execution
Default:
- deny shell/process execution

If a plugin requires external tools:
- it MUST be opt-in
- it MUST be explicit in config
- it MUST be documented as unsafe
- it MUST be disabled in CI by default

---

## 6) IR output contract (normalized IR)

Plugins emit a normalized IR that SIGNIA can validate and convert to Schema v1.

### 6.1 IR top-level shape

```json
{
  "ir_version": "v1",
  "plugin": { "id": "openapi", "version": "0.1.0" },
  "artifact": { ... },
  "entities": [ ... ],
  "edges": [ ... ],
  "types": [ ... ],
  "constraints": [ ... ],
  "stats": { ... }
}
```

Rules:
- `ir_version` MUST be `"v1"` for this spec.
- `entities` MUST have unique IDs.
- `edges` must reference existing entity IDs.
- All arrays that represent sets must be emitted in deterministic order (or SIGNIA must sort during canonicalization; plugin still must avoid nondeterministic duplicates).

### 6.2 IR artifact descriptor
Same intent as schema artifact descriptor, but may include plugin-specific fields:

```json
"artifact": {
  "kind": "repo|openapi|dataset|workflow|config|spec|unknown",
  "name": "<string>",
  "namespace": "<string>",
  "ref": "<string>",
  "labels": ["<string>", "..."]
}
```

Rules:
- `labels` should be sorted and deduplicated.

---

## 7) Entity emission rules

Entity object:

```json
{
  "id": "<stable entity id>",
  "kind": "<string>",
  "name": "<string>",
  "path": "<logical normalized path or null>",
  "digest": "<hex content digest or null>",
  "attrs": { ... },
  "tags": ["<string>", "..."]
}
```

### 7.1 Stable entity IDs
IDs MUST be stable and deterministic.

Recommended strategy:
- `id = "ent:<kind>:<stable-key>"`

Stable-key sources:
- normalized path
- normalized URI
- OpenAPI method + route
- dataset table + column

Rules:
- never use random UUIDs
- never use memory addresses
- never incorporate timestamps

### 7.2 Attribute constraints
- `attrs` must be JSON object with canonicalizable values.
- avoid floats and nondeterministic representations.

### 7.3 Sorting (plugin-side guidance)
Even if SIGNIA sorts later, plugins should produce deterministic order:
- sort by `(kind, id)`.

---

## 8) Edge emission rules

Edge object:

```json
{
  "id": "<stable edge id>",
  "relation": "<string>",
  "from": "<entity id>",
  "to": "<entity id>",
  "attrs": { ... }
}
```

### 8.1 Stable edge IDs
Recommended:
- `id = "edge:<relation>:<from>:<to>:<tiebreaker>"`

Where `tiebreaker` is deterministic, such as:
- a stable index derived from sorted occurrences
- a stable attribute hash

### 8.2 Sorting
- sort by `(relation, from, to, id)`.

---

## 9) Type emission rules (optional)

Plugins may emit types when structure requires it (OpenAPI/JSON Schema).

Type object:

```json
{
  "id": "<stable type id>",
  "kind": "object|array|string|number|integer|boolean|null|enum|ref|union",
  "name": "<string>",
  "definition": { ... },
  "attrs": { ... }
}
```

Rules:
- type IDs must be stable
- sort by `(kind, id)`
- `definition` must be canonicalizable

---

## 10) Constraint emission rules (optional)

Constraint object:

```json
{
  "id": "<stable constraint id>",
  "kind": "<string>",
  "scope": { "entities": ["..."], "types": ["..."] },
  "predicate": { ... },
  "severity": "info|warn|error",
  "attrs": { ... }
}
```

Rules:
- scope lists must be sorted and deduplicated
- constraints sorted by `(kind, id)`

---

## 11) Bounds and deterministic failures

Plugins MUST enforce bounds:
- maximum input bytes processed
- maximum entities/edges emitted
- maximum recursion depth

If limits are exceeded, the plugin MUST:
- fail with a deterministic error code
- include stable context (logical path, entity kind), not host paths

Recommended error shape:

```json
{
  "code": "PLUGIN_LIMIT_EXCEEDED",
  "message": "Entity limit exceeded",
  "details": { "max_entities": 200000 }
}
```

---

## 12) Determinism requirements (normative)

Plugins MUST NOT depend on:
- filesystem iteration order (must sort)
- locale/timezone
- wall-clock time
- random number generators
- nondeterministic concurrency ordering

Plugins MUST:
- normalize paths to logical form
- normalize newlines if text affects structure
- use stable ordering for collections
- use stable IDs derived from normalized inputs

---

## 13) Testing requirements

Every plugin should provide fixtures:
- input artifacts under `fixtures/<plugin>/<name>/input/`
- expected IR under `fixtures/<plugin>/<name>/expected/ir.json` (optional)
- expected bundle outputs under `fixtures/<plugin>/<name>/expected/` (recommended)

Minimum tests:
- compile fixture twice → identical schema bytes and hashes
- verify bundle passes `signia verify`
- negative test: mutate schema.json → verify fails

Recommended tests:
- fuzz parsing inputs (where feasible)
- cross-platform fixture checks

---

## 14) Plugin distribution and loading

Plugins may be:
- built-in (compiled into the binary)
- dynamic modules (future extension)
- external processes (unsafe mode; opt-in)

Loading rules:
- plugin identity and version must be surfaced in manifest
- plugins must declare permissions
- configuration hash must be computed and recorded

---

## 15) Compatibility and upgrades

If a plugin changes IR semantics:
- bump plugin version
- update schema/manifest expectations
- document changes in changelog
- keep old fixtures if you want to support old outputs

For breaking format changes:
- bump `ir_version` (future v2)
- update SIGNIA core accordingly

---

## 16) Appendix: Recommended built-in plugins

Common built-in plugins:
- `repo`:
  - emits module/file graph and import relationships
- `openapi`:
  - emits endpoints, schemas, and request/response relationships
- `jsonschema`:
  - emits type graphs and references
- `dataset`:
  - emits tables, columns, and constraints
- `markdown`:
  - emits document sections and references
- `workflow`:
  - emits workflow steps and dependency DAG

---

## 17) Related documents

- Schema v1: `docs/schemas/schema-v1.md`
- Manifest v1: `docs/schemas/manifest-v1.md`
- Proof v1: `docs/schemas/proof-v1.md`
- Canonicalization: `docs/determinism/canonicalization.md`
- Hashing: `docs/determinism/hashing.md`
- Determinism contract: `docs/determinism/determinism-contract.md`
- Threat model: `docs/security/security/threat-model.md`
