//! T-008 budget ledger (deny-first, seam: `hephaestus::budget`).
//!
//! Obligations cited in tests: R-055/AT-055 (reserve transactionally before
//! concurrent dispatch; committed reservations plus spend never exceed the
//! authorized bound) and R-056/AT-056 (ambiguous external charges become
//! unknown-or-reconciled, never blindly duplicated).

use hephaestus::budget::{BudgetError, BudgetLedger};
use hephaestus::contracts::generated::Money;

fn usd(minor_units: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units,
    }
}

fn eur(minor_units: i64) -> Money {
    Money {
        currency: "EUR".to_string(),
        minor_units,
    }
}

/// Post-condition asserted after every operation in every test: the read
/// model agrees with an independently recomputed capacity AND is never
/// negative (the load-bearing assert — review pass 1 found a reserve path
/// that could drive it below zero).
fn assert_invariant(ledger: &BudgetLedger) {
    let expected_available =
        ledger.limit().minor_units - ledger.spent().minor_units - ledger.reserved().minor_units;
    assert_eq!(
        ledger.available().minor_units,
        expected_available,
        "read model matches independent recomputation"
    );
    assert!(
        ledger.available().minor_units >= 0,
        "available never negative"
    );
}

#[test]
fn new_fixes_currency_and_read_model_starts_at_limit() {
    let ledger = BudgetLedger::new(usd(1000)).expect("new");
    assert_eq!(ledger.currency(), "USD");
    assert_eq!(ledger.limit(), usd(1000));
    assert_eq!(ledger.available(), usd(1000));
    assert_eq!(ledger.reserved(), usd(0));
    assert_eq!(ledger.spent(), usd(0));
    assert_invariant(&ledger);
    assert!(
        matches!(BudgetLedger::new(usd(-1)), Err(BudgetError::InvalidAmount)),
        "negative limit refused"
    );
    assert!(
        matches!(
            BudgetLedger::new(Money {
                currency: "usd".to_string(),
                minor_units: 1
            }),
            Err(BudgetError::CurrencyMismatch { .. })
        ),
        "currency must match the contract pattern"
    );
}

#[test]
fn reserve_is_transactional_and_refuses_without_mutation() {
    // AT-055 / R-055: losing refusals leave the ledger byte-identical.
    let mut ledger = BudgetLedger::new(usd(1000)).expect("new");
    ledger.reserve("R1", usd(600)).expect("first reserve");
    assert_eq!(ledger.reserved(), usd(600));
    assert_eq!(ledger.available(), usd(400));
    assert_invariant(&ledger);

    let before = (ledger.reserved(), ledger.available(), ledger.spent());
    // Over the bound:
    assert!(matches!(
        ledger.reserve("R2", usd(401)),
        Err(BudgetError::InsufficientAvailable { .. })
    ));
    // Exact fit still allowed:
    ledger.reserve("R2", usd(400)).expect("exact fit");
    // Duplicate id:
    assert!(matches!(
        ledger.reserve("R1", usd(1)),
        Err(BudgetError::DuplicateReservation { .. })
    ));
    // Wrong currency / non-positive:
    assert!(matches!(
        ledger.reserve("R3", eur(1)),
        Err(BudgetError::CurrencyMismatch { .. })
    ));
    assert!(matches!(
        ledger.reserve("R3", usd(0)),
        Err(BudgetError::InvalidAmount)
    ));
    assert!(matches!(
        ledger.reserve("R3", usd(-5)),
        Err(BudgetError::InvalidAmount)
    ));
    // Only the successful exact-fit call changed state:
    assert_eq!(ledger.reserved(), usd(1000));
    assert_eq!(ledger.available(), usd(0));
    assert_eq!(ledger.spent(), before.2);
    assert_invariant(&ledger);
}

