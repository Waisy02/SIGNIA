
# Manifest v1

This document specifies the **SIGNIA Manifest v1** format: the compilation context and integrity linkage emitted as `manifest.json` inside a SIGNIA bundle.

The manifest is designed to:
- enable reproducibility (record pinned inputs and toolchain versions)
- bind context to integrity anchors (schema hash, proof root)
- support caching and deduplication (input descriptor hashing)
- remain verifiable and canonicalizable

The manifest does not store the full input content. It records how to reproduce the compilation and how to verify the outputs.

---

## 1) Overview

A SIGNIA bundle contains:
- `schema.json`
- `manifest.json` (this document)
- `proof.json`

`manifest.json` captures:
- input descriptor (pinned ref or checksum)
- normalization policy
- plugin configuration hash
- compiler and plugin versions
- computed integrity links:
  - schema hash
  - proof root
  - optional manifest hash
- dependency references (other schema hashes)

---

## 2) Hashing and identity

### 2.1 Manifest hash
A manifest hash is recommended for integrity and caching.

Definition:
- `manifest_hash = H( domain("signia:manifest:v1") || canonical_bytes(manifest_hashed_view) )`

Important:
- The manifest may contain non-hashed metadata.
- The hashed view MUST be clearly defined.

Recommended approaches:
1. Keep manifest entirely hashed (simplest).
2. Split non-hashed metadata into a separate file (e.g., `manifest.meta.json`).
3. Use an explicit `non_hashed` section excluded from the hashed view.

This document defines an explicit `non_hashed` section.

### 2.2 Required header fields
`manifest.json` MUST contain:
- `manifest_version`: `"v1"`
- `hash_domain`: `"signia:manifest:v1"`
- `bundle`: integrity links and bundle IDs
- `input`: pinned input descriptor
- `toolchain`: compiler and plugin versions

---

## 3) JSON shape (top-level)

Top-level object:

```json
{
  "manifest_version": "v1",
  "hash_domain": "signia:manifest:v1",
  "bundle": { ... },
  "input": { ... },
  "toolchain": { ... },
  "policies": { ... },
  "dependencies": { ... },
  "non_hashed": { ... }
}
```

Canonicalization:
- JSON key ordering must be canonical (see `docs/determinism/canonicalization.md`).
- Arrays that represent sets must be sorted and deduplicated.

---

## 4) Bundle section

The `bundle` section binds compilation context to integrity anchors.

```json
"bundle": {
  "schema_hash": "<hex>",
  "proof_root": "<hex>",
  "manifest_hash": "<hex or null>",
  "schema_version": "v1",
  "proof_version": "v1",
  "created_by": {
    "compiler": "signia",
    "compiler_version": "<semver>",
    "build": {
      "git_commit": "<sha or null>",
      "build_profile": "release|debug",
      "target": "<triple or null>"
    }
  }
}
```

Rules:
- `schema_hash` MUST match the computed schema hash.
- `proof_root` MUST match the computed proof root.
- `manifest_hash` MAY be null until computed; if present, it MUST match the manifest hashed view.
- Versions MUST match the actual bundle files.

---

## 5) Input section

`input` describes the compiled artifact in a reproducible way.

```json
"input": {
  "source": {
    "kind": "repo|archive|file|url|stdin",
    "uri": "<string>",
    "resolved": {
      "commit": "<sha or null>",
      "checksum": "<hex or null>",
      "etag": "<string or null>"
    }
  },
  "descriptor": {
    "descriptor_version": "v1",
    "descriptor_hash": "<hex>",
    "fields": { ... }
  }
}
```

### 5.1 source.kind
- `repo`: VCS repository
- `archive`: tar/zip bundle
- `file`: local file
- `url`: remote URL
- `stdin`: data piped into compiler

### 5.2 source.uri
- a logical source reference
- must not include secrets
- should be stable for humans

### 5.3 source.resolved
Must contain immutable identifiers when available:
- `commit` for repos
- `checksum` for archives/files
- `etag` for HTTP resources (not sufficient alone; prefer checksums)

Rules:
- If compilation used a floating ref, the resolved immutable ref MUST be recorded.
- If no immutable ref can be resolved, compilation should fail by default (policy may override, but must be recorded).

### 5.4 descriptor (input descriptor hash)
The descriptor hash is used for caching and deduplication.

Definition:
- `descriptor_hash = H( domain("signia:input-descriptor:v1") || canonical_bytes(descriptor.fields) )`

Descriptor fields SHOULD include:
- plugin name and version
- normalization policy version
- relevant plugin config values (hashed)
- resolved immutable input identifiers
- any compile flags that affect outputs

---

## 6) Toolchain section

`toolchain` records versions for reproducibility.

```json
"toolchain": {
  "compiler": {
    "name": "signia",
    "version": "<semver>",
    "hash_function": "sha256|blake3",
    "canonicalization": {
      "rules_version": "v1"
    }
  },
  "plugins": [
    {
      "name": "<plugin name>",
      "version": "<semver>",
      "config_hash": "<hex>",
      "notes": "<string or null>"
    }
  ]
}
```

