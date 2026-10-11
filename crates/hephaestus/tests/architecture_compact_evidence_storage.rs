//! AE-03 S1 — stream ledger verification (R-014, R-015, R-017, R-107).
//!
//! The ledger verifies exact-byte chains incrementally and retains compact
//! per-line byte offsets instead of full line material. Corruption and
//! torn-tail recovery behavior stay identical to `event_ledger.rs`.

use hephaestus::contracts::generated::{
    Event, EventKind, MissionDataOrigin, Provenance, RecordRef, SchemaVersion, TrustOrigin,
};
use hephaestus::ledger::{EventLedger, LedgerError};

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
