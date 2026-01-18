
# Built-in Plugins

This document describes the built-in plugins shipped with SIGNIA, their supported inputs, IR output shape, configuration, limits, and determinism expectations.

Built-in plugins are maintained in this repository and are intended to be:
- safe by default (no network, no shell execution)
- deterministic (stable outputs for stable inputs)
- bounded (resource limits and predictable failures)
- composable (consistent entity/edge/type conventions)

If you add or modify a built-in plugin, update:
- this document
- fixtures under `fixtures/`
- determinism tests and expected hashes

---

## 1) Plugin catalog

SIGNIA built-in plugins (recommended baseline):

1. `repo` — Repository structure + dependency graph
2. `openapi` — OpenAPI/Swagger structure extraction
3. `jsonschema` — JSON Schema type graph extraction
4. `markdown` — Document structure and references
5. `dataset` — Dataset / table structure extraction
6. `workflow` — Workflow DAG extraction

Plugins share the IR v1 contract defined in:
- `docs/plugins/plugin-spec.md`

---

## 2) Common conventions (all plugins)

### 2.1 IDs
Entities:
- `ent:<kind>:<stable-key>`

Edges:
- `edge:<relation>:<from>:<to>:<tiebreaker>`

Types:
- `type:<kind>:<stable-id>`

Constraints:
- `c:<kind>:<stable-id>`

Stable keys MUST be derived from normalized inputs (paths/URIs/identifiers) and MUST NOT include:
- timestamps
- randomness
- host absolute paths

### 2.2 Tags
Tags are optional but recommended:
- `code`, `spec`, `doc`, `data`, `workflow`, `public`, `private`

Tags MUST be sorted and deduplicated.

### 2.3 Normalized paths
When a plugin emits a `path`:
- it MUST be a logical normalized path using `/`
- it MUST use the normalization root from manifest (e.g., `artifact:/`)

### 2.4 Bounds and deterministic failure
All plugins enforce:
- max entities
- max edges
- max depth/recursion where relevant

Exceeding a limit MUST:
- fail deterministically
- produce a stable error code

---

## 3) repo plugin

### 3.1 Purpose
Extract repository structure and dependency relationships.

### 3.2 Supported inputs
- `repo` directory input
- archive containing a repository (tar/zip)

### 3.3 Outputs
Entities:
- `module` (source files)
- `package` (package manifests)
- `dir` (optional directory nodes)

Edges:
- `contains` (dir → file)
- `imports` (module → module)
- `depends_on` (package → package)
- `references` (file → file) for dynamic requires or includes

### 3.4 Configuration
Example config:

```json
{
  "include_globs": ["**/*"],
  "exclude_globs": ["**/.git/**", "**/node_modules/**", "**/target/**"],
  "languages": ["typescript", "javascript", "rust"],
  "parse_imports": true,
  "emit_directories": false,
  "content_digests": true,
  "max_file_bytes": 1048576,
  "max_files": 20000
}
```

Rules:
- glob matching must be deterministic
- file enumeration must be stable sorted by normalized path

### 3.5 Determinism requirements
- stable file ordering
- stable import resolution rules:
  - prefer path-based resolution
  - avoid reading environment variables
- avoid reading user-specific config files outside the repo root

### 3.6 Limits
- max files
- max file size
- max entities/edges

---

## 4) openapi plugin

### 4.1 Purpose
Extract structural representation from OpenAPI v3 (and optionally v2).

### 4.2 Supported inputs
- OpenAPI JSON or YAML file
- repo containing OpenAPI definitions

### 4.3 Outputs
Entities:
- `endpoint` (method + route)
- `schema` (component schemas)
- `parameter`
- `response`
- `request_body`

Edges:
- `defines` (spec → schema)
- `uses` (endpoint → schema)
- `has_param` (endpoint → parameter)
- `returns` (endpoint → response)

Types:
- type definitions derived from schemas, if enabled

### 4.4 Configuration
Example config:

```json
{
  "input_format": "auto",
  "emit_types": true,
  "emit_examples": false,
  "resolve_refs": true,
  "content_digests": false,
  "max_schema_nodes": 200000
}
```

### 4.5 Determinism requirements
- YAML parsing must be stable for the same bytes
- reference resolution must be deterministic
- ordering of paths/components must be canonicalized

### 4.6 Limits
- max schema nodes
- max endpoints
- max depth of `$ref` resolution

---

## 5) jsonschema plugin

