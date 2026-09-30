//! The authoritative append-only history.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::contracts::generated::Event;
use crate::security::artifact::ArtifactDigest;

use super::{LedgerError, json_error};

/// A torn trailing line that was trimmed on open, with its byte size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recovery {
    /// Number of trailing bytes removed because the final line was incomplete.
    pub dropped_trailing_bytes: u64,
}

/// An append-only, reopenable history of contract events.
#[derive(Debug)]
pub struct EventLedger {
    path: PathBuf,
    events: Vec<Event>,
    /// Exact bytes of each complete line, trailing newline included.
    lines: Vec<Vec<u8>>,
    /// Next expected `sequence` per mission id.
    next_sequence: HashMap<String, i64>,
    recovery: Option<Recovery>,
}

impl EventLedger {
    /// Open (creating if absent) the ledger at `path`.
    ///
    /// Every line is parsed and the sha256 chain is verified end to end:
    /// line _n_'s `previous_event_sha256` must equal the sha256 of the exact
    /// bytes of line _n-1_ (trailing newline included); the first line must
    /// carry `null`. Failures name the 1-based line number.
    ///
    /// Two EOF conditions are recovered rather than rejected: a torn trailing
    /// line (incomplete JSON at EOF) is trimmed and reported via
    /// [`EventLedger::recovery`]; a complete final line missing its newline is
    /// normalized so a later append cannot fuse lines.
    pub fn open(path: &Path) -> Result<Self, LedgerError> {
        let mut events = Vec::new();
        let mut lines: Vec<Vec<u8>> = Vec::new();
        let mut next_sequence: HashMap<String, i64> = HashMap::new();
        let mut recovery: Option<Recovery> = None;
        if path.exists() {
            let contents = std::fs::read(path)?;
            for (offset, line) in split_lines_with_offsets(&contents) {
                if line.is_empty() {
                    continue;
                }
                let at_eof = offset + line.len() == contents.len();
                let event: Event = match serde_json::from_slice(line) {
                    Ok(event) => event,
                    Err(e) => {
                        if at_eof && !line.ends_with(b"\n") {
                            // Crash mid-append: trim to the last complete event.
                            let dropped = (contents.len() - offset) as u64;
                            let file = std::fs::OpenOptions::new().write(true).open(path)?;
                            file.set_len(offset as u64)?;
                            file.sync_data()?;
                            recovery = Some(Recovery {
                                dropped_trailing_bytes: dropped,
                            });
                            break;
                        }
                        return Err(LedgerError::Corrupt {
                            line: lines.len() + 1,
                            reason: e.to_string(),
                        });
                    }
                };
                let line_no = lines.len() + 1;
                // Persisted bytes must match what the chain hashes: normalize a
                // complete final line that lacks its newline.
                let mut owned = line.to_vec();
                if at_eof && !line.ends_with(b"\n") {
                    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
                    file.write_all(b"\n")?;
                    file.sync_data()?;
                    owned.push(b'\n');
                }
                if line_no == 1 {
                    if event.previous_event_sha256.is_some() {
                        return Err(LedgerError::ChainBroken { line: line_no });
                    }
                } else {
                    let expected = ArtifactDigest::of_bytes(&lines[line_no - 2]);
                    match &event.previous_event_sha256 {
                        Some(link) if link == expected.as_str() => {}
                        _ => return Err(LedgerError::ChainBroken { line: line_no }),
                    }
                }
                expect_sequence(
                    &next_sequence,
                    &event.mission_ref.id,
                    event.sequence,
                    line_no,
                )?;
                commit_sequence(
                    &mut next_sequence,
                    event.mission_ref.id.clone(),
                    event.sequence,
                );
                lines.push(owned);
                events.push(event);
            }
        }
        Ok(EventLedger {
            path: path.to_path_buf(),
            events,
            lines,
            next_sequence,
            recovery,
        })
    }

