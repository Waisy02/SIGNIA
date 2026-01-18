# signia-api

`signia-api` is the HTTP service for SIGNIA.

It exposes endpoints for:
- `POST /v1/compile` — submit a structure payload and receive deterministic artifacts (schema/manifest/proof)
- `POST /v1/verify` — verify proofs
- `GET /v1/artifacts/:id` — retrieve stored artifacts by object id
- `GET /v1/plugins` — list supported plugin ids and versions
- `GET /healthz` — health check
- `/v1/registry/*` — placeholder for on-chain registry integration

## Running

```bash
cargo run -p signia-api -- --config ./config.json
```

Example config:

```json
{
  "listen_addr": "0.0.0.0:8080",
  "log_level": "info",
  "auth": { "mode": "optional", "bearer_tokens": [] },
  "rate_limit": { "enabled": true, "rpm": 600 },
  "cors": { "allow_any_origin": true },
  "store_root": ".signia"
}
```

## Notes

- This server is deterministic with respect to compilation outputs:
  - Inputs are canonicalized before hashing
  - Artifact ids are content-addressed (sha256)
- Rate limiting and auth are middleware-level controls and do not affect artifact contents.

## License

MIT OR Apache-2.0
