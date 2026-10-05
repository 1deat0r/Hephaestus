//! Fixture campaign batch (dev-roadmap ticket 01; Covers AC 1).
//!
//! Twenty missions through the real chain; variance from actual runs.

use hephaestus::campaign::{BASE_THETA, BATCH_MISSIONS, CAND_THETA_BASE, run_fixture_batch};

fn fixture_trace() -> String {
    include_str!("fixtures/e2e-trace.log").to_string()
}

#[test]
fn campaign_runs_twenty_missions_with_variance_from_actual_runs() {
    let trace = fixture_trace();
    let report = run_fixture_batch(&trace).expect("batch runs end to end");
    assert_eq!(report.missions.len(), BATCH_MISSIONS);
    assert_eq!(report.total_missions, BATCH_MISSIONS);
    assert_eq!(report.failures_retained, 0);
    assert_eq!(report.outcomes.len(), BATCH_MISSIONS);
    // Every receipt binds the data actually run; values match the
    // chain outputs (interval lower bound minus declared theta).
    for m in &report.missions {
        assert!(!m.receipt_sha256.is_empty());
        let expect = m.interval.0 - m.theta;
        assert!(
            (m.value - expect).abs() < 1e-12,
            "value from run: {}",
            m.index
        );
        let expect_theta = if m.arm_id == "cand" {
            CAND_THETA_BASE + 0.01 * m.dropped_heavy_lines as f64
        } else {
            BASE_THETA
        };
        assert!((m.theta - expect_theta).abs() < 1e-12, "theta: {}", m.index);
    }
    // Variance is real: subsamples move the interval, so the cand arm
    // carries nonzero variance and the repo pairs differ.
    let cand = report
        .variance
        .per_arm
        .iter()
        .find(|(a, _, _)| a == "cand")
        .expect("cand arm");
    assert!(cand.2.expect("cand variance") > 0.0, "{cand:?}");
    assert!(
        report.variance.paired_difference_variance.expect("paired") > 0.0,
        "pairs differ across subsamples"
    );
    assert_eq!(report.variance.clustering_unit, "repository");
}

#[test]
fn twin_batches_are_byte_identical() {
    let trace = fixture_trace();
    let a = run_fixture_batch(&trace).expect("batch a");
    let b = run_fixture_batch(&trace).expect("batch b");
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
        "deterministic batch"
    );
}
