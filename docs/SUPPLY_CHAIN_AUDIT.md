# Supply-chain audit - v0.2.0 (2026-10-12 UTC)

Method: cargo-audit 0.22.2 against the frozen Cargo.lock (221
crate dependencies, 1296 RustSec advisories loaded); cargo-deny
with default config (no project deny.toml exists). The audit
scans; it upgrades nothing.

## Advisory scan

- Result: zero vulnerabilities. `cargo audit` reports no
  advisories against the locked tree.

## Yanked crates

- yoke-derive 0.8.3 (proc-macro): yanked, warning only, no
  advisory attached. Path: yoke-derive -> yoke -> icu_collections
  -> icu_normalizer -> idna_adapter -> idna -> url -> reqwest ->
  jsonschema (dev-dependency of hephaestus). Dev-only path;
  runtime binaries never load it. Disposition: pin stands,
  lockfile frozen; revisit on the next dependency ticket.

## License policy

- cargo-deny advisories, bans, sources: ok.
- cargo-deny licenses: FAILED under default config - the repo
  has no deny.toml, so the default allows nothing explicitly
  and every MIT/Apache crate reports rejected. This is tool
  noise, not a violation finding. Follow-up: add a project
  deny.toml allowlist, then re-run.

## Direct dependencies (hephaestus)

- hmac 0.13.0, libc 0.2, regex-lite 0.1.9,
  serde 1.0.229 (derive), serde_json 1.0.151, sha2 0.11.0,
  time 0.3.55 (parsing). Dev: jsonschema 0.58.2.
- All pinned through the frozen lockfile.

## Duplicates noted

- syn v2.0.119 and v3.0.6 coexist via transitive proc-macro
  paths (strum/displaydoc chains). Separate major versions
  never unify; no action.

## Limits

- No upgrade performed or recommended here.
- License posture undecided until a deny.toml exists.
- yanked monitoring is point-in-time, not continuous.
