//! Hephaestus control plane (skeleton + typed contracts, T-001/T-002).
//!
//! No scheduler, authorization service, or scientific analysis runtime yet.

/// Layer this crate implements in the staged architecture (ADR-003).
pub const LAYER: &str = "control-plane";

pub mod contracts;
pub mod migrate;
pub mod semantic;

#[cfg(test)]
mod tests {
    use super::LAYER;

    #[test]
    fn skeleton_links_and_tests_run() {
        assert_eq!(LAYER, "control-plane");
    }
}