#[test]
fn commit_converts_exactly_and_denies_overage() {
    let mut ledger = BudgetLedger::new(usd(1000)).expect("new");
    ledger.reserve("R1", usd(300)).expect("reserve");
    ledger
        .commit("R1", usd(250))
        .expect("commit under reservation");
    assert_eq!(ledger.spent(), usd(250));
    assert_eq!(ledger.reserved(), usd(0));
    assert_eq!(
        ledger.available(),
        usd(750),
        "remainder returns to capacity"
    );
    assert_invariant(&ledger);

    ledger.reserve("R2", usd(100)).expect("reserve 2");
    assert!(
        matches!(
            ledger.commit("R2", usd(101)),
            Err(BudgetError::ReservationExceeded { .. })
        ),
        "actual > reserved denied — re-reserve the delta"
    );
    assert_eq!(ledger.reserved(), usd(100), "refusal mutates nothing");
    assert_invariant(&ledger);

    assert!(matches!(
        ledger.commit("NOPE", usd(1)),
        Err(BudgetError::NotFound { .. })
    ));
    assert!(matches!(
        ledger.commit("R2", eur(1)),
        Err(BudgetError::CurrencyMismatch { .. })
    ));
    assert_invariant(&ledger);
}

#[test]
fn release_returns_capacity_and_unknown_ids_are_not_found() {
    let mut ledger = BudgetLedger::new(usd(500)).expect("new");
    ledger.reserve("R1", usd(500)).expect("reserve all");
    assert_eq!(ledger.available(), usd(0));
    ledger.release("R1").expect("release");
    assert_eq!(ledger.reserved(), usd(0));
    assert_eq!(ledger.available(), usd(500));
    assert_eq!(ledger.spent(), usd(0));
    assert_invariant(&ledger);
    assert!(
        matches!(ledger.release("R1"), Err(BudgetError::NotFound { .. })),
        "released id no longer exists"
    );
    // The bound still holds after a full reserve->release->re-reserve cycle.
    ledger.reserve("R2", usd(500)).expect("re-reserve");
    assert_invariant(&ledger);
}

// --- Ticket 02: unresolved charges + exactly-once reconciliation (R-056 / AT-056) ---

/// Four-bucket invariant from MASTER_SPEC §19 once unresolved joins:
/// reserved + spent + unresolved + independently-recomputed-available equals
/// the limit, and available is never negative.
fn assert_four_bucket_invariant(ledger: &BudgetLedger) {
    let expected_available = ledger.limit().minor_units
        - ledger.spent().minor_units
        - ledger.reserved().minor_units
        - ledger.unresolved().minor_units;
    assert_eq!(
        ledger.available().minor_units,
        expected_available,
        "read model matches independent recomputation"
    );
    let total = ledger
        .reserved()
        .minor_units
        .checked_add(ledger.spent().minor_units)
        .and_then(|v| v.checked_add(ledger.unresolved().minor_units))
        .and_then(|v| v.checked_add(ledger.available().minor_units))
        .expect("sums fit");
    assert_eq!(total, ledger.limit().minor_units, "four-bucket sum holds");
    assert!(ledger.available().minor_units >= 0);
}

#[test]
fn unknown_prices_are_explicit_entries_never_zeros() {
    // MASTER_SPEC:408 — numeric unknowns are null with a reason, not zero.
    // AT-056 / R-056.
    let mut ledger = BudgetLedger::new(usd(1000)).expect("new");
    ledger.reserve("R1", usd(200)).expect("reserve");
    ledger
        .mark_unresolved("CHG-1", None, "provider invoice pending")
        .expect("unknown price recorded");
    assert_eq!(
        ledger.unresolved_count(),
        1,
        "the entry is visible even without an amount"
    );
    assert_eq!(
        ledger.unresolved(),
        usd(0),
        "no amount invented — zero here means 'no known amount', not 'no exposure'"
    );
    assert_eq!(ledger.available(), usd(800), "known holds still count");
    assert_four_bucket_invariant(&ledger);

    // A known external charge holds capacity:
    ledger
        .mark_unresolved("CHG-2", Some(usd(150)), "observed egress charge")
        .expect("known amount recorded");
    assert_eq!(ledger.unresolved(), usd(150));
    assert_eq!(ledger.available(), usd(650));
    assert_eq!(ledger.unresolved_count(), 2);
    assert_four_bucket_invariant(&ledger);

    // Refusals mutate nothing:
    let snapshot = format!("{:?}", ledger);
    assert!(
        matches!(
            ledger.mark_unresolved("CHG-3", None, ""),
            Err(BudgetError::MissingReason)
        ),
        "unknown price without a reason is refused"
    );
    assert!(matches!(
        ledger.mark_unresolved("CHG-1", None, "dup"),
        Err(BudgetError::DuplicateCharge { .. })
    ));
    assert!(
        matches!(
            ledger.mark_unresolved("R1", Some(usd(1)), "id collides with a reservation"),
            Err(BudgetError::DuplicateReservation { .. })
        ),
        "one id namespace: a reservation id cannot also be a charge id"
    );
    assert!(matches!(
        ledger.mark_unresolved("CHG-3", Some(eur(10)), "wrong currency"),
        Err(BudgetError::CurrencyMismatch { .. })
    ));
    assert!(matches!(
        ledger.mark_unresolved("CHG-3", Some(usd(-1)), "negative"),
        Err(BudgetError::InvalidAmount)
    ));
    assert_eq!(format!("{:?}", ledger), snapshot, "refusals are pure");
}

