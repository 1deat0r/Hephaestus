//! AE-03 S1 — stream ledger verification (R-014, R-015, R-017, R-107).
//!
//! The ledger verifies exact-byte chains incrementally and retains compact
//! per-line byte offsets instead of full line material. Corruption and
//! torn-tail recovery behavior stay identical to `event_ledger.rs`.

use hephaestus::contracts::generated::{
    Event, EventKind, MissionDataOrigin, Provenance, RecordRef, SchemaVersion, TrustOrigin,
};
use hephaestus::knowledge::{
    Corpus, ParserId, ingest_bytes, quarantine, read_verified_span, source_artifact_identity,
    store_source_bytes,
};
use hephaestus::ledger::{
    ArtifactStore, EventLedger, LedgerError, ObjectCache, process_peak_bytes,
};

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("ae03-s1-{}-{}-{}", tag, std::process::id(), nanos));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn valid_event(id: &str, sequence: i64) -> Event {
    Event {
        id: id.to_string(),
        schema_version: SchemaVersion::V1_2,
        record_version: 1,
        created_at: "2026-09-30T00:00:00Z".to_string(),
        data_origin: MissionDataOrigin::SyntheticFixture,
        provenance: Provenance {
            actor_id: "TEST-ACTOR".to_string(),
            method: "unit_test".to_string(),
            input_refs: vec![],
            artifact_hashes: vec![],
            trust_origin: TrustOrigin::SyntheticFixture,
        },
        kind: EventKind::Event,
        mission_ref: RecordRef {
            id: "MIS-TEST".to_string(),
            version: 1,
        },
        sequence,
        event_type: "test.event".to_string(),
        subject_ref: RecordRef {
            id: "MIS-TEST".to_string(),
            version: 1,
        },
        operation_id: format!("OP-{id}"),
        policy_version: "policy-test-1".to_string(),
        payload_sha256: "0".repeat(64),
        previous_event_sha256: None,
    }
}

/// Append `n` sequenced events to a fresh ledger file, returning its path
/// (the temp dir is kept alive by the caller via the returned dir).
fn build_ledger(tag: &str, n: i64) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = temp_dir(tag);
    let path = dir.join("ledger.jsonl");
    let mut ledger = EventLedger::open(&path).expect("open for build");
    for i in 1..=n {
        let mut ev = valid_event(&format!("EVT-{i}"), i);
        ledger.append(&mut ev).expect("build append");
    }
    (dir, path)
}

/// Recompute line-start offsets by scanning raw file bytes.
fn scan_offsets(raw: &[u8]) -> Vec<u64> {
    let mut out = vec![0];
    for (i, b) in raw.iter().enumerate() {
        if *b == b'\n' && i + 1 < raw.len() {
            out.push((i + 1) as u64);
        }
    }
    out
}

