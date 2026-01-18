
# Reproducible Builds

This document describes reproducible build practices for SIGNIA. It covers:
- deterministic compilation outputs (SIGNIA bundles)
- deterministic software builds (binaries and containers)
- CI/CD practices for repeatable releases

SIGNIA’s primary reproducibility goal is **verifiable structure outputs**:
same input → same canonical bytes → same hashes → same proofs.

Binary reproducibility is strongly recommended, but may be bounded by toolchain realities. This document provides a practical, “production useful” approach.

---

## 1) Definitions

### Reproducible output (SIGNIA bundles)
Given the same input and policies, the emitted bundle (`schema.json`, `manifest.json`, `proof.json`) is identical byte-for-byte.

### Reproducible build (binaries)
Given the same source revision and toolchain, the produced binaries are identical (or at least cryptographically equivalent) across builders.

### Deterministic build
Build behavior does not depend on time, environment, or network variation.

---

## 2) Goals

1. Ensure deterministic bundle generation and verification in CI.
2. Pin toolchains and dependencies to prevent drift.
3. Produce release artifacts that users can verify with checksums.
4. Minimize supply chain risk by enforcing locked installs and audits.
5. Make local developer builds consistent with CI.

---

## 3) Toolchain pinning

### 3.1 Rust toolchain
Use `rust-toolchain.toml` to pin:
- Rust channel/version
- required components (clippy, rustfmt)
- targets if needed

CI should use:
- `rustup show`
- `cargo --version`
- `rustc --version`

### 3.2 Node toolchain
Pin Node and package manager versions:
- `.nvmrc` or `.node-version` or `package.json` engines
- `corepack` for pnpm/yarn if used

CI should print:
- `node --version`
- `pnpm --version` (or yarn/npm)

### 3.3 Solana toolchain (if registry program is included)
Pin Solana and Anchor versions:
- document pinned versions in `docs/onchain/` and CI workflow
- prefer installation scripts that accept explicit version numbers
- record versions in release notes

---

## 4) Dependency locking

### 4.1 Rust
- Commit `Cargo.lock` for binaries and workspace.
- Build and test with:
  - `cargo build --locked`
  - `cargo test --locked`

Avoid unpinned Git dependencies. If unavoidable:
- pin to commit SHA
- document in the PR

### 4.2 Node
- Commit lockfile (`pnpm-lock.yaml`, `package-lock.json`, or `yarn.lock`).
- Enforce locked installs:
  - pnpm: `pnpm install --frozen-lockfile`
  - npm: `npm ci`
  - yarn: `yarn install --immutable`

---

## 5) Build environment control

### 5.1 Avoid network dependence during builds
- do not fetch arbitrary URLs during build steps
- if external artifacts are required, pin checksums and store them as build inputs

### 5.2 Time and locale
In CI and release builds:
- set timezone to UTC
- set locale consistently
- avoid embedding build timestamps in binaries where possible

Example environment recommendations:
- `TZ=UTC`
- `LC_ALL=C`
- `LANG=C`

---

## 6) Deterministic SIGNIA bundle generation

SIGNIA output determinism must hold across:
- repeated runs
- different machines
- different operating systems (as supported)

### 6.1 Deterministic input resolution
- prefer pinned refs (commit SHA, checksum)
- avoid floating branches and “latest” URLs
- if floating refs are allowed, resolve them to pinned refs and record in manifest

### 6.2 Deterministic normalization
- normalize path separators and root mapping
- normalize newlines to LF for hashed domains
- remove or normalize timestamps
- enforce stable file ordering

### 6.3 Deterministic canonicalization
- canonical JSON encoding
- stable ordering rules for all collections
- explicit versioning for schema/manifest/proof formats

### 6.4 Deterministic proof construction
- deterministic leaf set and leaf ordering
- deterministic Merkle tree construction rules
- domain-separated hashing

---

## 7) CI enforcement patterns

