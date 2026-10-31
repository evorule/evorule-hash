# evorule-hash

Governance normative hashing crate (BLAKE3): the single source of truth for hash formulas across the evorule governance trio — evorule-bundle / evorule-rule / evorule-server — byte-compatible with the engine's audit chain.

[![CI][badge-ci]][ci-link]
[![crates.io][badge-crate]][crate-link]
[![docs.rs][badge-docs]][docs-link]
[![License: AGPL-3.0-or-later][badge-license]][license-link]

**License**: [AGPL-3.0-or-later](LICENSE) with [dual licensing](DUAL_LICENSE.md)

## Features

- **Three primitives** — `digest(&[u8])`, `json_digest<T: Serialize>(&T)`, `prefixed(&str)` — covering raw bytes, JSON content, and self-describing `blake3:`-prefixed storage forms
- **Domain-typed hashing** — the domain set is sealed; cross-domain interoperability is prevented at compile time (hash values from different domains must never be cross-verified)
- **Byte-compatible with the engine** — mirrors the byte semantics of `evorule-reactor/src/hash.rs` (`blake3(serde_json::to_string(value))`, unprefixed 64-hex), locked by golden-vector tests; the reactor is under formal verification and frozen, and this crate deliberately mirrors rather than replaces it

## Installation

```toml
[dependencies]
evorule-hash = "0.2"
```

## Usage

```rust
use evorule_hash::{digest, json_digest, prefixed};

let h = digest(b"evorule");                          // unprefixed 64-hex
let j = json_digest(&serde_json::json!({"a": 1}));   // mirrors reactor content_hash
let s = prefixed(&h);                                // "blake3:<hex>"
```

### Domain-typed hashing

```rust
use evorule_hash::domains::{AuditChainFacts, GovernanceAsset};
use evorule_hash::DomainHash;

// Computed domains: the input spec is achieved by this crate's primitives
let g = DomainHash::<GovernanceAsset>::digest(b"bytes");
let stored = g.to_stored(); // "blake3:<hex>" — the domain-disciplined storage form
let a = DomainHash::<AuditChainFacts>::from_json(&serde_json::json!({"x": 1}));

// Reference construction: strictly validates the 64-hex body and the domain prefix
let v = DomainHash::<GovernanceAsset>::from_hex(&stored)?;

// Cross-domain non-interoperability is a compile-time guarantee:
// DomainHash<AuditChainFacts> and DomainHash<GovernanceAsset> are distinct types
// with no cross-domain conversion; PartialEq only holds within one domain.
```

## Hash Domain Registry

**Hash values from different domains must not be cross-verified** — any verification logic must use the same domain as the value being verified. Since the domain-typed release (v0.2.0), the code-level source of truth is [`src/domains.rs`](src/domains.rs): the domain set is sealed, cross-domain interoperability is enforced by the type system, and prefix shape discipline is validated at load time. New domains must be added in code and in this table together.

