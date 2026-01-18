
# FAQ

This FAQ answers common questions about SIGNIA: what it is, what it is not, and how to use it correctly in production settings.

---

## 1) What is SIGNIA?

SIGNIA is a structure-level on-chain compilation system. It converts real-world, already-formed artifacts (repos, specs, schemas, workflows, configs) into **canonical, verifiable, composable structural representations**.

SIGNIA focuses on structure, not execution. It produces deterministic bundles (schema/manifest/proof) that can be verified locally and optionally anchored on Solana.

---

## 2) What problem does SIGNIA solve?

SIGNIA provides a stable structural anchor for artifacts that otherwise drift over time.

It solves:
- “Which exact interface contract did we rely on?”
- “What configuration graph was approved at that point in time?”
- “Can we verify the published structure independently?”
- “Can we reference this structure on-chain by hash?”

---

## 3) Is SIGNIA a smart contract framework?

No.

SIGNIA does not execute application logic, deploy code as programs, or provide a runtime.  
It compiles structure into a verifiable form and optionally registers identifiers on-chain.

---

## 4) Does SIGNIA store content on-chain?

No.

The on-chain registry stores minimal identifiers (schema hash, optional manifest hash/proof root, and minimal metadata). It does not store full schemas or arbitrary content.

Schemas and bundles are distributed off-chain, while the chain provides an anchor and a stable handle.

---

## 5) What does “structure-level” mean?

It means SIGNIA models:
- entities (modules, endpoints, fields)
- relationships (dependencies, references)
- types and constraints
- invariants required for verification

It does not model:
- runtime behavior
- execution semantics
- operational outcomes

---

## 6) What outputs do I get from SIGNIA?

A deterministic bundle containing:
- `schema.json`: canonical structural schema (versioned)
- `manifest.json`: compilation context and integrity links
- `proof.json`: cryptographic verification material (e.g., Merkle root)

Optionally, the schema hash can be registered on Solana.

---

## 7) How does verification work?

Verification checks:
- `schema.json` canonical bytes match the schema hash
- `manifest.json` references match schema hash and proof root
- `proof.json` root matches the leaf set and deterministic ordering rules
- all versions and domains match expected specifications

Verification does not require trust in the compiler runner; any party can recompute and verify.

---

## 8) What guarantees does SIGNIA provide?

The primary guarantees are:
- determinism: same input → same output
- canonical encoding: stable bytes for hashing
- verifiable integrity: hashes and proofs bind outputs
- composability: schemas can be referenced and combined by hash

SIGNIA does not guarantee the real-world “truthfulness” of an input. It guarantees reproducibility and integrity of the compilation.

---

## 9) Can I compile a GitHub repository?

Yes, if you use a repo plugin and provide an immutable reference (commit SHA recommended).  
For determinism, avoid floating refs like `main` without pinning.

A correct compilation input is something like:
- repo URL + commit SHA
- or a tarball with a stable checksum

---

## 10) Can I compile private repositories?

Yes, but it requires careful handling:
- access must be explicit (tokens/credentials)
- inputs must remain pinned and reproducible
- secrets must never be stored in bundles
- verification must be possible by authorized parties

For public registries, prefer public inputs or publish only hashes while distributing bundles privately.

---

## 11) Does SIGNIA require Solana?

No.

SIGNIA can be used fully off-chain:
- compile
- verify
- store and distribute bundles

Solana is used as an optional registry to anchor schema identifiers and enable on-chain referencing.

---

## 12) Why Solana for the registry?

Solana offers:
- low-latency, low-cost transactions
- a mature program model for registries
- broad ecosystem integration potential

However, the architecture keeps the registry minimal so the system remains portable conceptually.

---

## 13) How does SIGNIA handle schema evolution?

SIGNIA treats evolution as a relationship between immutable schema hashes.

Evolution can be expressed via:
- version links (supersedes, compatible_with, forks_from)
- explicit compatibility metadata in manifests
- separate registry records per schema hash

A new schema version is a new hash. Links define the relationship.

---

## 14) Can outputs be indexed and searched?

Yes, but indexing is off-chain.

The registry is not a full-text search engine. It provides stable identifiers.  
A separate indexing service (optional) can:
- ingest bundles
- build search indexes
- support rich queries over entities/edges

---

## 15) Is SIGNIA an oracle?

Not in the typical price/data oracle sense.

SIGNIA provides a verifiable structural representation. It can be used as a building block for oracle-like systems that need to reference immutable interface or rule structure.

---

## 16) How do plugins affect determinism?

Plugins must be deterministic by contract.

A plugin must:
- normalize inputs consistently
- avoid environment-dependent behavior
- avoid unpinned network calls
- define ordering and identity rules explicitly
- produce IR that can be canonicalized deterministically

Plugins should be versioned and their version recorded in the manifest.

---

## 17) What about AI in SIGNIA?

AI can be used in a controlled, deterministic way, for example:
- classification into known categories
- mapping to predefined structural types
- extracting structured fields when the output is fully normalized

If AI introduces nondeterminism, it must be confined to **non-hashed metadata domains** or replaced by deterministic rules.

Determinism is the primary constraint.

---

## 18) Can I trust bundles produced by someone else?

You should not trust them blindly. You should verify.

The verification model is designed so:
- anyone can recompute hashes
- proofs bind schema and manifest
- mismatches are detectable

This is the intended trust posture.

---

## 19) What are common pitfalls?

- Using unpinned inputs (floating branches, mutable URLs)
- Allowing network access without content addressing
- Including timestamps or random IDs in hashed domains
- Relying on filesystem iteration order
- Mixing hashed and non-hashed metadata without clear boundaries

---

## 20) Is there a token?

No.

There is currently **no token issued**.

If you see token claims, treat them as unrelated to the SIGNIA repository unless explicitly documented in official releases and channels.

---

## 21) How do I contribute?

Open a PR with:
- a clear summary
- tests for determinism or compatibility
- documentation updates where required

Start with:
- documentation improvements
- new plugin adapters
- better fixtures and verification coverage

---

## 22) Where do I start if I want to integrate SIGNIA?

Start off-chain:
1. Compile an artifact and generate a bundle.
2. Verify the bundle locally.
3. Integrate by consuming the schema hash and bundle format in your system.
4. Optionally register the schema hash on Solana for on-chain referencing.

---

## 23) What is the expected stability policy?

SIGNIA aims to keep stable:
- bundle contracts (schema/manifest/proof) once published
- hash domains and canonical encoding rules
- registry instruction semantics

Anything that changes those must be versioned and documented.

---

## 24) How do I report security issues?

Please do not open public issues for sensitive vulnerabilities.

Use the security policy in `SECURITY.md` to report vulnerabilities responsibly.

---

## 25) Where can I see examples?

Check:
- `examples/` for runnable inputs and expected outputs
- `docs/` for formal specifications
- CI workflows for the expected build and verification commands

---

## Quick link index

- Overview: `docs/overview.md`
- Architecture: `docs/architecture.md`
- Glossary: `docs/glossary.md`
- CLI: `docs/cli/usage.md`
- Determinism: `docs/data-model/determinism-rules.md`
- On-chain registry: `docs/onchain/registry-program.md`
