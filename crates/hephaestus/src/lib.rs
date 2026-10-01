//! Hephaestus control plane (skeleton + typed contracts, T-001/T-002).
//!
//! No scheduler, authorization service, or scientific analysis runtime yet.

/// Layer this crate implements in the staged architecture (ADR-003).
pub const LAYER: &str = "control-plane";

pub mod budget;
pub mod contracts;
pub mod discovery;
pub mod experiment;
pub mod genesis;
pub mod knowledge;
pub mod ledger;
pub mod migrate;
pub mod mission;
pub mod operations;
pub mod orchestrator;
pub mod policy;
pub mod priorart;
pub mod sandbox;
pub mod scheduler;
pub mod security;
pub mod semantic;
pub mod traceability;

#[cfg(test)]
mod tests {
    use super::LAYER;

    #[test]
    fn skeleton_links_and_tests_run() {
        assert_eq!(LAYER, "control-plane");
    }
}