Rules:
- `plugins` MUST be sorted by `(name, version, config_hash)`.
- `config_hash` is computed from canonical plugin config (if applicable).

---

## 7) Policies section

`policies` describes operational rules used during compilation.

```json
"policies": {
  "normalization": {
    "policy_version": "v1",
    "path_root": "artifact:/",
    "newline": "lf",
    "encoding": "utf-8",
    "symlinks": "deny|resolve-within-root",
    "network": "deny|allow-pinned-only"
  },
  "limits": {
    "max_total_bytes": 268435456,
    "max_file_bytes": 10485760,
    "max_files": 20000,
    "max_depth": 64,
    "max_nodes": 200000,
    "max_edges": 400000,
    "timeout_ms": 300000
  }
}
```

Rules:
- Limits MUST be deterministic and explicitly recorded if they affect output (e.g., truncation behavior).
- If truncation is possible, it MUST be described and reflected in the schema (e.g., “partial” flags).

---

## 8) Dependencies section

Dependencies are references to other schemas by hash.

```json
"dependencies": {
  "schemas": [
    {
      "schema_hash": "<hex>",
      "relation": "imports|extends|includes|references",
      "notes": "<string or null>"
    }
  ]
}
```

Rules:
- `schemas` MUST be sorted by `(relation, schema_hash)`.
- Dependency references must not rely on human-readable names for integrity.

---

## 9) Non-hashed section

`non_hashed` contains metadata that must not affect verification.

```json
"non_hashed": {
  "display": {
    "title": "<string>",
    "description": "<string>"
  },
  "annotations": {
    "publisher_label": "<string or null>",
    "tags": ["<string>", "..."]
  }
}
```

Rules:
- Verification MUST ignore `non_hashed`.
- Clients MUST treat `non_hashed` as untrusted.
- `tags` should be sorted and deduplicated for cleanliness, even though non-hashed.

---

## 10) Canonicalization requirements (normative)

Manifest v1 is valid only if canonicalization requirements are met.

Normative rules:
- JSON key ordering: lexicographic by Unicode code point
- UTF-8 output
- No insignificant whitespace
- Sorted arrays where specified:
  - plugins by `(name, version, config_hash)`
  - dependency schemas by `(relation, schema_hash)`
  - any set-like arrays sorted and deduplicated

---

## 11) Validation rules (normative)

A manifest MUST fail validation if:
- `manifest_version` is not `"v1"`
- `hash_domain` is incorrect
- `bundle.schema_hash` is missing
- `bundle.proof_root` is missing
- `input.source.kind` is missing
- required resolved identifiers are missing under the configured network policy
- ordering constraints are violated (when validating canonical form)

---

## 12) Security considerations

- Do not include secrets in `uri`, `notes`, or `non_hashed`.
- Ensure `descriptor_hash` includes all values that affect outputs.
- Ensure policies are explicit to avoid “it compiled differently” ambiguity.
- Treat nondeterminism as a security issue.

---

## 13) Example (minimal)

```json
{
  "manifest_version": "v1",
  "hash_domain": "signia:manifest:v1",
  "bundle": {
    "schema_hash": "0123...abcd",
    "proof_root": "4567...cdef",
    "manifest_hash": null,
    "schema_version": "v1",
    "proof_version": "v1",
    "created_by": {
      "compiler": "signia",
      "compiler_version": "0.1.0",
      "build": {
        "git_commit": null,
        "build_profile": "release",
        "target": null
      }
    }
  },
  "input": {
    "source": {
      "kind": "repo",
      "uri": "repo://example/openapi",
      "resolved": {
        "commit": "deadbeef...",
        "checksum": null,
        "etag": null
      }
    },
    "descriptor": {
      "descriptor_version": "v1",
      "descriptor_hash": "89ab...0123",
      "fields": {
        "plugin": "openapi",
        "plugin_version": "0.1.0",
        "normalization_policy": "v1"
      }
    }
  },
  "toolchain": {
    "compiler": {
      "name": "signia",
      "version": "0.1.0",
      "hash_function": "sha256",
      "canonicalization": { "rules_version": "v1" }
    },
    "plugins": []
  },
  "policies": {
    "normalization": {
      "policy_version": "v1",
      "path_root": "artifact:/",
      "newline": "lf",
      "encoding": "utf-8",
      "symlinks": "deny",
      "network": "deny"
    },
    "limits": {
      "max_total_bytes": 268435456,
      "max_file_bytes": 10485760,
      "max_files": 20000,
      "max_depth": 64,
      "max_nodes": 200000,
      "max_edges": 400000,
      "timeout_ms": 300000
    }
  },
  "dependencies": { "schemas": [] },
  "non_hashed": { "display": { "title": "Example", "description": "" }, "annotations": { "publisher_label": null, "tags": [] } }
}
```

---

## 14) Related documents

- Schema spec: `docs/schemas/schema-v1.md`
- Proof spec: `docs/schemas/proof-v1.md`
- Canonicalization: `docs/determinism/canonicalization.md`
- Hashing: `docs/determinism/hashing.md`