### 7.1 Compile twice, compare outputs
For determinism fixtures, CI should:
1. compile fixture input → bundle A
2. compile the same fixture input → bundle B
3. compare:
   - canonical bytes of `schema.json`
   - schema hash
   - proof root

The CI must fail if outputs differ.

### 7.2 Golden fixtures
Maintain committed fixtures:
- `fixtures/<name>/input/`
- `fixtures/<name>/expected/schema.json`
- `fixtures/<name>/expected/manifest.json` (hashed view)
- `fixtures/<name>/expected/proof.json` (or proof root)
- `fixtures/<name>/expected/hashes.json`

CI verifies the compiler output matches the committed expected outputs.

### 7.3 Verification as a gate
CI should always run:
- `signia verify <bundle>`
to ensure:
- canonical bytes match hashes
- manifests link correctly
- proof roots match

---

## 8) Release artifacts and verification

### 8.1 Checksums
Every release should publish:
- binaries for supported platforms
- a `SHA256SUMS` file for artifacts
- optionally a signature over `SHA256SUMS`

Users can verify:
- artifact hash matches checksum
- checksum file signature matches maintainer key (optional)

### 8.2 Provenance and attestations (recommended)
If adopting provenance:
- generate build provenance (e.g., SLSA-style)
- attach attestations to releases
- ensure they reference the exact source revision

This is optional but recommended for infrastructure projects.

### 8.3 Container images
For Docker images:
- build in CI from pinned versions
- tag images with version and commit SHA
- publish digest references
- consider generating SBOM

---

## 9) Practical Docker reproducibility

### 9.1 Deterministic Docker builds
Recommendations:
- use fixed base image tags (or digests)
- avoid `apt-get upgrade` without version pinning
- clean apt caches
- avoid build steps that depend on current time or network variability
- build with BuildKit and deterministic settings when possible

### 9.2 Example practices
- pin base images by digest
- pin dependencies using lockfiles copied before source
- minimize layers and include only required artifacts

---

## 10) Local developer workflows

Developers should be able to reproduce CI results.

Recommendations:
- provide `make` targets or scripts:
  - `make fmt`
  - `make lint`
  - `make test`
  - `make build`
  - `make e2e`
- provide a `devcontainer` or Docker-based dev environment (optional)
- document tool versions and install methods

---

## 11) Reproducible builds limitations

Binary reproducibility can be impacted by:
- toolchain differences
- linker behavior
- embedded metadata and debug info
- platform-specific compilation

Mitigation strategy:
- pin toolchains tightly
- disable embedding timestamps where possible
- prefer release builds with stable flags
- focus on reproducible SIGNIA bundle outputs as the primary trust anchor

---

## 12) Security considerations

Reproducibility improves security by:
- enabling independent verification of outputs
- reducing trust in single build operators
- making supply chain attacks harder to hide

However:
- reproducibility does not prevent malicious code from being merged
- it must be paired with reviews, audits, and CI checks

---

## 13) Checklist

### For maintainers
- [ ] Toolchains pinned (Rust/Node/Solana)
- [ ] Lockfiles committed and enforced
- [ ] CI compiles determinism fixtures twice and compares outputs
- [ ] Golden fixtures exist for core plugins
- [ ] `verify` is a mandatory CI gate
- [ ] Releases publish checksums
- [ ] Containers are built from pinned inputs

### For contributors
- [ ] Do not introduce unpinned dependencies
- [ ] Update fixtures when changing canonicalization rules
- [ ] Add tests for determinism-sensitive changes
- [ ] Run `verify` locally before submitting PRs

---

## 14) Summary

SIGNIA reproducibility is anchored on deterministic, canonical structure outputs:
- stable canonical bytes
- stable hashes and proof roots
- independent verification

Reproducible software builds are strongly recommended and supported through pinned toolchains, locked dependencies, CI gates, and checksum-based releases.
