//! Release packet assembly (T-028, R-077/R-078/R-080).
//!
//! Assembles the scoped release packet with a critical-failure gate
//! (R-077), required finite-suite uncertainty (R-078 — no
//! universal-reliability representation exists), all five outcome classes
//! (R-080), quantity-only measurements, and a reproducibility report.

pub mod record;

pub use record::{
    Finding, Measurement, OutcomeCounts, QualificationInputs, ReleaseBlock, ReleasePacket,
    ReproducibilityReport, ScopeLabel, Severity, Uncertainty, scope_label,
};

/// Assemble the release packet (R-077 gate + completeness).
pub fn assemble_release(packet: ReleasePacket) -> Result<ReleasePacket, ReleaseBlock> {
    // R-077: a critical unresolved failure INSIDE the declared scope
    // blocks the applicable release gate. Out-of-scope criticals are
    // recorded but do not block (scope honesty).
    let blocking = packet
        .findings
        .iter()
        .find(|f| f.severity == Severity::Critical && !f.resolved && f.in_scope);
    if let Some(f) = blocking {
        return Err(ReleaseBlock::CriticalUnresolvedInScope(f.id.clone()));
    }
    // R-078: finite-suite uncertainty required; there is no field that
    // could carry a universal-reliability claim.
    if packet.uncertainty.n == 0 || packet.uncertainty.suite_id.is_empty() {
        return Err(ReleaseBlock::MissingUncertainty);
    }
    // Reproducibility report must be present with commands.
    if packet.reproducibility.repro_commands.is_empty() {
        return Err(ReleaseBlock::MissingReproducibility);
    }
    // R-103 continuity: measurements are quantities; a priced entry (a
    // "$" in the unit) is an invented conversion — refused.
    for m in &packet.cost_latency {
        if m.unit.contains('$') {
            return Err(ReleaseBlock::PricedMeasurement);
        }
    }
    Ok(packet)
}