#[test]
fn unknown_price_over_available_is_refused_not_silently_negative() {
    let mut ledger = BudgetLedger::new(usd(100)).expect("new");
    ledger.reserve("R", usd(60)).expect("reserve");
    assert!(
        matches!(
            ledger.mark_unresolved("CHG-BIG", Some(usd(50)), "overrun"),
            Err(BudgetError::InsufficientAvailable { .. })
        ),
        "an overrun is an explicit error, never a silent negative available"
    );
    assert_four_bucket_invariant(&ledger);
}

#[test]
fn reconcile_settles_exactly_once_and_never_duplicates() {
    // AT-056: "The operation becomes unknown or reconciled, never blindly
    // duplicated."
    let mut ledger = BudgetLedger::new(usd(1000)).expect("new");
    ledger
        .mark_unresolved("CHG-1", Some(usd(200)), "observed charge")
        .expect("record");
    ledger.reconcile("CHG-1", usd(180)).expect("settle under");
    assert_eq!(ledger.spent(), usd(180));
    assert_eq!(ledger.unresolved(), usd(0));
    assert_eq!(ledger.unresolved_count(), 0);
    assert_eq!(ledger.available(), usd(820), "unused hold returns");
    assert_four_bucket_invariant(&ledger);

    // Replay refused — never blindly duplicated:
    assert!(matches!(
        ledger.reconcile("CHG-1", usd(180)),
        Err(BudgetError::AlreadyReconciled { .. })
    ));
    assert_eq!(ledger.spent(), usd(180), "replay mutates nothing");

    // Unknown-amount charge settles to an explicit amount too:
    ledger
        .mark_unresolved("CHG-2", None, "pending invoice")
        .expect("record unknown");
    ledger.reconcile("CHG-2", usd(40)).expect("settle");
    assert_eq!(ledger.spent(), usd(220));
    assert_eq!(ledger.unresolved_count(), 0);
    assert_four_bucket_invariant(&ledger);

    // Wrong currency / unknown id refused:
    assert!(matches!(
        ledger.reconcile("CHG-3", usd(1)),
        Err(BudgetError::NotFound { .. })
    ));
    ledger
        .mark_unresolved("CHG-3", None, "another pending")
        .expect("record");
    assert!(matches!(
        ledger.reconcile("CHG-3", eur(1)),
        Err(BudgetError::CurrencyMismatch { .. })
    ));
    assert_eq!(ledger.unresolved_count(), 1, "refusal keeps the entry");
    assert_four_bucket_invariant(&ledger);
}

#[test]
fn settling_more_than_available_is_refused_and_keeps_the_entry() {
    let mut ledger = BudgetLedger::new(usd(100)).expect("new");
    ledger.reserve("R", usd(70)).expect("reserve");
    ledger
        .mark_unresolved("CHG", Some(usd(20)), "charge")
        .expect("record");
    // available = 10; settling 50 would need 30 of headroom that the
    // reservation is holding — refused, entry intact.
    assert!(matches!(
        ledger.reconcile("CHG", usd(50)),
        Err(BudgetError::InsufficientAvailable { .. })
    ));
    assert_eq!(ledger.unresolved_count(), 1);
    assert_four_bucket_invariant(&ledger);
}

