# SIGNIA Examples

These examples are runnable end-to-end using the workspace CLI (`signia-cli`).

Each example includes:
- an input fixture
- a `run.sh` script that compiles + verifies (and optionally publishes to devnet)
- an `expected/` directory with sample outputs

## Prerequisites
- Rust stable
- `signia` CLI available (build from repo root):
  - `cargo build -p signia-cli`
  - `./target/debug/signia --help`

## Running
From the repository root:

```bash
bash examples/openapi/run.sh
bash examples/dataset/run.sh
bash examples/workflow/run.sh
bash examples/repo-github/run.sh
```

Notes:
- The `expected/` outputs are illustrative. Your actual hashes will change if inputs change.
- The scripts rely on deterministic canonicalization for stable results per input.
