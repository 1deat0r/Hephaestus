//! T-006 ticket 01 — append-only event ledger (deny-first, seam: `hephaestus::ledger`).
//!
//! Obligations cited in tests: R-013/AT-013, R-014/AT-014, R-057/AT-057.

use std::path::PathBuf;

use hephaestus::contracts::generated::{
    Event, EventKind, MissionDataOrigin, Provenance, RecordRef, SchemaVersion, TrustOrigin,
};
use hephaestus::ledger::{ArtifactStore, EventLedger};
use std::time::SystemTime;

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("t006-{}-{}-{}", tag, std::process::id(), nanos));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

/// Open `path` and append each id as sequence 1..=ids.len(), then drop it.
fn append_seq(path: &std::path::Path, ids: &[&str]) {
    let mut ledger = EventLedger::open(path).expect("open for append_seq");
    for (i, id) in ids.iter().enumerate() {
        let mut ev = valid_event(id, (i + 1) as i64);
        ledger.append(&mut ev).expect("append_seq entry");
    }
}

/// Replace the first occurrence of `needle` with an equal-length replacement.
fn patch(raw: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    assert_eq!(
        needle.len(),
        replacement.len(),
        "patch preserves byte length"
    );
    let mut out = raw.to_vec();
    let pos = out
        .windows(needle.len())
        .position(|w| w == needle)
        .expect("needle present");
    out[pos..pos + needle.len()].copy_from_slice(replacement);
    out
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

#[test]
fn appended_event_survives_reopen() {
    // AT-014 / R-014: evidence outlives ephemeral process state.
    let dir = temp_dir("append-reopen");
    let path = dir.join("ledger.jsonl");
    {
        let mut ledger = EventLedger::open(&path).expect("open empty ledger");
        assert_eq!(ledger.events().len(), 0, "fresh ledger starts empty");
        let mut ev = valid_event("EVT-ONE", 1);
        ledger.append(&mut ev).expect("append");
    }
    let reopened = EventLedger::open(&path).expect("reopen");
    assert_eq!(reopened.events().len(), 1);
    assert_eq!(reopened.events()[0].id, "EVT-ONE");
    assert_eq!(reopened.recovery(), None, "clean close needs no recovery");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn invalid_event_is_refused_and_nothing_is_written() {
    // AT-013 / R-013: authoritative state changes go through validation.
    let dir = temp_dir("deny-invalid");
    let path = dir.join("ledger.jsonl");
    let mut ledger = EventLedger::open(&path).expect("open");
    let mut bad = valid_event("EVT-BAD", 1);
    bad.created_at = "not-a-timestamp".to_string();
    assert!(
        ledger.append(&mut bad).is_err(),
        "contract-invalid event must be refused"
    );
    assert_eq!(ledger.events().len(), 0, "refusal must not mutate memory");
    assert!(
        !path.exists() || std::fs::read(&path).expect("read").is_empty(),
        "refusal must not write bytes"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chain_links_previous_line_and_tamper_fails_reopen() {
    // R-013: a broken history is reported, never silently replayed.
    let dir = temp_dir("chain");
    let path = dir.join("ledger.jsonl");
    append_seq(&path, &["EVT-ONE", "EVT-TWO"]);
    {
        let ledger = EventLedger::open(&path).expect("reopen for link check");
        assert_eq!(ledger.events()[0].previous_event_sha256, None);
        let link = ledger.events()[1]
            .previous_event_sha256
            .clone()
            .expect("second event must carry the chain link");
        assert_eq!(link.len(), 64);
        assert!(
            link.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "chain link must be lowercase sha256 hex"
        );
    }
    // Tamper with line 1 in place (same byte length). Line 1's integrity is
    // attested only by line 2's link, so the break surfaces at line 2.
    let raw = std::fs::read(&path).expect("read ledger");
    let tampered = patch(&raw, b"EVT-ONE", b"EVT-ONF");
    std::fs::write(&path, tampered).expect("write tampered");
    match EventLedger::open(&path) {
        Err(hephaestus::ledger::LedgerError::ChainBroken { line }) => assert_eq!(line, 2),
        Err(other) => panic!("expected ChainBroken at line 2, got: {other}"),
        Ok(_) => panic!("tampered chain must not open"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn non_contiguous_sequence_is_refused() {
    // R-013: the ledger enforces its own ordering invariant before writing.
    let dir = temp_dir("sequence");
    let path = dir.join("ledger.jsonl");
    let mut ledger = EventLedger::open(&path).expect("open");
    let mut first = valid_event("EVT-SEQ-1", 1);
    ledger.append(&mut first).expect("sequence 1 accepted");
    let mut gap = valid_event("EVT-SEQ-3", 3);
    assert!(
        ledger.append(&mut gap).is_err(),
        "sequence 3 after 1 must be refused"
    );
    assert_eq!(ledger.events().len(), 1, "refusal writes nothing");
    drop(ledger);
    let reopened = EventLedger::open(&path).expect("reopen");
    assert_eq!(reopened.events().len(), 1, "file untouched by refusal");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn torn_trailing_line_is_trimmed_and_history_survives() {
    // AT-057 / R-057: crash mid-append must not cost prior audit history.
    let dir = temp_dir("torn-tail");
    let path = dir.join("ledger.jsonl");
    append_seq(&path, &["EVT-ONE", "EVT-TWO"]);
    // Simulate a process death halfway through writing line 3.
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("open for tear");
    std::io::Write::write_all(&mut f, br#"{"id": "EVT-TORN, "seq"#).expect("tear");
    drop(f);

    let recovered = EventLedger::open(&path).expect("open recovers");
    assert_eq!(
        recovered.events().len(),
        2,
        "history before the tear intact"
    );
    let recovery = recovered
        .recovery()
        .expect("torn tail must be reported as recovered");
    assert!(recovery.dropped_trailing_bytes > 0);
    drop(recovered);

    // The ledger keeps working after recovery: line 3 chains from line 2.
    let mut ledger = EventLedger::open(&path).expect("reopen after recovery");
    let mut next = valid_event("EVT-THREE", 3);
    ledger.append(&mut next).expect("append after recovery");
    drop(ledger);
    let final_state = EventLedger::open(&path).expect("final open");
    assert_eq!(final_state.events().len(), 3);
    assert_eq!(final_state.recovery(), None, "no tear left behind");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn open_detects_sequence_gap_left_in_the_file() {
    // R-013: ordering is re-checked from disk, not trusted from append time.
    let dir = temp_dir("open-seq-gap");
    let path = dir.join("ledger.jsonl");
    append_seq(&path, &["EVT-ONE", "EVT-TWO"]);
    // Rewrite line 2 with sequence 5; line 2's chain link still points at the
    // untouched line 1, so only the ordering check can catch this.
    let raw = std::fs::read(&path).expect("read");
    let edited = patch(&raw, b"\"sequence\":2,", b"\"sequence\":5,");
    std::fs::write(&path, edited).expect("write edited");
    match EventLedger::open(&path) {
        Err(hephaestus::ledger::LedgerError::SequenceMismatch {
            expected,
            found,
            line,
            ..
        }) => {
            assert_eq!(expected, 2);
            assert_eq!(found, 5);
            assert_eq!(line, 2, "the error names the 1-based line number");
        }
        Err(other) => panic!("expected SequenceMismatch, got: {other}"),
        Ok(_) => panic!("sequence gap must not open"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn events_after_supports_cursor_reads() {
    // T-006 deliverable: ordered reads from a cursor (R-068 itself is out of scope).
    let dir = temp_dir("cursor");
    let path = dir.join("ledger.jsonl");
    {
        append_seq(&path, &["EVT-1", "EVT-2", "EVT-3"]);
        let ledger = EventLedger::open(&path).expect("reopen for cursor reads");
        let after_zero: Vec<_> = ledger.events_after("MIS-TEST", 0).collect();
        assert_eq!(after_zero.len(), 3);
        assert_eq!(
            after_zero.iter().map(|e| e.sequence).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        let after_one: Vec<_> = ledger.events_after("MIS-TEST", 1).collect();
        assert_eq!(
            after_one.iter().map(|e| e.sequence).collect::<Vec<_>>(),
            vec![2, 3],
            "cursor is exclusive"
        );
        let other_mission: Vec<_> = ledger.events_after("MIS-OTHER", 0).collect();
        assert!(other_mission.is_empty(), "cursor is per mission");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Ticket 02: transactional projection (AT-015 / R-015) ---

fn append_three(dir: &std::path::Path) -> std::path::PathBuf {
    let path = dir.join("ledger.jsonl");
    append_seq(&path, &["EVT-1", "EVT-2", "EVT-3"]);
    path
}

#[test]
fn projection_rebuilds_byte_identically_after_deletion() {
    // AT-015 / R-015: a view is fully derivable from authoritative records.
    let dir = temp_dir("projection-rebuild");
    let path = append_three(&dir);
    let ledger = EventLedger::open(&path).expect("open");
    let timeline = hephaestus::ledger::Timeline::rebuild(&ledger);
    assert_eq!(timeline.entries().len(), 3);
    let index_path = dir.join("timeline.json");
    timeline.write(&index_path).expect("write index");
    let original_bytes = std::fs::read(&index_path).expect("read index");

    std::fs::remove_file(&index_path).expect("delete view");

    // The rebuild side must re-derive from bytes on disk, not from the same
    // in-memory ledger that produced the original view.
    let reopened = EventLedger::open(&path).expect("reopen ledger from disk");
    let rebuilt = hephaestus::ledger::Timeline::rebuild(&reopened);
    rebuilt.write(&index_path).expect("rewrite index");
    let rebuilt_bytes = std::fs::read(&index_path).expect("read rebuilt");
    assert_eq!(
        original_bytes, rebuilt_bytes,
        "deleting the view must lose nothing"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn projection_is_written_only_after_the_event_is_durable() {
    // Append-then-project: simulated death between the two is recoverable.
    let dir = temp_dir("append-then-project");
    let path = dir.join("ledger.jsonl");
    let index_path = dir.join("timeline.json");
    {
        let mut ledger = EventLedger::open(&path).expect("open");
        let mut first = valid_event("EVT-ONE", 1);
        ledger.append(&mut first).expect("append 1");
        let timeline = hephaestus::ledger::Timeline::rebuild(&ledger);
        timeline.write(&index_path).expect("project 1");
        // Process dies here: event 2 lands, projection never runs.
        let mut second = valid_event("EVT-TWO", 2);
        ledger.append(&mut second).expect("append 2");
    }
    // Reopen: the persisted view is behind the ledger, then rebuild catches up.
    let ledger = EventLedger::open(&path).expect("reopen ledger");
    let stale = hephaestus::ledger::Timeline::load(&index_path).expect("load stale view");
    assert_eq!(stale.entries().len(), 1, "stale view is visibly behind");
    let recovered = hephaestus::ledger::Timeline::rebuild(&ledger);
    assert_eq!(
        recovered.entries().len(),
        2,
        "rebuild restores the view from authoritative records alone"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Ticket 03: content-addressed artifact store (AT-057 / R-057) ---

#[test]
fn staged_bytes_are_addressable_only_after_commit() {
    // AT-057 / R-057: half-written bytes never carry their digest's name.
    let dir = temp_dir("store-commit");
    let store_root = dir.join("store");
    let store = ArtifactStore::open(&store_root).expect("open store");
    let payload = b"artifact bytes v1".to_vec();
    let staged = store.stage(&payload).expect("stage");
    assert!(
        store.read(staged.digest()).is_err(),
        "staged bytes must not be readable under their digest yet"
    );
    let digest = store.commit(&staged).expect("commit");
    assert_eq!(digest, staged.digest());
    assert_eq!(store.read(&digest).expect("read after commit"), payload);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn malformed_digest_string_is_refused() {
    let dir = temp_dir("bad-digest-string");
    let store = ArtifactStore::open(&dir.join("store")).expect("open store");
    assert!(
        matches!(
            store.read("not-a-digest"),
            Err(hephaestus::ledger::LedgerError::BadDigest { .. })
        ),
        "a malformed digest string is refused distinctly from IO failure"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reads_fail_loudly_on_digest_mismatch() {
    // Read path re-hashes: corruption is an error, never a silent pass.
    let dir = temp_dir("store-corrupt");
    let store = ArtifactStore::open(&dir.join("store")).expect("open store");
    let staged = store.stage(b"original bytes").expect("stage");
    let digest = store.commit(&staged).expect("commit");
    let object_path = dir
        .join("store")
        .join("objects")
        .join(&digest[..2])
        .join(&digest);
    std::fs::write(&object_path, b"original bytez").expect("tamper");
    match store.read(&digest) {
        Err(hephaestus::ledger::LedgerError::DigestMismatch { .. }) => {}
        Err(other) => panic!("expected DigestMismatch, got: {other}"),
        Ok(_) => panic!("tampered object must not read cleanly"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn crash_before_commit_leaves_only_sweepable_staging() {
    // AT-057: death between stage and rename — no object, staging is GC bait.
    let dir = temp_dir("store-crash-precommit");
    let store_root = dir.join("store");
    {
        let store = ArtifactStore::open(&store_root).expect("open store");
        store.stage(b"never committed").expect("stage");
    } // process dies here
    let store = ArtifactStore::open(&store_root).expect("reopen");
    assert!(store.read(&"a".repeat(64)).is_err(), "no object exists");
    let removed = store
        .gc(SystemTime::now() + std::time::Duration::from_secs(60))
        .expect("gc");
    assert_eq!(removed.len(), 1, "abandoned staging entry swept");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn crash_after_commit_before_event_leaves_safe_orphan() {
    // AT-057: death between rename and referencing event — object retained.
    let dir = temp_dir("store-crash-postcommit");
    let store_root = dir.join("store");
    let ledger_path = dir.join("ledger.jsonl");
    let digest = {
        let store = ArtifactStore::open(&store_root).expect("open store");
        let staged = store.stage(b"durable payload").expect("stage");
        store.commit(&staged).expect("commit")
    }; // process dies before appending the referencing event
    let store = ArtifactStore::open(&store_root).expect("reopen store");
    assert_eq!(
        store.read(&digest).expect("orphan object still readable"),
        b"durable payload".to_vec()
    );
    // Give the sweep a real, abandoned target alongside the orphan.
    store.stage(b"abandoned later").expect("stage abandoned");
    let removed = store
        .gc(SystemTime::now() + std::time::Duration::from_secs(60))
        .expect("gc");
    assert_eq!(
        removed.len(),
        1,
        "only the abandoned staging entry is swept"
    );
    assert_eq!(
        store.read(&digest).expect("committed orphan survives gc"),
        b"durable payload".to_vec(),
        "committed objects are never GC candidates"
    );
    // Recovery path: the referencing event can still be recorded afterwards.
    let mut ledger = EventLedger::open(&ledger_path).expect("open ledger");
    let mut ev = valid_event("EVT-ART", 1);
    ev.provenance.artifact_hashes = vec![digest.to_string()];
    ledger.append(&mut ev).expect("append after recovery");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn fresh_staging_entries_are_protected_from_gc() {
    // The collector must not race a live writer: cutoff is respected.
    let dir = temp_dir("store-gc-cutoff");
    let store = ArtifactStore::open(&dir.join("store")).expect("open store");
    store.stage(b"in flight").expect("stage");
    let removed = store
        .gc(SystemTime::now() - std::time::Duration::from_secs(60))
        .expect("gc with past cutoff");
    assert!(
        removed.is_empty(),
        "entries newer than the cutoff must survive"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// --- Ticket 04: end-to-end recovery (AT-014 / AT-015 / AT-057) ---

#[test]
fn full_story_survives_death_at_every_persistence_boundary() {
    // AT-014 / R-014, AT-015 / R-015, AT-057 / R-057:
    // after process death at any boundary, reopen reconstructs history,
    // views, and artifacts from persisted records alone.
    let dir = temp_dir("t006-e2e");
    let ledger_path = dir.join("ledger.jsonl");
    let index_path = dir.join("timeline.json");
    let store_root = dir.join("store");

    {
        let mut ledger = EventLedger::open(&ledger_path).expect("ledger");
        let store = ArtifactStore::open(&store_root).expect("store");

        // Event 1 lands; projection follows only after it is durable.
        let mut first = valid_event("EVT-ONE", 1);
        ledger.append(&mut first).expect("append 1");
        hephaestus::ledger::Timeline::rebuild(&ledger)
            .write(&index_path)
            .expect("project after durable append");

        // Boundary: staged bytes, then death before commit.
        let staged = store.stage(b"payload-before-death").expect("stage");
        assert!(store.read(staged.digest()).is_err());

        // Boundary: committed object, then death before the referencing event.
        store.commit(&staged).expect("commit");

        // Boundary: torn line, death mid-append.
        let mut tail = std::fs::OpenOptions::new()
            .append(true)
            .open(&ledger_path)
            .expect("append tail");
        std::io::Write::write_all(&mut tail, br#"{"id": "EVT-DEAD"#).expect("tear");
    } // <- process death: no in-memory state survives

    // Reopen from persisted records only.
    let ledger = EventLedger::open(&ledger_path).expect("recover ledger");
    let recovery = ledger.recovery().expect("torn tail reported");
    assert!(recovery.dropped_trailing_bytes > 0);
    assert_eq!(ledger.events().len(), 1, "audit history preserved");

    // View reconstructed without any ephemeral state (AT-014 / AT-015).
    let timeline = hephaestus::ledger::Timeline::rebuild(&ledger);
    let ids: Vec<&str> = timeline
        .entries()
        .iter()
        .map(|e| e.event_id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec!["EVT-ONE"],
        "timeline reconstructs from the ledger alone"
    );

    // The artifact committed before death is still addressable and verified.
    let store = ArtifactStore::open(&store_root).expect("reopen store");
    let payload_digest =
        hephaestus::security::artifact::ArtifactDigest::of_bytes(b"payload-before-death");
    assert_eq!(
        store
            .read(payload_digest.as_str())
            .expect("object survives"),
        b"payload-before-death".to_vec()
    );

    // The reference event that never landed can now be recorded, and the
    // repaired view covers both.
    let mut ledger = EventLedger::open(&ledger_path).expect("ledger again");
    let mut second = valid_event("EVT-TWO", 2);
    second.provenance.artifact_hashes = vec![payload_digest.as_str().to_string()];
    ledger.append(&mut second).expect("append 2");
    let final_timeline = hephaestus::ledger::Timeline::rebuild(&ledger);
    assert_eq!(final_timeline.entries().len(), 2);
    final_timeline.write(&index_path).expect("refresh view");

    // Final state reopens cleanly: chain intact, no pending recovery.
    let final_ledger = EventLedger::open(&ledger_path).expect("final ledger");
    assert_eq!(final_ledger.recovery(), None);
    assert_eq!(
        final_ledger.events()[1].provenance.artifact_hashes,
        vec![payload_digest.as_str().to_string()]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn complete_final_line_without_newline_is_normalized_on_open() {
    // Recovery edge: complete JSON at EOF that lacks its newline must not
    // poison the next append (otherwise line N and N+1 would fuse and the
    // ledger would never open again).
    let dir = temp_dir("no-newline-eof");
    let path = dir.join("ledger.jsonl");
    {
        let mut ledger = EventLedger::open(&path).expect("open");
        let mut a = valid_event("EVT-ONE", 1);
        ledger.append(&mut a).expect("append");
    }
    // Strip the final newline: still a complete, parseable event line.
    let raw = std::fs::read(&path).expect("read");
    assert!(raw.ends_with(b"\n"));
    std::fs::write(&path, &raw[..raw.len() - 1]).expect("strip newline");

    let opened = EventLedger::open(&path).expect("open unterminated-but-complete line");
    assert_eq!(opened.events().len(), 1);
    assert_eq!(opened.recovery(), None, "not a tear: nothing was dropped");
    drop(opened);

    let mut ledger = EventLedger::open(&path).expect("reopen for append");
    let mut b = valid_event("EVT-TWO", 2);
    ledger.append(&mut b).expect("append after normalization");
    drop(ledger);
    let final_state = EventLedger::open(&path).expect("final open");
    assert_eq!(
        final_state.events().len(),
        2,
        "chain survived the fused-line risk"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unparseable_middle_line_is_reported_with_its_line_number() {
    // Spec: a break or unparseable complete line is an error naming the line.
    let dir = temp_dir("corrupt-middle");
    let path = dir.join("ledger.jsonl");
    append_seq(&path, &["EVT-ONE", "EVT-TWO", "EVT-THREE"]);
    // Replace line 2 wholesale with junk that is not JSON (line length may
    // change; the chain will also break, but parse failure comes first).
    let raw = std::fs::read(&path).expect("read");
    let lines: Vec<&[u8]> = raw
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(lines.len(), 3);
    let mut rebuilt = Vec::new();
    rebuilt.extend_from_slice(lines[0]);
    rebuilt.push(b'\n');
    rebuilt.extend_from_slice(b"not json at all");
    rebuilt.push(b'\n');
    rebuilt.extend_from_slice(lines[2]);
    rebuilt.push(b'\n');
    std::fs::write(&path, rebuilt).expect("write corrupt");
    match EventLedger::open(&path) {
        Err(hephaestus::ledger::LedgerError::Corrupt { line, .. }) => assert_eq!(line, 2),
        Err(other) => panic!("expected Corrupt at line 2, got: {other}"),
        Ok(_) => panic!("corrupt middle line must not open"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn failed_write_does_not_consume_a_sequence() {
    // Spec: a refusal mutates neither the file nor the ledger's view of
    // history — a failed write must leave the next sequence still usable.
    let dir = temp_dir("io-desync");
    let path = dir.join("ledger.jsonl");
    let mut ledger = EventLedger::open(&path).expect("open");
    let mut a = valid_event("EVT-ONE", 1);
    ledger.append(&mut a).expect("append 1");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444))
            .expect("chmod read-only");
        let mut b = valid_event("EVT-TWO", 2);
        assert!(
            ledger.append(&mut b).is_err(),
            "read-only file must fail the write"
        );
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("chmod read-write");
    }

    let mut b2 = valid_event("EVT-TWO", 2);
    ledger
        .append(&mut b2)
        .expect("sequence 2 still available after a failed write");
    drop(ledger);
    let reopened = EventLedger::open(&path).expect("reopen");
    assert_eq!(reopened.events().len(), 2, "one real event, no ghost lines");
    let _ = std::fs::remove_dir_all(&dir);
}
