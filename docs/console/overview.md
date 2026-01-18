
# Console Overview

SIGNIA Console is the interactive operator and user interface for the SIGNIA system. It provides:
- a query surface for schema hashes, bundles, and on-chain anchors
- a guided interface for compilation and verification
- a project-aware assistant that explains concepts and deployment steps
- operational dashboards (jobs, logs, limits) for self-hosted deployments

The Console is designed to be embedded on a website or run as a standalone app.

Related docs:
- `docs/api/openapi.yaml`
- `docs/api/auth.md`
- `docs/cli/usage.md`
- `docs/onchain/registry-program.md`

---

## 1) What the Console is

The Console is a thin client over the SIGNIA API.

It does not implement compilation itself. Instead, it:
- submits compilation jobs to the API
- uploads inputs
- polls job status
- downloads and verifies bundles
- visualizes schema graphs and proof paths
- optionally triggers on-chain publishing flows

This keeps the UI portable and prevents duplication of determinism logic.

---

## 2) Primary user journeys

### 2.1 "Compile an artifact"
A user selects:
- a plugin (repo, openapi, dataset, etc.)
- an input (upload or pinned URL)
- a policy profile (safe by default)

The Console:
- creates a job
- uploads the input
- starts compilation
- shows output hashes (schema hash, proof root)
- offers download of bundle archive

### 2.2 "Verify an artifact"
A user uploads a bundle or pastes a schema hash.

The Console:
- verifies the bundle locally (optional) and via API
- shows each verification check
- shows mismatches with precise details
- provides copyable hashes

### 2.3 "Lookup on-chain anchor"
A user provides:
- schema hash (hex)
- network (devnet/mainnet)
- program id (optional override)

The Console:
- derives PDA
- fetches the on-chain record
- compares on-chain hashes with local bundle
- shows publisher and status

### 2.4 "Ask the assistant"
Users ask:
- what SIGNIA is
- what hashes mean
- how to deploy the API
- how to publish anchors on-chain
- how to write a plugin

The assistant responds using:
- project docs
- deterministic explanations
- copy/paste commands

The assistant must never hallucinate chain state and should prefer verifying facts via API queries.

---

## 3) Console components

### 3.1 Navigation
Core sections:
- Home
- Compile
- Verify
- Schemas
- Bundles
- On-chain
- Docs
- Settings (API base URL, API key, limits)

### 3.2 Compile flow
UI controls:
- plugin selector
- input uploader (file/zip)
- optional URL input (with checksum)
- policy profile selector
- advanced limits (collapsed)

Outputs:
- job id
- logs (sanitized)
- bundle download link
- schema hash, proof root, manifest hash

### 3.3 Verify flow
UI controls:
- bundle upload
- schema hash paste field
- strict toggle

Outputs:
- per-check status
- warnings
- computed hashes
- exportable JSON report

### 3.4 Schema viewer
Visualizations:
- entity list
- edges and dependency graph
- filters (type, namespace, file path)
- export (JSON, CSV)

Keep graph rendering robust for large schemas:
- pagination / virtualized lists
- progressive graph rendering
- avoid fully rendering huge graphs by default

### 3.5 On-chain view
Features:
- derive PDAs
- lookup record by schema hash
- publish flow (unsigned tx or operator mode)
- compare record hashes to bundle

---

## 4) Trust and safety posture

### 4.1 Default safe mode
The Console should default to safe compilation policies:
- network denied
- symlinks denied
- strict limits

### 4.2 Pinned fetch only
If remote URLs are enabled:
- require checksum
- show explicit warnings
- store the checksum in job metadata

### 4.3 Redaction
The Console must:
- never display API keys in logs
- never store API keys in URLs
- use secure storage for secrets (browser storage with care)

---

## 5) Deployment

### 5.1 Standalone mode
- Hosted as a static site (Next.js/Vite build)
- Config points to an API base URL

### 5.2 Embedded widget mode
- Expose the Console as an embeddable component:
  - `<SigniaConsole baseUrl="..." />`
- Useful for docs sites and landing pages

### 5.3 Environment config
At build time:
- `SIGNIA_CONSOLE_DEFAULT_API_BASE_URL`
- feature flags:
  - enable on-chain
  - enable uploads
  - enable remote URLs

At runtime:
- allow overriding base URL and API key in Settings

---

## 6) Observability

The Console can display:
- job duration
- verification durations
- error code summaries
- rate limit headers and retry hints

Operators can:
- export logs and job metadata
- inspect error codes

---

## 7) Minimal API requirements

For a basic Console:
- `GET /health`
- `POST /jobs`
- `POST /jobs/{jobId}/inputs`
- `POST /jobs/{jobId}/run`
- `GET /jobs/{jobId}`
- `POST /bundles/{bundleId}/verify` (or verify via job result)

Optional:
- schema list endpoints
- on-chain endpoints

---

## 8) Roadmap

Planned extensions:
- schema diff and evolution tracking
- multi-network on-chain lookup caching
- signed publish flows (publisher identity)
- bundle pinning and distribution (IPFS/Arweave integration)
- plugin marketplace UI

---

## 9) Related documents

- API: `docs/api/openapi.yaml`
- CLI usage: `docs/cli/usage.md`
- On-chain: `docs/onchain/registry-program.md`