| Domain | Input spec | Prefix | SSOT | Status |
|---|---|---|---|---|
| Audit-chain fact hash | TCB `JsonValue` via compact serde_json serialization | none | `evorule-reactor::hash` (frozen crate) | Baseline domain; the server's offline rebuild re-verifies it with the same formula |
| Governance assets (bundle / entry fingerprints) | serde_json canonical values | `blake3:` | this crate (`json_digest` + `prefixed`) | Consumed by evorule-bundle / evorule-rule; the server's bundle API consumes it indirectly |
| Credentials (passwords / API keys / JWTs) | PBKDF2-HMAC / SHA-256 | none | evorule-rule `src/auth` | Standard cryptographic practice; never mixed with content-hash fields |
| Platform token inventory hash | BLAKE3 byte hash of 64-hex random tokens (512-bit entropy) | none (hex) | evorule-server `src/api/platform_auth.rs` | High-entropy subset of the credentials domain — lookup hashing, not password hashing; only for high-entropy random tokens |
| Publish-queue rule-set hash | custom concatenation (publish_service) | none | evorule-server `core/workspace` | pending consolidation |
| Cognitive audit chain (agent-local) | agent-local canonical_json + chain step | none | agent cognition module `src/cognition` | Independent domain; formula self-declared |
| Session anchor `initial_content_hash` | TCB `JsonValue` `Display` (spaced) | none | evorule-governance `session.rs` | Known divergence, fix pending; **not interoperable** with the audit-chain fact hash |
| Context snapshot chain ([evorule-context-inspector](https://github.com/evorule/evorule-context-inspector)) | canonical JSON skeleton (BTreeMap key order, float rejection, integer discipline); skeleton_hash + observation hash; `content_hash` exception = raw-content hash (not a chain anchor, may contain provider-native floats) | none | evorule-context-inspector | Independent domain; chain step reuses reactor `chain_step` (genesis `"genesis"`); not interoperable with audit-chain facts |
| Skill ledger content hash (evo-agent skill assembly) | SKILL.md file bytes (`digest`) | `blake3:` | evo-agent `src/agent/skill_store.rs` | Independent domain; hash pinned at approval time, recomputed at session creation, mismatch degrades automatically |
| Marketplace asset content hash (template / wasm distribution) | distribution file bytes (bare 64-hex, no prefix — a different domain from the wasm manifest's prefixed form) | none | evorule-server `src/api/marketplace.rs` | Independent domain: distribution integrity verification; not interoperable with governance hashes |
| WASM distribution integrity chain | `.wasm` file bytes (manifest `blake3` field; signature chain sig → manifest → blake3 → wasm, one signature anchoring two files) | `blake3:` | wasm-host `src/integrity.rs` | Independent domain: same shape as the bundle audit content hash, different domain |
| Rule-set version hash | dedicated `VERSION_HASH_DOMAIN` constant + rule count + per-rule streaming hasher (serialized rule \| source tag) | none (hex) | evorule-server `src/api/hit_stats.rs` | Independent domain: version-bucket fingerprint, not a verification anchor |
| Test determinism snapshot hash | direct hash of kernel / state Debug strings (stable per build) | none (hex) | per-repo tests | Independent domain: test-only self-comparison, never chained or persisted |

### Conversion seam

`json_digest` only accepts `serde_json::Serialize` input. The TCB `JsonValue` (evorule-tcb) does **not** implement `Serialize`; callers must convert to `serde_json::Value` first — and the conversion logic itself is part of the domain spec. A unified conversion contract (aligned with the reactor's `tcb_to_serde`) is pending consolidation.

## Compatibility

- v0.2.0 is purely additive over v0.1.4: the primitive surface and byte semantics are identical, so consumers can upgrade without behavioral change
- The governance trio depends on this crate with identical version requirements, checked by the ecosystem's integration-test version gate
- Hash semantics are frozen by golden vectors; versioning discipline: any change to primitive semantics is a breaking change

## Documentation

API reference: [docs.rs/evorule-hash](https://docs.rs/evorule-hash)

## License

**AGPL-3.0-or-later** (unified across the ecosystem). Dual licensing applies: closed-source use — see [DUAL_LICENSE.md](DUAL_LICENSE.md), [FREE_COMMERCIAL_LICENSE.md](FREE_COMMERCIAL_LICENSE.md) (free waiver for eligible entities), or [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md). The constitution `core_eval.json` is CC0-1.0. Commercial inquiries: <evorulelab@gmail.com>.

> History: v0.1.0–v0.1.2 shipped as AGPL, v0.1.3 briefly as Apache-2.0; from v0.1.4 the crate is back on AGPL-3.0-or-later. Already-published versions keep their original license (published crates.io versions cannot be relicensed) — consumers pinning old versions should check the license of the pinned release.

[badge-ci]: https://github.com/evorule/evorule-hash/actions/workflows/ci.yml/badge.svg
[ci-link]: https://github.com/evorule/evorule-hash/actions/workflows/ci.yml
[badge-crate]: https://img.shields.io/crates/v/evorule-hash?style=flat-square
[crate-link]: https://crates.io/crates/evorule-hash
[badge-docs]: https://img.shields.io/docsrs/evorule-hash?style=flat-square
[docs-link]: https://docs.rs/evorule-hash
[badge-license]: https://img.shields.io/badge/license-AGPL--3.0--or--later-blue?style=flat-square
[license-link]: LICENSE
