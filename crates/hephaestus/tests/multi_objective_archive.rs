//! R-032 / AT-032 multi-objective archive (T-044); R-033/AT-033
//! (alternatives with no calibrated model are discriminated by
//! qualitative value bands — no fabricated probabilities).
//!
//! Integration tests at the public seam: `MultiObjectiveArchive`,
//! `ValueBand`, dominance, audited eviction.

use hephaestus::orchestrator::archive::{EvictionReason, MultiObjectiveArchive, ValueBand};

use ValueBand::{High, Low, Medium};

#[test]
fn at032_both_candidates_remain() {
    // AT-032 negative case: cheap+uncertain vs expensive+plausible.
    // Neither dominates -> both stay in the frontier.
    let mut archive = MultiObjectiveArchive::new(16);
    archive.admit(1, "cheap uncertain probe", 0.5, Low);
    archive.admit(2, "expensive plausible plan", 50.0, High);
    let frontier: Vec<u64> = archive.frontier().iter().map(|n| n.id).collect();
    assert!(frontier.contains(&1), "cheap uncertain must stay");
    assert!(frontier.contains(&2), "expensive plausible must stay");
}

#[test]
fn dominated_node_is_evictable() {
    // Node 3 is dominated by node 1 (same cost, higher value): only
    // non-dominated nodes remain in the frontier.
    let mut archive = MultiObjectiveArchive::new(16);
    archive.admit(1, "dominator", 1.0, High);
    archive.admit(2, "cheap uncertain", 0.5, Low);
    archive.admit(3, "dominated", 1.0, Low);
    let frontier: Vec<u64> = archive.frontier().iter().map(|n| n.id).collect();
    assert!(!frontier.contains(&3));
    // And eviction was audited with its dominator named.
    assert!(
        archive
            .evictions()
            .iter()
            .any(|(id, r)| *id == 3 && matches!(r, EvictionReason::Dominated { by: 1 }))
    );
}

#[test]
fn cap_eviction_is_audited_lru_lowest_band() {
    // Three mutually non-dominated nodes (strictly rising cost AND
    // band), cap 2 -> one LRU eviction from the lowest band, audited.
    let mut archive = MultiObjectiveArchive::new(2);
    archive.admit(1, "a low", 1.0, Low);
    archive.admit(2, "b mid", 2.0, Medium);
    archive.admit(3, "c high", 3.0, High);
    assert!(archive.len() <= 2);
    // Node 1 (lowest band, oldest) was evicted with an audited reason.
    assert!(
        archive
            .evictions()
            .iter()
            .any(|(id, r)| *id == 1 && matches!(r, EvictionReason::CapLru { cap: 2 }))
    );
}

#[test]
fn missing_value_defaults_low_no_fabrication() {
    // Default band is Low: a node without a declared value is never
    // assigned an invented score (R-033 continuity).
    assert_eq!(ValueBand::default(), Low);
    // Low-band nodes survive alongside High when costs differ enough
    // that neither dominates: cheap Low is not dominated by costly High.
    let mut archive = MultiObjectiveArchive::new(16);
    archive.admit(1, "cheap low", 0.1, Low);
    archive.admit(2, "costly high", 100.0, High);
    let frontier: Vec<u64> = archive.frontier().iter().map(|n| n.id).collect();
    assert!(frontier.contains(&1) && frontier.contains(&2));
}

#[test]
fn twin_run_byte_identical() {
    let build = || {
        let mut a = MultiObjectiveArchive::new(4);
        a.admit(1, "x", 1.5, Medium);
        a.admit(2, "y", 2.5, High);
        a.admit(3, "z", 1.0, Low);
        a.admit(4, "w", 1.5, Medium);
        (
            a.frontier()
                .iter()
                .map(|n| (n.id, n.cost, n.value))
                .collect::<Vec<_>>(),
            a.evictions().to_vec(),
        )
    };
    assert_eq!(build(), build());
}