// --- Ticket 03: concurrency proof (AT-055) + determinism ---

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[test]
fn concurrent_reservations_never_oversubscribe() {
    // AT-055 / R-055: "Dispatch many tasks against a small shared budget."
    // Expected: committed reservations plus spend do not exceed the bound.
    const THREADS: usize = 8;
    const ATTEMPTS: usize = 50;
    const HOLD: i64 = 3;
    const LIMIT: i64 = 100;

    let ledger = Arc::new(Mutex::new(BudgetLedger::new(usd(LIMIT)).expect("new")));
    let successes = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..THREADS)
        .map(|thread| {
            let ledger = Arc::clone(&ledger);
            let successes = Arc::clone(&successes);
            let failures = Arc::clone(&failures);
            std::thread::spawn(move || {
                for attempt in 0..ATTEMPTS {
                    let mut guard = ledger.lock().expect("lock");
                    let id = format!("R-{thread}-{attempt}");
                    match guard.reserve(&id, usd(HOLD)) {
                        Ok(()) => {
                            successes.fetch_add(1, AtomicOrdering::SeqCst);
                        }
                        Err(_) => {
                            failures.fetch_add(1, AtomicOrdering::SeqCst);
                        }
                    }
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("thread");
    }

    let guard = ledger.lock().expect("lock");
    let ok = successes.load(AtomicOrdering::SeqCst);
    let err = failures.load(AtomicOrdering::SeqCst);
    assert_eq!(ok + err, THREADS * ATTEMPTS, "every attempt is accounted");
    assert!(
        ok > 0 && err > 0,
        "the race must actually race: {ok} succeeded, {err} refused"
    );
    assert_eq!(guard.reserved().minor_units, ok as i64 * HOLD);
    assert!(
        guard.reserved().minor_units <= LIMIT,
        "never oversubscribed"
    );
    assert_four_bucket_invariant(&guard);
    // Nothing slipped past: successes * hold is exactly what is held.
    assert_eq!(
        guard.available().minor_units,
        LIMIT - ok as i64 * HOLD,
        "refusals left capacity untouched"
    );
}

#[test]
fn identical_call_sequences_yield_identical_totals() {
    // Determinism: no clock, no randomness — state follows call order.
    fn run() -> (i64, i64, i64, i64) {
        let mut ledger = BudgetLedger::new(usd(500)).expect("new");
        ledger.reserve("A", usd(200)).expect("reserve A");
        ledger.commit("A", usd(150)).expect("commit A");
        ledger.reserve("B", usd(100)).expect("reserve B");
        ledger.release("B").expect("release B");
        ledger
            .mark_unresolved("C", Some(usd(50)), "charge")
            .expect("charge");
        ledger.reconcile("C", usd(40)).expect("settle");
        (
            ledger.available().minor_units,
            ledger.reserved().minor_units,
            ledger.spent().minor_units,
            ledger.unresolved().minor_units,
        )
    }
    assert_eq!(run(), run(), "same calls, same totals");
    assert_eq!(run(), (310, 0, 190, 0));
}

#[test]
fn reserve_respects_unresolved_holds() {
    // Regression (review pass 1): reserve's capacity check must use the
    // same formula as available() — a known unresolved hold is spent
    // capacity-in-waiting, and reserve may never push available negative.
    let mut ledger = BudgetLedger::new(usd(1000)).expect("new");
    ledger
        .mark_unresolved("CHG", Some(usd(800)), "observed charge")
        .expect("hold");
    assert!(
        matches!(
            ledger.reserve("R", usd(500)),
            Err(BudgetError::InsufficientAvailable { .. })
        ),
        "500 cannot be reserved when only 200 is free"
    );
    assert!(ledger.available().minor_units >= 0);
    ledger.reserve("fits", usd(200)).expect("exact fit allowed");
    assert_four_bucket_invariant(&ledger);
}
