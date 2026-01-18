
# Responsible Disclosure

This document explains how to report security issues in SIGNIA responsibly. Please follow these guidelines to help us protect users and the ecosystem.

SIGNIA is a determinism-first, verifiability-first system. Security issues include not only traditional vulnerabilities (RCE, auth bypass) but also integrity failures (nondeterminism, hash-domain confusion) that can impact verification and trust.

---

## 1) Do not disclose vulnerabilities publicly

Please do **not**:
- open a public GitHub issue with exploit details
- post proof-of-concepts in public discussions
- share vulnerable endpoints, credentials, or private data

Public disclosure before a fix can put users at risk.

---

## 2) What qualifies as a security issue

### 2.1 High severity
- Remote code execution (RCE) or command injection
- Unauthorized file read/write (path traversal, sandbox escape)
- Authentication/authorization bypass (API service)
- Solana program vulnerabilities:
  - incorrect signer constraints
  - PDA collision or seed misuse
  - account corruption or unintended mutability
- Secret leakage (tokens, keys, private inputs)

### 2.2 Integrity and verification issues
- Nondeterministic compilation (same input produces different canonical outputs)
- Canonicalization or hashing ambiguities that could enable substitution attacks
- Proof verification bypass or inconsistent proof roots
- Bundle verification accepting invalid or tampered bundles

### 2.3 Denial of service
- Inputs that reliably crash parsers
- Resource exhaustion vectors (zip bombs, degenerate graphs)
- API endpoint abuse paths that bypass limits

---

## 3) How to report a vulnerability

### Preferred reporting channel
Use the repository’s **Security Advisories** feature (GitHub) if enabled:
- Go to the repository → **Security** → **Advisories** → **Report a vulnerability**

If advisories are not enabled, use the contact method listed in `SECURITY.md`.

### What to include
Provide enough detail to reproduce and validate the issue:

- A clear description of the vulnerability and impact
- Affected components (CLI, compiler, plugin, store, API, on-chain program)
- Steps to reproduce (minimal PoC if possible)
- Expected vs actual behavior
- Logs or stack traces (redact any secrets)
- Environment details:
  - OS
  - tool versions (`signia --version`, Rust version, Node version)
  - plugin versions if relevant
- Suggested fix or mitigation (if you have one)

---

## 4) Reporting determinism regressions

Determinism regressions are security-relevant in a hash-addressed system.

Include:
- the exact input artifact (or a minimal reproducer)
- the normalization policy used
- the expected schema hash (if known)
- the observed differing outputs and hashes
- how often it reproduces (every run vs occasionally)
- whether it depends on OS, filesystem, locale, or time

We treat “same input → different output” as a priority issue.

---

## 5) Handling sensitive information

- Do not include secrets (tokens/keys) in reports.
- If you must provide sample inputs that are private, redact or replace with minimal synthetic inputs.
- If the issue requires private inputs, state that clearly and propose a secure way to share them.

---

## 6) Response expectations

We aim to:
- acknowledge receipt promptly
- triage severity and scope
- provide an initial assessment and mitigation guidance
- coordinate a fix and responsible disclosure timeline

Severity and complexity will affect timelines, but we prioritize issues that impact verification integrity or allow code execution.

---

## 7) Coordinated disclosure

We prefer coordinated disclosure:
- reporters share details privately
- maintainers produce a fix
- a security advisory is published with:
  - affected versions
  - patched versions
  - mitigations
  - credit (if desired)

We may request that you wait to publish details until a patch is available.

---

## 8) Safe harbor

We support good-faith security research.

Please:
- avoid disrupting production systems
- avoid accessing or modifying data you do not own
- avoid social engineering or physical attacks
- follow applicable laws and regulations

If you follow these rules, we will not pursue legal action against you for good-faith testing.

---

## 9) Vulnerability severity guidance

We generally consider:

Critical:
- RCE, sandbox escape, on-chain account corruption, auth bypass with write access

High:
- unauthorized file read/write, proof verification bypass, deterministic integrity break enabling substitution

Medium:
- DoS that is easy to trigger, metadata poisoning that can mislead users at scale

Low:
- minor information disclosure without sensitive data, non-exploitable crashes, missing headers

This is guidance only; we will assess each report individually.

---

## 10) Acknowledgements

If you want to be credited, include:
- preferred name/handle
- link (optional)
- whether you want public attribution

We appreciate responsible reporting and will credit contributions where appropriate.

---

## 11) Temporary mitigations (common)

While a fix is in progress, mitigations may include:
- disable network-enabled compilation modes
- reduce plugin surface (compile only known safe formats)
- increase resource limits and timeouts defensively
- require verification in all consumer workflows
- pin tool versions and lockfiles

---

## 12) Disclaimer

There is currently **no token issued**.

Do not treat token claims as official project security communications unless explicitly documented in official releases and channels.