### 5.1 Purpose
Extract type graphs and references from JSON Schema.

### 5.2 Supported inputs
- JSON Schema file
- repo containing schema files

### 5.3 Outputs
Entities:
- `schema_doc`
- `type`

Edges:
- `refers_to` (type → type)
- `contains_type` (doc → type)

Types:
- JSON Schema types and unions

### 5.4 Configuration
Example config:

```json
{
  "draft": "auto",
  "resolve_refs": true,
  "emit_union_types": true,
  "max_depth": 64,
  "max_types": 200000
}
```

### 5.5 Determinism requirements
- `$ref` resolution must be stable and pinned to inputs
- no network resolution by default
- stable ordering of properties and required lists

---

## 6) markdown plugin

### 6.1 Purpose
Extract document structure: headings, sections, links, references.

### 6.2 Supported inputs
- Markdown files
- repo containing documentation

### 6.3 Outputs
Entities:
- `doc`
- `section`
- `link`

Edges:
- `contains` (doc → section)
- `links_to` (section → link target)
- `references` (doc/section → entity if resolvable)

### 6.4 Configuration
Example config:

```json
{
  "emit_links": true,
  "resolve_relative_links": true,
  "max_sections": 200000,
  "max_doc_bytes": 2097152
}
```

### 6.5 Determinism requirements
- stable parsing based on file bytes
- stable heading ID generation rules (documented)
- stable link normalization

---

## 7) dataset plugin

### 7.1 Purpose
Extract dataset structure:
- tables
- columns
- types
- constraints (optional)

### 7.2 Supported inputs
- CSV, Parquet (optional), JSONL (optional)
- data directories (bounded)

### 7.3 Outputs
Entities:
- `dataset`
- `table`
- `column`

Edges:
- `contains` (dataset → table, table → column)
- `relates_to` (column → column) if foreign-key inference enabled

Constraints:
- primary key suggestions
- uniqueness, nullability
- inferred types

### 7.4 Configuration
Example config:

```json
{
  "format": "csv",
  "infer_types": true,
  "infer_constraints": false,
  "max_rows_sampled": 50000,
  "max_columns": 10000,
  "max_bytes": 268435456
}
```

### 7.5 Determinism requirements
- sampling strategy must be deterministic (e.g., first N rows only)
- type inference must be deterministic
- avoid any randomness

---

## 8) workflow plugin

### 8.1 Purpose
Extract workflow DAGs:
- steps
- dependencies
- inputs/outputs (structural)

### 8.2 Supported inputs
- YAML workflow definitions (GitHub Actions, CI configs)
- JSON workflow descriptions
- repo containing workflows

### 8.3 Outputs
Entities:
- `workflow`
- `job`
- `step`
- `artifact`

Edges:
- `contains` (workflow → job → step)
- `depends_on` (job → job)
- `produces` (step → artifact)
- `consumes` (step → artifact)

### 8.4 Configuration
Example config:

```json
{
  "format": "auto",
  "emit_steps": true,
  "emit_artifacts": true,
  "max_jobs": 20000,
  "max_steps": 200000
}
```

### 8.5 Determinism requirements
- stable parsing and ordering
- stable normalization of IDs and names
- no environment variable expansion unless explicitly enabled and recorded

---

## 9) Plugin output mapping to Schema v1

Built-in plugins emit IR v1 which SIGNIA maps into Schema v1:
- IR entities → schema graph.entities
- IR edges → schema graph.edges
- IR types → schema types.definitions
- IR constraints → schema constraints.rules

Mapping MUST:
- preserve stable IDs
- preserve relations and attributes
- apply canonical ordering in the final schema

---

## 10) Fixture recommendations

For each built-in plugin:
- include at least one realistic fixture
- include edge cases:
  - empty input
  - deeply nested structures (bounded)
  - invalid inputs and deterministic errors

Recommended fixture layout:
- `fixtures/<plugin>/<name>/input/`
- `fixtures/<plugin>/<name>/expected/schema.json`
- `fixtures/<plugin>/<name>/expected/manifest.json`
- `fixtures/<plugin>/<name>/expected/proof.json`
- `fixtures/<plugin>/<name>/expected/hashes.json`

---

## 11) Related documents

- Plugin spec: `docs/plugins/plugin-spec.md`
- Schema v1: `docs/schemas/schema-v1.md`
- Canonicalization: `docs/determinism/canonicalization.md`
- Hashing: `docs/determinism/hashing.md`
