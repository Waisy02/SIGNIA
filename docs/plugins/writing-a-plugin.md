
# Writing a Plugin

This guide explains how to write a SIGNIA plugin that produces deterministic IR v1 and compiles into verifiable Schema v1 bundles.

It includes:
- plugin lifecycle
- configuration and hashing
- deterministic ID strategies
- bounds and error handling
- an end-to-end example plugin skeleton

This guide is intentionally practical and maps directly to repository code conventions.

Related specs:
- `docs/plugins/plugin-spec.md`
- `docs/schemas/schema-v1.md`
- `docs/determinism/determinism-contract.md`
- `docs/determinism/canonicalization.md`
- `docs/determinism/hashing.md`

---

## 1) Mental model

A SIGNIA plugin is a **structure extractor**:
- input bytes → normalized view → deterministic IR

SIGNIA core is responsible for:
- validating IR invariants
- canonicalizing schema bytes
- hashing and proof construction
- bundle emission and verification

Plugins should be small, deterministic, and safe by default.

---

## 2) Where plugins live in the repo

Recommended layout:

- `crates/signia-plugins/` — plugin implementations (Rust)
- `crates/signia-core/` — pipeline and IR validation
- `crates/signia-cli/` — CLI commands and plugin wiring
- `crates/signia-ir/` — IR types and helpers
- `crates/signia-schema/` — schema projection and canonicalization helpers

A new built-in plugin typically adds:
- `crates/signia-plugins/src/<plugin_name>/`
- fixtures under `fixtures/<plugin_name>/...`
- docs updates in `docs/plugins/builtin-plugins.md`

---

## 3) Plugin lifecycle

At runtime, SIGNIA will:

1. Parse CLI args / config file
2. Resolve and normalize input (paths/newlines/encoding)
3. Instantiate plugin with config
4. Run plugin:
   - plugin reads the normalized input root
   - plugin emits IR entities/edges/types/constraints
5. Core validates IR and emits bundle

Plugin entrypoints:
- `Plugin::metadata()`
- `Plugin::config_schema()`
- `Plugin::default_config()`
- `Plugin::run(ctx) -> Result<IrBundle, PluginError>`

---

## 4) Determinism checklist (mandatory)

Your plugin MUST satisfy:

- Stable file enumeration (sort by normalized path)
- Stable parsing (depends only on bytes and config)
- Stable IDs (derived from normalized inputs)
- Stable ordering in emitted collections (or at least no nondeterministic duplicates)
- No wall-clock time
- No randomness
- No locale/timezone dependencies
- No environment variable dependencies (unless explicitly allowlisted and recorded, and not affecting hashed outputs)

If you violate any of these, the bundle hashes will drift and verification becomes unreliable.

---

## 5) Configuration and config hashing

### 5.1 Define a config struct
Your config must:
- have explicit defaults
- reject unknown fields (recommended)
- be canonicalizable for hashing

Example (Rust):

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleConfig {
    pub emit_digests: bool,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub max_file_bytes: u64,
}

impl Default for ExampleConfig {
    fn default() -> Self {
        Self {
            emit_digests: true,
            include_globs: vec!["**/*".to_string()],
            exclude_globs: vec!["**/.git/**".to_string(), "**/target/**".to_string()],
            max_file_bytes: 1_048_576,
        }
    }
}
```

### 5.2 Config hash
Core should compute and record:
- `config_hash = H(domain("signia:plugin-config:v1") || canonical_bytes(config))`

Your plugin should not implement hashing itself unless required; it should expose the config struct and let core hash the canonical JSON.

---

## 6) Input handling

### 6.1 Use the normalized input root
The plugin receives an input root already normalized by core.

Rules:
- never use absolute host paths in IDs
- never read outside the input root
- treat input as read-only

### 6.2 File enumeration
Always enumerate files deterministically:

- collect all candidate paths
- normalize to logical paths (e.g., `artifact:/...`)
- sort lexicographically
- then process in sorted order

### 6.3 Bounds and truncation
Respect global limits passed via context:
- max file bytes
- max files
- max depth
- timeout

If you must truncate:
- do it deterministically (e.g., first N files by sorted order)
- record the truncation in IR stats and schema attrs (as a flag)

---

## 7) Stable ID strategies

Stable IDs are the most important part of a plugin.

### 7.1 Entity IDs
Recommended scheme:
- `ent:<kind>:<stable-key>`

Stable keys:
- normalized logical paths: `src/main.ts`
- normalized URIs: `api:/GET_/health`
- name + disambiguator: `User#email`

Avoid:
- numeric indexes based on discovery order unless order is sorted and stable
- hashing raw file content unless necessary
- platform-dependent path separators

### 7.2 Edge IDs
Recommended:
- `edge:<relation>:<from>:<to>:<tiebreaker>`

`tiebreaker` should be deterministic:
- `0` if unique edge
- or a stable hash of edge attrs canonical bytes
- or a stable occurrence index after sorting duplicates by a stable key

### 7.3 Type IDs
Recommended:
- `type:<kind>:<stable-id>`
- stable-id can be derived from schema names or normalized JSON pointers

---

## 8) Emitting IR v1

The IR v1 shape is defined in `docs/plugins/plugin-spec.md`.

Minimum required fields:
- `ir_version`
- `plugin` metadata
- `artifact`
- `entities` and `edges`

Optional:
- `types`, `constraints`
- `stats` (recommended)

Your plugin should always emit:
- sorted lists where possible
- deduplicated tags/labels
- canonicalizable attrs

---

## 9) Error handling

Errors must be deterministic.

### 9.1 Error codes
Use stable error codes:

