# 01: Ingest local files, capture bytes, verify spans

**What to build:** Authorized local files ingest into immutable captured
bytes with sha256, locator, validity/ingestion times, and parser identity;
byte-offset spans verify against captured bytes and fail closed on tampering
or drift (R-017, R-107).

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `ingest_bytes` captures bytes + sha256 + times + parser id/version (red-first seam)
- [x] Local-file adapter reads real files through the same path (GLOSSARY excerpt fixture)
- [x] `verify_span` passes on exact slices, fails on tampered bytes / wrong offsets / unrecorded transforms
- [x] Spans never resolve against live files (test proves captured-bytes-only: modify file after ingest, span still verifies against capture)
- [x] Twin-run byte-identical ingest

## Comments
Done 2026-10-01: `knowledge/record.rs` + `service.rs` ingest/verify + `LocalFileAdapter`; 5/5 in `corpus_ingest.rs` (red was unresolved-import; 2 failures were test-data bugs: crate CWD path, matching "bad" span).
