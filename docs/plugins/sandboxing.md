
# Sandboxing

This document describes the sandboxing model for SIGNIA plugins and compilation runs. Sandboxing is a security requirement because plugins process untrusted inputs and may include parsers that could be attacked.

The sandbox model aims to:
- prevent filesystem escape
- prevent secret exfiltration
- prevent uncontrolled network access
- enforce resource limits
- keep deterministic behavior

---

## 1) Threats addressed

Sandboxing reduces risk from:
- malicious archives (path traversal, symlink tricks)
- parser crashes and memory abuse
- arbitrary code execution via vulnerable dependencies
- secrets exfiltration via CI environment leakage
- network-based dependency confusion and data exfiltration

This doc complements:
- `docs/security/security/threat-model.md`
- `docs/security/security/trust-boundaries.md`
- `docs/security/security/supply-chain.md`

---

## 2) Default sandbox policy (safe mode)

By default, SIGNIA runs in **safe mode**:

- No network access
- Read-only access to input root
- Write access only to the output directory
- No subprocess execution
- No access to environment secrets (best-effort)
- Strict resource limits

Safe mode is intended for:
- CI
- deterministic fixtures
- untrusted inputs from users

---

## 3) Sandbox boundary and trust model

### 3.1 Trust boundary
Untrusted:
- input artifacts (repos, archives, URLs, stdin)
- plugin code and dependencies (treated as potentially vulnerable)
- output bundles until verified

Trusted:
- SIGNIA core verification logic
- canonicalization + hashing rules
- bundled specs and fixtures

### 3.2 Execution boundary
Sandboxing should isolate:
- filesystem access
- network access
- process execution
- CPU and memory usage

---

## 4) Filesystem sandbox

### 4.1 Logical input root
All filesystem reads must be scoped to an input root.

Rules:
- deny absolute host paths in hashed domains
- normalize paths and enforce containment
- reject traversal attempts (`..` escape)
- deny symlinks by default

### 4.2 Symlink policy
Default:
- `symlinks: deny`

If enabled:
- resolve symlinks only within input root
- canonicalize resolved path
- reject if it escapes root

### 4.3 Archive extraction rules
Archives are dangerous; extraction must be defensive:
- reject entries with absolute paths
- reject `..` traversal
- apply size limits
- limit number of files
- treat symlinks as policy-controlled (default deny)
- canonicalize file permissions to avoid behavior differences

### 4.4 Read-only input
Input directories are mounted read-only where possible.
Plugins must not mutate inputs.

---

## 5) Network sandbox

### 5.1 Default: no network
Network access is disabled by default.

### 5.2 Pinned-only network mode (opt-in)
If network access is enabled, it MUST be pinned-only:
- all fetched resources must have checksums or immutable refs
- HTTP requests must have strict timeouts
- enforce max response size
- record resolved identifiers in `manifest.json`

Forbidden:
- fetching “latest” without pinning
- following redirects to unknown hosts without pinning
- using environment proxies without explicit configuration

### 5.3 Allowed host allowlist (optional)
For production deployments that require limited network:
- maintain an allowlist of hosts
- version the allowlist
- record allowlist version in manifest policies

---

## 6) Process execution sandbox

### 6.1 Default: no subprocesses
Plugins must not run subprocesses in safe mode.

### 6.2 Unsafe mode (explicit)
If subprocess execution is required:
- it must be enabled with an explicit `--unsafe` flag
- the manifest must record unsafe mode
- the CLI should warn loudly
- CI should disallow unsafe mode unless explicitly configured

If enabled, further restrict:
- allowlist commands
- run with reduced privileges
- capture deterministic outputs only
- timeouts for each subprocess

---

## 7) Resource limits

Sandboxing must enforce resource limits to prevent DoS:
- CPU time limits
- wall-clock timeouts
- memory limits
- file size limits
- file count limits
- recursion depth limits
- max entities/edges emitted

### 7.1 Deterministic failures
When limits are exceeded, failures must be deterministic:
- stable error codes
- stable messages
- no host-dependent paths in error output

### 7.2 Recommended default limits (illustrative)
- max total input bytes: 256 MiB
- max file bytes: 10 MiB
- max files: 20,000
- max nodes: 200,000
- max edges: 400,000
- max depth: 64
- timeout: 5 minutes

Record limits in `manifest.json` under `policies.limits`.

---

## 8) Environment isolation

### 8.1 Secrets and environment variables
Principle:
- plugins must not depend on environment variables for outputs

Recommendations:
- clear or restrict environment passed to plugins
- do not expose CI secrets to untrusted PR builds
- avoid printing environment variables in logs

If environment variables are used for configuration:
- require explicit allowlist
- record the allowlist in manifest policies
- ensure they do not affect hashed outputs unless explicitly recorded

### 8.2 Locale and timezone
Set consistent values:
- `TZ=UTC`
- `LC_ALL=C`
- `LANG=C`

This reduces nondeterminism.

---

## 9) Sandboxing implementation options

Different deployments can choose different sandbox mechanisms. This doc is mechanism-agnostic but provides recommended options.

### 9.1 OS-level sandboxing
Options:
- Linux: namespaces + seccomp + cgroups
- macOS: sandbox-exec (limited), containers
- Windows: job objects / containers

### 9.2 Container sandboxing
Run SIGNIA in a container:
- read-only input mount
- writable output mount
- no network (`--network=none`)
- limited CPU/memory

This is often the easiest reliable sandbox.

### 9.3 In-process sandboxing
In-process restrictions are weaker but still useful:
- strict path containment checks
- deny network calls at application level
- strict timeouts and memory-aware parsing
- no dynamic code loading

---

## 10) Determinism interactions

Sandboxing must not introduce nondeterminism.

Rules:
- do not use nondeterministic schedulers to decide ordering
- if concurrency is used, merge results with stable sorting
- avoid caching that depends on current time
- avoid random backoff algorithms affecting output

If network is enabled:
- require pinning and record resolved refs
- caches must not change results

---

## 11) CI and fork safety

GitHub Actions guidance:
- do not run unsafe mode on untrusted PRs
- do not expose secrets to forked PR workflows
- prefer `pull_request` events without secrets
- avoid `pull_request_target` unless hardened

Use dependency review and CodeQL.

---

## 12) Recommended CLI surface (for implementation)

Recommended flags (illustrative):
- `--safe` (default)
- `--unsafe` (enables subprocess and/or network if configured)
- `--network=deny|pinned`
- `--symlinks=deny|resolve-within-root`
- `--limits=<profile>` or explicit limit flags

All selected policies should be recorded in `manifest.json`.

---

## 13) Testing sandbox correctness

Required tests:
- archive traversal tests (`../` entries)
- symlink escape tests
- network disabled tests
- deterministic limit failures
- file permission normalization tests (where relevant)

Recommended:
- fuzz archive parser and input normalizer
- run tests inside containers with `--network=none`

---

## 14) Summary

SIGNIA sandboxing is designed to make plugin execution:
- safe by default
- deterministic
- bounded
- verifiable

The strongest practical approach is container-based isolation with no network and read-only mounts, combined with strict normalization and canonicalization rules.