- `PLUGIN_INVALID_INPUT`
- `PLUGIN_PARSE_ERROR`
- `PLUGIN_LIMIT_EXCEEDED`
- `PLUGIN_UNSUPPORTED_FORMAT`
- `PLUGIN_IO_ERROR` (avoid embedding OS-specific messages)

### 9.2 Include stable context
Prefer logical paths and stable identifiers:
- `artifact:/path/to/file`
Not:
- `/home/runner/work/...`

---

## 10) Testing your plugin

### 10.1 Add fixtures
Create:
- `fixtures/<plugin>/<name>/input/...`
- `fixtures/<plugin>/<name>/expected/schema.json`
- `fixtures/<plugin>/<name>/expected/manifest.json`
- `fixtures/<plugin>/<name>/expected/proof.json`
- `fixtures/<plugin>/<name>/expected/hashes.json`

### 10.2 Determinism tests
In CI:
- compile fixture twice, compare bytes
- verify bundle
- run negative tests (tamper, verify fails)

### 10.3 Cross-platform checks
Run fixtures on:
- Linux and macOS at minimum

---

## 11) Example: a tiny "lines" plugin (walk files and emit file entities)

This example plugin:
- enumerates files under input root
- emits one entity per file
- emits `contains` edges from a root entity to each file
- optionally computes content digests
- enforces file size limits

This is a minimal skeleton to copy.

### 11.1 Example config

```json
{
  "emit_digests": true,
  "include_globs": ["**/*.md"],
  "exclude_globs": ["**/.git/**"],
  "max_file_bytes": 1048576
}
```

### 11.2 Example entity kinds
- `root`
- `file`

### 11.3 Suggested IR stats
- file_count
- total_bytes
- truncated (bool)

---

## 12) Recommended code skeleton (Rust)

Below is a conceptual skeleton. The real repository implementation should match the actual core traits and types.

```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinesConfig {
    pub emit_digests: bool,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub max_file_bytes: u64,
}

impl Default for LinesConfig {
    fn default() -> Self {
        Self {
            emit_digests: true,
            include_globs: vec!["**/*".to_string()],
            exclude_globs: vec!["**/.git/**".to_string(), "**/target/**".to_string()],
            max_file_bytes: 1_048_576,
        }
    }
}

pub struct LinesPlugin {
    cfg: LinesConfig,
}

impl LinesPlugin {
    pub fn new(cfg: LinesConfig) -> Self {
        Self { cfg }
    }
}

// The following traits are illustrative.
// Replace with the repo's actual Plugin trait definitions.

pub trait Plugin {
    fn id(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn run(&self, ctx: PluginContext) -> Result<IrBundle, PluginError>;
}

pub struct PluginContext {
    pub input_root: PathBuf,
    pub logical_root: String, // e.g., "artifact:/"
    pub limits: Limits,
}

pub struct Limits {
    pub max_files: usize,
    pub max_file_bytes: u64,
}

#[derive(Debug)]
pub struct PluginError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct IrBundle {
    pub ir_version: String,
    pub plugin: IrPluginMeta,
    pub artifact: IrArtifact,
    pub entities: Vec<IrEntity>,
    pub edges: Vec<IrEdge>,
    pub types: Vec<serde_json::Value>,
    pub constraints: Vec<serde_json::Value>,
    pub stats: serde_json::Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct IrPluginMeta {
    pub id: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct IrArtifact {
    pub kind: String,
    pub name: String,
    pub namespace: String,
    pub r#ref: String,
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct IrEntity {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub path: Option<String>,
    pub digest: Option<String>,
    pub attrs: serde_json::Value,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct IrEdge {
    pub id: String,
    pub relation: String,
    pub from: String,
    pub to: String,
    pub attrs: serde_json::Value,
}

impl Plugin for LinesPlugin {
    fn id(&self) -> &'static str { "lines" }
    fn version(&self) -> &'static str { "0.1.0" }

    fn run(&self, ctx: PluginContext) -> Result<IrBundle, PluginError> {
        // 1) Enumerate files deterministically (collect + sort)
        // 2) Enforce limits
        // 3) Emit entities + edges with stable IDs
        // 4) Return IR bundle

        Ok(IrBundle {
            ir_version: "v1".to_string(),
            plugin: IrPluginMeta { id: self.id().to_string(), version: self.version().to_string() },
            artifact: IrArtifact {
                kind: "repo".to_string(),
                name: "lines".to_string(),
                namespace: "example".to_string(),
                r#ref: "input".to_string(),
                labels: vec!["repo".to_string()],
            },
            entities: vec![],
            edges: vec![],
            types: vec![],
            constraints: vec![],
            stats: serde_json::json!({ "file_count": 0, "total_bytes": 0, "truncated": false }),
        })
    }
}
```

This skeleton highlights the key requirements:
- deterministic enumeration
- stable IDs
- bounded execution
- deterministic error handling

---

## 13) Checklist before submitting a plugin PR

- [ ] Stable IDs derived from normalized inputs
- [ ] No nondeterministic ordering
- [ ] Bounds enforced with deterministic failures
- [ ] Config is schema-validated and hashable
- [ ] Fixtures added
- [ ] Determinism tests pass (double-compile compare)
- [ ] `signia verify` passes on generated bundles
- [ ] Documentation updated (`builtin-plugins.md` or plugin docs)

---

## 14) Related documents

- Plugin spec: `docs/plugins/plugin-spec.md`
- Built-in plugins: `docs/plugins/builtin-plugins.md`
- Sandboxing: `docs/plugins/sandboxing.md`
- Schema v1: `docs/schemas/schema-v1.md`
- Manifest v1: `docs/schemas/manifest-v1.md`
- Proof v1: `docs/schemas/proof-v1.md`
- Determinism contract: `docs/determinism/determinism-contract.md`
