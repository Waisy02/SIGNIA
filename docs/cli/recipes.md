
# Recipes

This document provides practical, copy-paste recipes for using SIGNIA in real workflows. Recipes are written to be reproducible and safe by default.

Assumptions:
- `signia` CLI is installed and on PATH
- you are running from the repository root or a working directory containing your input
- `--safe` is used unless otherwise stated

Related docs:
- `docs/cli/usage.md`
- `docs/cli/config.md`
- `docs/api/openapi.yaml`
- `docs/onchain/instructions.md`

---

## 1) Compile a Git repository snapshot (local)

Goal:
- create a deterministic bundle for a local repo snapshot (filesystem structure)

```bash
rm -rf ./out
signia compile --plugin repo --input . --out ./out --safe
signia verify --bundle ./out
signia inspect bundle --bundle ./out --json
```

Notes:
- `.git/` is typically excluded by the repo plugin.
- for large repos, tune limits in `signia.toml` or via flags.

---

## 2) Compile an OpenAPI spec (YAML)

Goal:
- compile OpenAPI v3 YAML into a schema bundle

```bash
rm -rf ./out
signia compile --plugin openapi --input ./specs/api.yaml --out ./out --safe
signia verify --bundle ./out
signia inspect bundle --bundle ./out
```

If you want to enforce strict parsing:
```bash
signia compile --plugin openapi --input ./specs/api.yaml --out ./out --safe \
  --plugin-config ./configs/openapi-strict.json
```

---

## 3) Verify a third-party bundle

Goal:
- verify a bundle before trusting it

```bash
signia verify --bundle ./incoming/bundle.zip --strict
signia inspect bundle --bundle ./incoming/bundle.zip --json
```

If verification fails:
- do not proceed
- inspect mismatch details

---

## 4) Determinism check (compile twice)

Goal:
- prove the compiler produces identical outputs for the same input

```bash
rm -rf out1 out2
signia compile --plugin openapi --input fixtures/openapi/petstore --out out1 --safe
signia compile --plugin openapi --input fixtures/openapi/petstore --out out2 --safe
diff -r out1 out2
```

If `diff` shows differences:
- the input bytes differ
- the environment differs
- or the plugin is nondeterministic (bug)

---

## 5) Generate a bundle zip artifact

Goal:
- produce a single file artifact for distribution (if supported)

```bash
rm -f ./signia-bundle.zip
signia compile --plugin repo --input ./my-project --out ./signia-bundle.zip --safe
signia verify --bundle ./signia-bundle.zip
```

---

## 6) Extract schema hash for sharing

Goal:
- publish the schema hash so others can verify integrity

```bash
signia inspect bundle --bundle ./out --json | jq -r .schemaHash
```

If you do not have `jq`:
```bash
signia inspect bundle --bundle ./out
```

---

## 7) Publish an anchor on-chain (devnet)

Goal:
- register schema hash in the SIGNIA registry program

Requirements:
- Solana CLI installed
- keypair available (payer)
- registry program id

```bash
export PROGRAM_ID="<REGISTRY_PROGRAM_ID>"
signia compile --plugin repo --input ./my-project --out ./out --safe
signia verify --bundle ./out

signia onchain publish \
  --bundle ./out \
  --network devnet \
  --program-id "$PROGRAM_ID" \
  --payer-keypair ~/.config/solana/id.json
```

Then verify record exists:

```bash
HASH=$(signia inspect bundle --bundle ./out --json | jq -r .schemaHash)
signia onchain get --schema-hash "$HASH" --network devnet --program-id "$PROGRAM_ID"
```

---

## 8) Use the SIGNIA API service (local)

Goal:
- compile via API instead of CLI

Start server (example):
```bash
cargo run -p signia-api -- --bind 0.0.0.0:8787
```

Create a job:
```bash
curl -sS -X POST http://localhost:8787/jobs \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $SIGNIA_API_KEY" \
  -d '{
    "plugin": { "id": "openapi", "version": "0.1.0" },
    "policies": {
      "normalization": {
        "policyVersion": "v1",
        "pathRoot": "artifact:/",
        "newline": "lf",
        "encoding": "utf-8",
        "symlinks": "deny",
        "network": "deny"
      },
      "limits": {
        "maxTotalBytes": 268435456,
        "maxFileBytes": 10485760,
        "maxFiles": 20000,
        "maxDepth": 64,
        "maxNodes": 200000,
        "maxEdges": 400000,
        "timeoutMs": 300000
      }
    }
  }'
```

Upload input:
```bash
JOB_ID="<jobId>"
curl -sS -X POST "http://localhost:8787/jobs/$JOB_ID/inputs" \
  -H "X-API-Key: $SIGNIA_API_KEY" \
  -F "kind=file" \
  -F "file=@./specs/api.yaml"
```

Run:
```bash
curl -sS -X POST "http://localhost:8787/jobs/$JOB_ID/run" \
  -H "X-API-Key: $SIGNIA_API_KEY"
```

Poll until succeeded:
```bash
curl -sS "http://localhost:8787/jobs/$JOB_ID" -H "X-API-Key: $SIGNIA_API_KEY"
```

---

## 9) CI: Verify bundles in pull requests

Goal:
- ensure a known fixture always verifies

Example step:
```bash
signia compile --plugin openapi --input fixtures/openapi/petstore --out ./out --safe
signia verify --bundle ./out --strict
```

If your project produces bundles:
- store `schemaHash` and compare against expected value
- or store the bundle itself under `artifacts/` and verify it

---

## 10) Troubleshooting recipe: isolate inputs

If results differ across machines:
1. run inside Docker with `--network=none`
2. ensure line endings are normalized
3. archive inputs and compare checksums

Example:
```bash
tar -czf input.tgz ./my-project
sha256sum input.tgz
docker run --rm -it \
  -v "$(pwd)/input.tgz:/work/input.tgz:ro" \
  -v "$(pwd)/out:/work/out:rw" \
  --network=none \
  signia:local \
  sh -lc "mkdir -p /tmp/in && tar -xzf /work/input.tgz -C /tmp/in && signia compile --plugin repo --input /tmp/in --out /work/out --safe"
```

---

## 11) Related documents

- CLI usage: `docs/cli/usage.md`
- CLI config: `docs/cli/config.md`
- API: `docs/api/openapi.yaml`
- On-chain instructions: `docs/onchain/instructions.md`