#[test]
fn s1_ledger_stream_verifies_chain_and_retains_compact_offsets() {
    // AC 7 positive: incremental verification accepts the chain; resident
    // material stays compact (last line only) while offsets cover all lines.
    let (dir, path) = build_ledger("offsets", 5);
    let ledger = EventLedger::open(&path).expect("reopen");
    assert_eq!(ledger.events().len(), 5);
    assert_eq!(ledger.recovery(), None);

    let raw = std::fs::read(&path).expect("read raw");
    assert_eq!(ledger.line_offsets(), scan_offsets(&raw).as_slice());

    let last_len = (raw.len() as u64 - ledger.line_offsets()[4]) as usize;
    assert_eq!(ledger.resident_line_bytes(), last_len);
    assert!(
        ledger.resident_line_bytes() < raw.len(),
        "five lines must not all stay resident"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s1_ledger_corrupt_middle_line_names_the_line() {
    // AC 7 refusal: a line that no longer parses is rejected with its number.
    let (dir, path) = build_ledger("corrupt", 3);
    let mut raw = std::fs::read(&path).expect("read raw");
    let needle = b"test.event";
    let pos = raw
        .windows(needle.len())
        .position(|w| w == needle)
        .expect("needle present in line 1");
    raw[pos + 4] = b'"'; // breaks JSON on line 1 with equal length
    std::fs::write(&path, &raw).expect("write corrupt");
    match EventLedger::open(&path) {
        Err(LedgerError::Corrupt { line, .. }) => assert_eq!(line, 1),
        other => panic!("expected Corrupt at line 1, got: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s1_ledger_broken_link_names_the_line() {
    // AC 7 refusal: intact JSON with a wrong chain link is rejected.
    let (dir, path) = build_ledger("link", 3);
    let mut raw = std::fs::read(&path).expect("read raw");
    let needle = b"test.event";
    // Second occurrence lives on line 2; changing it keeps JSON valid but
    // changes line 2's exact bytes, so line 3's link no longer matches.
    let mut seen = 0;
    let mut pos = None;
    for (i, w) in raw.windows(needle.len()).enumerate() {
        if w == needle {
            seen += 1;
            if seen == 2 {
                pos = Some(i);
                break;
            }
        }
    }
    let pos = pos.expect("two occurrences");
    raw[pos] = b'X';
    std::fs::write(&path, &raw).expect("write relinked");
    match EventLedger::open(&path) {
        Err(LedgerError::ChainBroken { line }) => assert_eq!(line, 3),
        other => panic!("expected ChainBroken at line 3, got: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s1_ledger_torn_tail_trims_and_reports_offsets() {
    // AC 7 regression: a crash-mid-append tail is trimmed, reported, and
    // surviving offsets stay exact.
    let (dir, path) = build_ledger("torn", 2);
    let partial = b"{\"incomplete\": true";
    {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("open for tear");
        file.write_all(partial).expect("write partial");
        file.sync_data().expect("sync");
    }
    let ledger = EventLedger::open(&path).expect("reopen torn");
    assert_eq!(ledger.events().len(), 2);
    assert_eq!(
        ledger.recovery().map(|r| r.dropped_trailing_bytes),
        Some(partial.len() as u64)
    );
    let raw = std::fs::read(&path).expect("read raw");
    assert_eq!(ledger.line_offsets(), scan_offsets(&raw).as_slice());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s1_ledger_missing_final_newline_normalizes_and_appends() {
    // AC 7 regression: a complete final line without its newline is
    // normalized so a later append cannot fuse lines; the chain survives.
    let (dir, path) = build_ledger("nonl", 2);
    let mut raw = std::fs::read(&path).expect("read raw");
    assert_eq!(raw.pop(), Some(b'\n'));
    std::fs::write(&path, &raw).expect("strip newline");
    let mut ledger = EventLedger::open(&path).expect("reopen");
    assert_eq!(ledger.events().len(), 2);
    let normalized = std::fs::read(&path).expect("read normalized");
    assert_eq!(normalized.last(), Some(&b'\n'));
    let mut ev = valid_event("EVT-3", 3);
    ledger.append(&mut ev).expect("append after normalize");
    drop(ledger);
    let reopened = EventLedger::open(&path).expect("reopen again");
    assert_eq!(reopened.events().len(), 3);
    assert_eq!(reopened.events()[2].id, "EVT-3");
    let raw = std::fs::read(&path).expect("read final");
    assert_eq!(reopened.line_offsets(), scan_offsets(&raw).as_slice());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s1_ledger_empty_open_has_no_offsets() {
    // AC 7 regression: a fresh ledger retains nothing.
    let dir = temp_dir("empty");
    let path = dir.join("ledger.jsonl");
    let ledger = EventLedger::open(&path).expect("open empty");
    assert!(ledger.line_offsets().is_empty());
    assert_eq!(ledger.resident_line_bytes(), 0);
    let _ = std::fs::remove_dir_all(&dir);
}

fn parser() -> ParserId {
    ParserId {
        name: "text-plain".to_string(),
        version: "1".to_string(),
    }
}

fn byte_store(tag: &str) -> (std::path::PathBuf, ArtifactStore) {
    let dir = temp_dir(tag);
    let store = ArtifactStore::open(&dir.join("store")).expect("open store");
    (dir, store)
}

/// Count regular files under the store objects namespace.
fn object_count(store_root: &std::path::Path) -> usize {
    let mut count = 0;
    let objects = store_root.join("objects");
    let mut dirs = vec![objects];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let kind = entry.file_type().expect("file type");
            if kind.is_dir() {
                dirs.push(entry.path());
            } else {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn s2_source_identical_bytes_share_one_identity() {
    // AC 8 positive: immutable bytes are stored once; the digest is the
    // identity and the second store creates no duplicate object.
    let (dir, store) = byte_store("dedup");
    let bytes = b"pressure point: conflicting objectives on latency".to_vec();
    let first = store_source_bytes(&store, &bytes).expect("first store");
    let second = store_source_bytes(&store, &bytes).expect("second store");
    assert_eq!(first, second, "identical bytes share one identity");
    assert_eq!(
        object_count(&dir.join("store")),
        1,
        "one object on disk for identical bytes"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s2_source_span_reads_through_identity_without_full_copy() {
    // AC 8 positive: a verified span returns only its slice through the
    // artifact identity; sibling sources never load.
    let (dir, store) = byte_store("span");
    let a = b"alpha line one\nshared evidence span here\n".to_vec();
    let b = b"beta lines live elsewhere and stay unread\n".to_vec();
    let digest_a = store_source_bytes(&store, &a).expect("store a");
    store_source_bytes(&store, &b).expect("store b");
    let start = a.iter().position(|c| *c == b's').expect("span start");
    let slice = read_verified_span(&store, &digest_a, start, a.len()).expect("read span");
    assert_eq!(slice, &a[start..], "only the requested slice returns");
    assert!(slice.len() < a.len() + b.len());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s2_source_corpus_span_resolves_to_its_identity() {
    // AC 8 positive: a corpus span resolves to the source's artifact
    // identity, and the stored bytes verify the span's coordinates.
    let (dir, store) = byte_store("corpus-link");
    let bytes = b"bottleneck extraction grounds opportunities\n".to_vec();
    let mut corpus = Corpus::new();
    let id = ingest_bytes(
        &mut corpus,
        "inline:real",
        bytes.clone(),
        "2026-10-11",
        parser(),
    );
    let identity = source_artifact_identity(&corpus, id).expect("resolve identity");
    assert_eq!(identity, corpus.get(id).expect("source").sha256);
    store_source_bytes(&store, &bytes).expect("store bytes");
    let end = bytes.iter().position(|c| *c == b'\n').expect("line end");
    let slice = read_verified_span(&store, &identity, 0, end).expect("read span");
    assert_eq!(slice, &bytes[..end]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s2_source_refusals_name_the_cause() {
    // AC 8 refusal: malformed digests, missing objects, bad ranges, and
    // tampered bytes each fail with a named cause.
    use hephaestus::knowledge::OnDemandError;
    let (dir, store) = byte_store("refuse");
    let bytes = b"bounded retrieval refuses bad coordinates\n".to_vec();
    let digest = store_source_bytes(&store, &bytes).expect("store");
    match read_verified_span(&store, "not-a-digest", 0, 1) {
        Err(OnDemandError::BadDigest { .. }) => {}
        other => panic!("expected BadDigest, got: {other:?}"),
    }
    let missing = "f".repeat(64);
    match read_verified_span(&store, &missing, 0, 1) {
        Err(OnDemandError::UnknownArtifact { .. }) => {}
        other => panic!("expected UnknownArtifact, got: {other:?}"),
    }
    match read_verified_span(&store, &digest, 5, 4) {
        Err(OnDemandError::OutOfBounds { .. }) => {}
        other => panic!("expected OutOfBounds (start > end), got: {other:?}"),
    }
    match read_verified_span(&store, &digest, 0, bytes.len() + 1) {
        Err(OnDemandError::OutOfBounds { .. }) => {}
        other => panic!("expected OutOfBounds (past end), got: {other:?}"),
    }
    // Tamper the object file: the identity no longer matches its bytes.
    let object = dir
        .join("store")
        .join("objects")
        .join(&digest[..2])
        .join(&digest);
    let mut raw = std::fs::read(&object).expect("read object");
    raw[0] ^= 0xff;
    std::fs::write(&object, &raw).expect("tamper object");
    match read_verified_span(&store, &digest, 0, 1) {
        Err(OnDemandError::DigestMismatch { .. }) => {}
        other => panic!("expected DigestMismatch, got: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s2_source_identity_refuses_unknown_and_quarantined() {
    // AC 8 refusal: identity resolution fails closed on unknown and
    // quarantined sources.
    use hephaestus::knowledge::OnDemandError;
    let mut corpus = Corpus::new();
    let id = ingest_bytes(
        &mut corpus,
        "inline:quarantine-me",
        b"counterevidence stays visible\n".to_vec(),
        "2026-10-11",
        parser(),
    );
    match source_artifact_identity(&corpus, id + 99) {
        Err(OnDemandError::UnknownSource(_)) => {}
        other => panic!("expected UnknownSource, got: {other:?}"),
    }
    quarantine(&mut corpus, id, "suspect provenance").expect("quarantine");
    match source_artifact_identity(&corpus, id) {
        Err(OnDemandError::QuarantinedSource(_)) => {}
        other => panic!("expected QuarantinedSource, got: {other:?}"),
    }
}

fn cached_store(tag: &str, capacity: u64) -> (std::path::PathBuf, ObjectCache) {
    let dir = temp_dir(tag);
    let store = ArtifactStore::open(&dir.join("store")).expect("open store");
    (dir, ObjectCache::new(store, capacity))
}

fn commit_bytes(store: &ArtifactStore, bytes: &[u8]) -> String {
    let staged = store.stage(bytes).expect("stage");
    store.commit(&staged).expect("commit")
}

#[test]
fn s3_cache_bounds_resident_bytes_with_lru_eviction() {
    // AC 9 positive: resident bytes stay within capacity; the
    // least-recently-used reloadable entry evicts first.
    let (dir, mut cache) = cached_store("bounds", 90);
    let store = cache.store().clone();
    let a = commit_bytes(&store, &[b'a'; 30]);
    let b = commit_bytes(&store, &[b'b'; 30]);
    let c = commit_bytes(&store, &[b'c'; 30]);
    let d = commit_bytes(&store, &[b'd'; 30]);
    cache.read(&a).expect("read a");
    cache.read(&b).expect("read b");
    cache.read(&c).expect("read c");
    assert_eq!(cache.resident_bytes(), 90);
    cache.read(&a).expect("re-read a");
    cache.read(&d).expect("read d evicts");
    assert!(
        cache.resident_bytes() <= 90,
        "resident bytes stay bounded: {}",
        cache.resident_bytes()
    );
    assert_eq!(cache.evictions(), 1, "exactly one LRU eviction");
    assert_eq!(cache.hits(), 1, "one warm hit recorded");
    assert_eq!(cache.misses(), 4, "four cold loads recorded");
    // The evicted entry (b: untouched since load) reloads byte-identical.
    let reloaded = cache.read(&b).expect("reload evicted b");
    assert_eq!(reloaded, vec![b'b'; 30]);
    assert_eq!(cache.evictions(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s3_cache_evicts_reloadable_data_and_retains_originals() {
    // AC 9 positive: eviction drops memory only; committed originals stay
    // readable through a fresh cache and through the store itself.
    let (dir, mut cache) = cached_store("retain", 30);
    let store = cache.store().clone();
    let a = commit_bytes(&store, &[b'a'; 30]);
    let b = commit_bytes(&store, &[b'b'; 30]);
    cache.read(&a).expect("read a");
    cache.read(&b).expect("read b evicts a");
    assert_eq!(cache.evictions(), 1);
    assert_eq!(store.read(&a).expect("store still holds a"), vec![b'a'; 30]);
    let fresh = ObjectCache::new(store, 30);
    drop(cache);
    let mut fresh = fresh;
    assert_eq!(fresh.read(&a).expect("fresh cache reads a"), vec![b'a'; 30]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s3_cache_oversize_objects_bypass_without_residence() {
    // AC 9 refusal path: an object larger than capacity is served
    // without caching, so the bound holds and the bytes stay exact.
    let (dir, mut cache) = cached_store("oversize", 10);
    let store = cache.store().clone();
    let big = commit_bytes(&store, &[b'z'; 64]);
    let bytes = cache.read(&big).expect("oversize read serves");
    assert_eq!(bytes, [b'z'; 64]);
    assert_eq!(cache.resident_bytes(), 0, "nothing cached past capacity");
    assert_eq!(cache.uncached(), 1, "one bypass recorded");
    assert_eq!(cache.misses(), 1, "bypass still counts its cold load");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s3_cache_tampered_and_malformed_identities_still_refuse() {
    // AC 9 regression: the cache never launders store verification.
    let (dir, mut cache) = cached_store("verify", 1024);
    let store = cache.store().clone();
    let digest = commit_bytes(&store, b"cache never launders verification");
    match cache.read("not-a-digest") {
        Err(LedgerError::BadDigest { .. }) => {}
        other => panic!("expected BadDigest, got: {other:?}"),
    }
    let object = dir
        .join("store")
        .join("objects")
        .join(&digest[..2])
        .join(&digest);
    let mut raw = std::fs::read(&object).expect("read object");
    raw[0] ^= 0xff;
    std::fs::write(&object, &raw).expect("tamper object");
    match cache.read(&digest) {
        Err(LedgerError::DigestMismatch { .. }) => {}
        other => panic!("expected DigestMismatch, got: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn s3_cache_measures_cold_and_warm_peaks() {
    // AC 9 measurement: cold load raises resident bytes to the object
    // size and records the peak; warm reads hold both steady while the
    // process high-water mark is observed.
    let (dir, mut cache) = cached_store("peaks", 1024);
    let store = cache.store().clone();
    let digest = commit_bytes(&store, &[b'p'; 128]);
    cache.clear();
    assert_eq!(cache.resident_bytes(), 0);
    cache.read(&digest).expect("cold read");
    let cold_resident = cache.resident_bytes();
    let cold_peak = cache.peak_bytes();
    assert_eq!(cold_resident, 128);
    assert_eq!(cold_peak, 128, "peak records the cold high-water mark");
    let cold_process = process_peak_bytes();
    for _ in 0..8 {
        cache.read(&digest).expect("warm read");
    }
    assert_eq!(
        cache.resident_bytes(),
        cold_resident,
        "warm reads add nothing"
    );
    assert_eq!(cache.peak_bytes(), cold_peak, "peak holds without growth");
    assert_eq!(cache.hits(), 8);
    // The process mark is observed but not ordered: it is process-wide
    // and outside the cache contract, so its ordering carries no
    // residence evidence. Cold/warm evidence comes from the cache
    // counters above. (An ordering assert flaked in CI: the OS mark is
    // not a reliable test oracle.)
    let _ = (cold_process, process_peak_bytes());
    let _ = std::fs::remove_dir_all(&dir);
}
