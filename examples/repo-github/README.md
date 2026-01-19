# Repo (GitHub) Example

This example compiles a pinned GitHub repository reference into a SIGNIA schema bundle.

## Input
- `input.txt` contains a repo URL plus a pinned ref (tag/commit).

## Run
```bash
bash examples/repo-github/run.sh
```

The script will:
1) compile the repo structure
2) verify the resulting proof
3) optionally publish to devnet (if configured)
