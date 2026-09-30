//! Fixture corpus loader and observation receipts (T-003).
//!
//! Each corpus entry pairs a hidden [`World`] with probe candidates and the
//! expected verdicts, plus an observation receipt: the SHA-256 of the
//! canonical observation stream. Regenerating a world must reproduce its
//! receipt byte-for-byte, or the fixture has drifted and tests fail.

use serde::{Deserialize, Serialize};

use crate::{Candidate, Observation, Verdict, World};

/// One probe: a candidate the evaluator must place at the expected verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub label: String,
    pub candidate: Candidate,
    pub expected: Verdict,
}

/// One fixture-corpus entry: hidden world + probes + observation receipt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEntry {
    pub id: String,
    pub world: World,
    pub observation_receipt_sha256: String,
    pub probes: Vec<Probe>,
}

/// Canonical observation stream text: one `input,output` line per
/// observation, then SHA-256. Deterministic across platforms because Rust's
/// float formatting is correctly rounded and specification-defined.
pub fn observation_receipt(observations: &[Observation]) -> String {
    use sha2::{Digest, Sha256};
    let mut text = String::new();
    for o in observations {
        text.push_str(&format!("{},{}\n", o.input, o.output));
    }
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Load the committed fixture corpus, sorted by file name for stable order.
pub fn load_corpus() -> Vec<CorpusEntry> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/corpus");
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("corpus directory readable ({dir}): {e}"))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            let text = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("corpus file readable ({}): {e}", path.display()));
            serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("corpus entry valid ({}): {e}", path.display()))
        })
        .collect()
}
