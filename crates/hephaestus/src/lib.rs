//! Hephaestus control plane (skeleton + typed contracts, T-001/T-002).
//!
//! No scheduler, authorization service, or scientific analysis runtime yet.

/// Layer this crate implements in the staged architecture (ADR-003).
pub const LAYER: &str = "control-plane";

pub mod acceleration;
pub mod advisory;
pub mod amendment;
pub mod backend;
pub mod budget;
pub mod containment;
pub mod contracts;
pub mod discovery;
pub mod domainpack;
pub mod dossier;
pub mod evalsuite;
pub mod evaluation;
pub mod experiment;
pub mod genesis;
pub mod knowledge;
pub mod ledger;
pub mod lifecycle;
pub mod methods;
pub mod migrate;
pub mod mission;
pub mod operations;
pub mod orchestrator;
pub mod pilot;
pub mod policy;
pub mod priorart;
pub mod prototype;
pub mod release;
pub mod reproduction;
pub mod sandbox;
pub mod scheduler;
pub mod security;
pub mod selfimprove;
pub mod semantic;
pub mod traceability;
pub mod workspace;

#[cfg(test)]
mod tests {
    use super::LAYER;

    #[test]
    fn skeleton_links_and_tests_run() {
        assert_eq!(LAYER, "control-plane");
    }
}
