//! Hephaestus control plane (skeleton).
//!
//! Workspace scaffolding only — no contract records yet (T-002+).

/// Layer this crate implements in the staged architecture (ADR-003).
pub const LAYER: &str = "control-plane";

#[cfg(test)]
mod tests {
    use super::LAYER;

    #[test]
    fn skeleton_links_and_tests_run() {
        assert_eq!(LAYER, "control-plane");
    }
}
