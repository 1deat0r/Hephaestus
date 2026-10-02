//! E2E trace fixture generation (T-061): world-derived trace data for
//! the control plane's M2 chain.
//!
//! Generated HERE — inside the hidden world's own crate — and consumed
//! by `hephaestus` only as a checked-in data file
//! (`hephaestus/tests/fixtures/e2e-trace.log`). The `evaluator_access`
//! guard forbids any dependency edge the other way: the control plane
//! must never link the hidden evaluator, while this crate's provenance
//! test binds the file byte-for-byte to `load_corpus()` worlds.

use crate::load_corpus;

/// Rank-preserving world-derived trace text: observation magnitudes
/// sorted from the hidden worlds, bottom half mapped to 1..100 ms and
/// top half to 401..1000 ms — ordering comes entirely from the world
/// outputs (no truth read: observations only), and the scale clears
/// the discovery operators' 4x materiality rule so the fixture has
/// structure to find.
pub fn trace_fixture() -> String {
    let corpus = load_corpus();
    let world = &corpus[0].world;
    let mut mags: Vec<f64> = world
        .sample_observations()
        .iter()
        .map(|o| o.output.abs())
        .collect();
    mags.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let n = mags.len();
    let split = n / 2;
    let mut out = String::new();
    for i in 0..n {
        let stage = if i < split { "light" } else { "heavy" };
        let dur = if i < split {
            1 + (i as u64 * 99 / split.max(1) as u64)
        } else {
            401 + ((i - split) as u64 * 599 / (n - split).max(1) as u64)
        };
        out.push_str(&format!("step={stage} dur_ms={dur}\n"));
    }
    out
}
