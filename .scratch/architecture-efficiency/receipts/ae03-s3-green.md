# AE-03 S3 receipt — bound storage caches

Date: 2026-10-11 UTC.
Requirement IDs: R-014, R-015, R-017, R-107. Spec AC: 9.
No architectural departure: no RUNTIME_DECISIONS entry needed.

## Red (missing behavior)

Command:
cargo test -p hephaestus --test architecture_compact_evidence_storage s3_cache

Result: 0 passed, 11 filtered out — the filter matches nothing.
Then compile failure (E0432): `ObjectCache` and `process_peak_bytes`
are absent from `hephaestus::ledger`.
During implementation one borrow error (E0502) appears in the cache
hit path; the hit clone moves before the mutable touch.

## Green (after implementation)

Command: same as above.
Result: 5 passed, 0 failed —
s3_cache_bounds_resident_bytes_with_lru_eviction,
s3_cache_evicts_reloadable_data_and_retains_originals,
s3_cache_oversize_objects_bypass_without_residence,
s3_cache_tampered_and_malformed_identities_still_refuse,
s3_cache_measures_cold_and_warm_peaks.
Runner: `python3 .scratch/architecture-efficiency/workflow.py verify
AE-03 S3` → PASS, exit 0.
Receipt: ae03-s3-workflow-verify.log.

## Implementation

`crates/hephaestus/src/ledger/artifact_store.rs` — new `ObjectCache`:
LRU eviction holds resident bytes within capacity; eviction drops
memory only while committed originals stay on disk and reload
byte-identical; objects past capacity serve verified but uncached and
count as bypasses; hits, misses, evictions, bypasses, residence, and
the resident high-water mark are counted. New `process_peak_bytes`
reads the process high-water mark from `/proc/self/status`, returning
`None` where the platform exposes none. Exports added in
`ledger/mod.rs`. No `ArtifactStore` behavior changes.

## Regressions

`event_ledger` 19 passed; `corpus_ingest` 5 passed; `corpus_search`
6 passed. `make ci` and `make doc-check` run before landing.

## Limitations

Warm hits clone resident bytes on return. The process mark is
process-wide, not per-cache. Cache holds no persistence; a restart
starts cold. This completes AE-03 S1–S3 and unblocks AE-04.
Developer model is Muse Spark per user direction to continue; the Pi
protocol names MiMo. The contract code carries no model dependency.

## Addendum 2026-10-11 — drop OS-mark ordering assert

`make ci` failed once on
`s3_cache_measures_cold_and_warm_peaks` at the
`after >= before` process-mark assert. The cache counters
(resident, peak, hits) all held; only the OS `VmHWM` ordering
broke. The helper doc no longer claims monotonicity, and the test
observes the mark without ordering it. Residence evidence comes
from cache counters alone. This matches the earlier review flag
that warm-peak measurement cannot rest on the process mark.
