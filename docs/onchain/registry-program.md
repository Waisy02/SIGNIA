
# On-chain Registry Program

This document specifies the SIGNIA on-chain **registry program**. The registry is a minimal Solana program that anchors SIGNIA bundle integrity roots on-chain.

It is intentionally small:
- it stores schema hashes and optional proof roots
- it records publisher identity
- it supports status changes (active/revoked) with authority checks
- it avoids storing large data on-chain

The registry does not:
- store full schemas or manifests
- execute compilation
- validate complex proofs on-chain (verification is done off-chain; on-chain stores anchors)

---

## 1) Goals

1. Provide an immutable on-chain anchor for a schema hash (content-addressed identity).
2. Optionally store proof root and manifest hash for additional linkage.
3. Associate an anchor with a publisher (authority).
4. Support revocation and metadata updates without breaking identity.
5. Maintain a minimal, auditable on-chain surface area.

---

## 2) High-level design

### 2.1 Accounts

The registry uses program-derived addresses (PDAs):

1. `RegistryConfig` (singleton PDA)
- stores admin authority and program configuration

2. `SchemaRecord` (one PDA per schema hash)
- stores anchored hashes and publisher
- stores status flags and timestamps/slots (optional)

### 2.2 Seeds

Recommended PDA seeds (illustrative):
- `RegistryConfig`:
  - seeds: `["signia-registry-config"]`
- `SchemaRecord`:
  - seeds: `["signia-schema", schema_hash_bytes]`

Rules:
- use fixed ASCII seeds
- include schema hash raw bytes (32 bytes)
- avoid variable-length unbounded seeds

---

## 3) Stored fields

### 3.1 RegistryConfig
Fields:
- `admin`: Pubkey
- `bump`: u8
- `version`: u16
- `flags`: u32 (reserved)
- `reserved`: [u8; N] (future upgrades)

### 3.2 SchemaRecord
Fields:
- `schema_hash`: [u8; 32]
- `proof_root`: [u8; 32] (optional, can be zeroed)
- `manifest_hash`: [u8; 32] (optional, can be zeroed)
- `publisher`: Pubkey
- `created_slot`: u64
- `updated_slot`: u64
- `status`: u8 (0=active, 1=revoked)
- `bump`: u8
- `version`: u16
- `flags`: u32 (reserved)
- `reserved`: [u8; N]

Notes:
- storing slots is optional but practical for indexing and audits.
- use fixed-size arrays to keep account size stable.

---

## 4) Instructions

### 4.1 InitializeConfig
Creates the `RegistryConfig` PDA.

Accounts:
- `payer` (signer)
- `admin` (pubkey, may be payer)
- `config` PDA (writable)
- `system_program`

Rules:
- can only be called once (config must not exist)

### 4.2 RegisterSchema
Creates a `SchemaRecord` PDA for a schema hash.

Inputs:
- `schema_hash` (32 bytes)
- optional `proof_root` (32 bytes or none)
- optional `manifest_hash` (32 bytes or none)

Accounts:
- `payer` (signer)
- `publisher` (signer) — authority recorded in the record
- `config` PDA (read-only)
- `record` PDA derived from schema hash (writable)
- `system_program`

Rules:
- record PDA must not exist
- schema_hash must be exactly 32 bytes
- publisher must sign
- status starts active

### 4.3 UpdateSchemaMetadata
Allows the publisher (or admin, depending on policy) to update optional fields:
- proof_root
- manifest_hash
- flags

Accounts:
- `authority` (signer) — publisher or admin depending on policy
- `config` PDA (read-only)
- `record` PDA (writable)

Rules:
- authority must match publisher OR admin (define policy)
- update `updated_slot`

### 4.4 RevokeSchema
Marks a schema record as revoked.

Accounts:
- `authority` (signer) — publisher or admin depending on policy
- `config` PDA (read-only)
- `record` PDA (writable)

Rules:
- authority must be publisher or admin
- set status to revoked
- update `updated_slot`

### 4.5 TransferPublisher (optional)
Transfers publisher authority to a new pubkey.

Accounts:
- `current_publisher` (signer)
- `new_publisher` (pubkey)
- `record` PDA (writable)

Rules:
- current publisher must sign
- new publisher pubkey is stored in record

---

## 5) Authority and policy

There are two common policies:

### Policy A: Publisher-only updates
- only publisher can update metadata and revoke
- admin exists only for initializing config

Pros:
- stronger decentralization
Cons:
- no recovery if publisher key is lost

### Policy B: Admin override
- publisher can update and revoke
- admin can revoke or update in emergencies

Pros:
- supports emergency response
Cons:
- introduces admin trust

Choose one policy and document it. Default recommendation for early stage:
- Policy B with transparent governance.

---

## 6) Client integration

### 6.1 Deriving record address
Clients derive the record PDA from schema hash bytes:
- seeds: `["signia-schema", schema_hash_bytes]`

Then they fetch `SchemaRecord` and check:
- schema hash matches
- status is active
- publisher matches expected authority (optional)

### 6.2 End-to-end verification flow
1. Obtain SIGNIA bundle (schema.json, manifest.json, proof.json)
2. Run `signia verify` off-chain to recompute:
   - schema hash
   - proof root
   - manifest hash (optional)
3. Query on-chain registry for `SchemaRecord(schema_hash)`
4. Compare on-chain stored hashes to recomputed hashes
5. Accept bundle if:
   - record exists
   - status is active
   - hashes match

---

## 7) Security considerations

- Never store arbitrary user-controlled strings on-chain.
- Prefer fixed-size hashes and pubkeys.
- Validate PDA seeds and account ownership.
- Ensure signer constraints are correct.
- Enforce authority checks on updates and revocations.
- Keep program small to reduce audit surface.

---

## 8) Program upgrades and versioning

- Include `version` fields in accounts.
- Reserve bytes for future upgrades.
- Use feature flags for new behavior.
- If migrating accounts, define migration instructions and keep backward compatibility where possible.

---

## 9) Testing recommendations

- unit tests for PDA derivations
- tests for register/update/revoke flows
- negative tests:
  - wrong signer
  - wrong PDA
  - double initialization
  - register existing schema hash
- size and serialization stability tests

---

## 10) Example JSON view (client-side)

This is a conceptual JSON view (not the on-chain representation):

```json
{
  "schema_hash": "0123...abcd",
  "proof_root": "4567...cdef",
  "manifest_hash": "89ab...0123",
  "publisher": "Pubkey...",
  "status": "active",
  "created_slot": 123456789,
  "updated_slot": 123456999
}
```

---

## 11) Related documents

- Hashing: `docs/determinism/hashing.md`
- Proof spec: `docs/schemas/proof-v1.md`
- Threat model: `docs/security/security/threat-model.md`
- Trust boundaries: `docs/security/security/trust-boundaries.md`
