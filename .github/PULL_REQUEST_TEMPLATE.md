## Summary
Describe what this PR changes and why.

- What problem does it solve?
- What behavior changes?
- Any user-facing impact?

## Scope
Select all that apply:

- [ ] Core compiler (`crates/signia-core`)
- [ ] Plugins (`crates/signia-plugins`)
- [ ] Store / proofs (`crates/signia-store`)
- [ ] API service (`crates/signia-api`)
- [ ] CLI (`crates/signia-cli`)
- [ ] Solana registry program (`programs/signia-registry`)
- [ ] SDK (TypeScript) (`sdk/ts`)
- [ ] SDK (Python) (`sdk/python`)
- [ ] Console (`console/web`)
- [ ] Interface module (`console/interface`)
- [ ] Docs / specs (`docs/`, `schemas/`)
- [ ] CI / Tooling

## Motivation
Explain the motivation and context.

- Why is this needed now?
- What assumptions does it make?
- What constraints shaped the implementation?

## Design / Implementation Notes
Provide technical details that will help reviewers.

- Key decisions and trade-offs
- Any new invariants
- Any relevant diagrams or links

## Determinism Checklist (required when applicable)
If this PR touches parsing, canonicalization, hashing, ordering, schema generation, or proof generation:

- [ ] I documented any new deterministic rules (ordering, normalization, hashing).
- [ ] I added or updated golden fixtures to enforce byte-for-byte stability.
- [ ] I verified "same input → same output" across multiple runs locally.
- [ ] I considered platform differences (path separators, newline normalization, locale/timezone).
- [ ] I used domain-separated hashing and did not reuse hash domains.

## Schema / Proof Compatibility
If this PR changes schema/manifest/proof formats:

- [ ] Backward compatibility is maintained.
- [ ] A version bump is included (schema version and/or manifest version).
- [ ] Migration notes are documented (if needed).
- [ ] JSON Schemas in `schemas/` were updated.
- [ ] SDK types were updated (TS/Python) if relevant.

## On-chain Program Safety (required when applicable)
If this PR changes Solana program logic:

- [ ] Instruction and account changes are documented.
- [ ] Account sizing and rent implications were considered.
- [ ] PDA seeds and constraints were reviewed for safety.
- [ ] Upgrade authority / governance implications were considered.
- [ ] Anchor tests were added/updated.

## API / CLI Contract Changes
If this PR changes public interfaces:

- [ ] CLI output remains stable or changes are documented.
- [ ] Errors are actionable and include context.
- [ ] API endpoints and DTOs are updated consistently.
- [ ] OpenAPI spec (`docs/api/openapi.yaml`) is updated (if applicable).

## Testing
Describe what tests you ran and how reviewers can validate.

### Unit / Integration
- [ ] `cargo test --all-features --locked`
- [ ] Targeted crate tests (list them):
  - `cargo test -p <crate> ...`

### Lint / Format
- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `pnpm lint` (if applicable)
- [ ] `pnpm typecheck` (if applicable)

### E2E (recommended when applicable)
- [ ] I ran at least one end-to-end flow:
  - `signia compile ...`
  - `signia verify ...`
  - `signia publish ...` (if on-chain)
  - `signia fetch ...` (if on-chain)

## Performance / Resource Impact
- Does this change affect CPU, memory, or storage usage?
- Any new caching or indexing behavior?
- Any expected impact on compile time?

## Security Considerations
- Any new attack surface (parsers, plugin sandbox, network I/O)?
- Any dependency changes that require extra scrutiny?
- Any secrets or credentials involved? (Should be none.)

## Breaking Changes
- [ ] No breaking changes
- [ ] Yes (describe below)

If breaking:
- What breaks?
- Who is impacted?
- Migration plan:

## Documentation
- [ ] Docs updated (or not needed)
- [ ] Examples updated (or not needed)
- [ ] Changelog entry added (or not needed)

## Screenshots / Output (optional)
If this affects console/UI or CLI output, include screenshots or example output.

## Related Issues
Link issues or PRs:
- Closes #
- Related #

## Checklist
- [ ] I kept the change focused and minimal.
- [ ] I added tests or updated existing tests.
- [ ] I updated docs/specs when appropriate.
- [ ] I verified CI should pass (or explained why not).