    /// Append `event`, filling its chain link. Durable on return.
    ///
    /// The chain link is filled before validation so exactly the bytes that
    /// were validated are the bytes written. A refusal mutates neither the
    /// file nor the ledger's view of history.
    pub fn append(&mut self, event: &mut Event) -> Result<(), LedgerError> {
        event.previous_event_sha256 = self
            .lines
            .last()
            .map(|prev| ArtifactDigest::of_bytes(prev).as_str().to_string());
        let violations = event.validate();
        if !violations.is_empty() {
            return Err(LedgerError::InvalidEvent(violations));
        }
        let mission = event.mission_ref.id.clone();
        expect_sequence(
            &self.next_sequence,
            &mission,
            event.sequence,
            self.lines.len() + 1,
        )?;
        let mut line = serde_json::to_vec(event).map_err(json_error)?;
        line.push(b'\n');
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(&line)?;
        file.flush()?;
        file.sync_data()?;
        // The sequence is consumed only once the bytes are durable, so a
        // failed write cannot desync memory from disk — keeping this doc's
        // "a refusal mutates neither" claim true on the IO-error path too.
        commit_sequence(&mut self.next_sequence, mission, event.sequence);
        self.lines.push(line);
        self.events.push(event.clone());
        Ok(())
    }

    /// All events in ledger order.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// One mission's events with `sequence > after_sequence`, in ledger order.
    pub fn events_after(
        &self,
        mission_id: &str,
        after_sequence: i64,
    ) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(move |event| {
            event.mission_ref.id == mission_id && event.sequence > after_sequence
        })
    }

    /// Recovery performed on the last `open`, if any.
    pub fn recovery(&self) -> Option<&Recovery> {
        self.recovery.as_ref()
    }
}

/// Check `sequence` against the mission's next expected value — pure, never
/// mutates `next_sequence`, so a later failed write cannot strand the map.
fn expect_sequence(
    next_sequence: &HashMap<String, i64>,
    mission: &str,
    sequence: i64,
    line: usize,
) -> Result<(), LedgerError> {
    let expected = next_sequence.get(mission).copied().unwrap_or(1);
    if sequence != expected {
        return Err(LedgerError::SequenceMismatch {
            mission: mission.to_string(),
            expected,
            found: sequence,
            line,
        });
    }
    Ok(())
}

/// Record that `expected` (the mission's checked sequence value) is durable.
fn commit_sequence(next_sequence: &mut HashMap<String, i64>, mission: String, expected: i64) {
    next_sequence.insert(mission, expected + 1);
}

/// Split into (byte offset, line bytes) pairs, keeping each trailing newline;
/// a final unterminated fragment is returned as-is (torn-tail candidate).
fn split_lines_with_offsets(contents: &[u8]) -> Vec<(usize, &[u8])> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, b) in contents.iter().enumerate() {
        if *b == b'\n' {
            out.push((start, &contents[start..=i]));
            start = i + 1;
        }
    }
    if start < contents.len() {
        out.push((start, &contents[start..]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{LedgerError, commit_sequence, expect_sequence, split_lines_with_offsets};
    use std::collections::HashMap;

    #[test]
    fn split_keeps_offsets_and_newlines() {
        assert!(split_lines_with_offsets(b"").is_empty());
        assert_eq!(
            split_lines_with_offsets(b"a\nb\n"),
            vec![(0, b"a\n".as_slice()), (2, b"b\n".as_slice())]
        );
        // Unterminated final fragment stays visible as its own piece.
        assert_eq!(
            split_lines_with_offsets(b"a\ntorn"),
            vec![(0, b"a\n".as_slice()), (2, b"torn".as_slice())]
        );
    }

    #[test]
    fn sequences_advance_per_mission_and_report_the_line() {
        let mut next = HashMap::new();
        expect_sequence(&next, "MIS-A", 1, 1).expect("first");
        commit_sequence(&mut next, "MIS-A".to_string(), 1);
        expect_sequence(&next, "MIS-A", 2, 2).expect("second");
        commit_sequence(&mut next, "MIS-A".to_string(), 2);
        // Missions are independent.
        expect_sequence(&next, "MIS-B", 1, 3).expect("other mission");
        commit_sequence(&mut next, "MIS-B".to_string(), 1);
        // A pure check never consumes anything.
        expect_sequence(&next, "MIS-A", 3, 99).expect("still available");
        match expect_sequence(&next, "MIS-A", 4, 4) {
            Err(LedgerError::SequenceMismatch {
                mission,
                expected,
                found,
                line,
            }) => {
                assert_eq!(mission, "MIS-A");
                assert_eq!(expected, 3);
                assert_eq!(found, 4);
                assert_eq!(line, 4);
            }
            other => panic!("expected SequenceMismatch, got: {other:?}"),
        }
    }
}
