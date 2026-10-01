//! The budget ledger (T-008): integer minor units, one currency, four
//! accounting states from MASTER_SPEC §19 — reserved, spent, released, and
//! unresolved — where *released* capacity is visible as [`BudgetLedger::available`]
//! (freed capacity, not a separate counter that could drift).
//!
//! Deterministic by construction: no clock, no randomness, checked integer
//! arithmetic only, state follows call order. The "never oversubscribe"
//! guarantee is in-process: every mutator takes `&mut self`, so the
//! check-and-debit step cannot interleave (MASTER_SPEC's local MVP: one
//! authoritative writer).

use crate::contracts::generated::Money;

/// A held amount keyed by a caller-chosen opaque id.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Reservation {
    id: String,
    amount: i64,
}

/// Why a budget operation was refused. Every variant mutates nothing.
#[derive(Debug, PartialEq)]
pub enum BudgetError {
    /// Zero, negative, or arithmetically unsafe amount (or limit).
    InvalidAmount,
    /// The amount's currency differs from the ledger's fixed currency.
    CurrencyMismatch {
        /// The ledger's currency.
        expected: String,
        /// The offending amount's currency.
        found: String,
    },
    /// A reservation with this id already exists.
    DuplicateReservation {
        /// The conflicting id.
        id: String,
    },
    /// The reservation would exceed `limit - spent - reserved`.
    InsufficientAvailable {
        /// What the caller asked to hold.
        requested: Money,
        /// What is actually available right now.
        available: Money,
    },
    /// `commit` of more than was reserved — re-reserve the delta instead.
    ReservationExceeded {
        /// The active reservation's amount.
        reserved: Money,
        /// The actual amount the caller tried to commit.
        actual: Money,
    },
    /// No reservation with this id exists (never created, or already settled).
    NotFound {
        /// The unknown id.
        id: String,
    },
    /// An unresolved charge was recorded without a reason (unknowns are
    /// null-with-reason, never silent).
    MissingReason,
    /// An unresolved charge with this id already exists.
    DuplicateCharge {
        /// The conflicting id.
        id: String,
    },
    /// This charge was already settled — AT-056 forbids replaying it.
    AlreadyReconciled {
        /// The settled charge id.
        id: String,
    },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::InvalidAmount => {
                write!(f, "amount must be positive and fit integer minor units")
            }
            BudgetError::CurrencyMismatch { expected, found } => {
                write!(
                    f,
                    "currency mismatch: ledger is {expected}, amount is {found}"
                )
            }
            BudgetError::DuplicateReservation { id } => {
                write!(f, "reservation id already exists: {id}")
            }
            BudgetError::InsufficientAvailable {
                requested,
                available,
            } => write!(
                f,
                "insufficient available budget: requested {} {}, available {} {}",
                requested.minor_units,
                requested.currency,
                available.minor_units,
                available.currency
            ),
            BudgetError::ReservationExceeded { reserved, actual } => write!(
                f,
                "actual {} {} exceeds reservation {} {}",
                actual.minor_units, actual.currency, reserved.minor_units, reserved.currency
            ),
            BudgetError::NotFound { id } => write!(f, "no reservation with id: {id}"),
            BudgetError::MissingReason => write!(f, "an unresolved charge requires a reason"),
            BudgetError::DuplicateCharge { id } => {
                write!(f, "unresolved charge id already exists: {id}")
            }
            BudgetError::AlreadyReconciled { id } => {
                write!(f, "charge already reconciled: {id}")
            }
        }
    }
}

impl std::error::Error for BudgetError {}

/// An external charge awaiting reconciliation: known amount holds capacity,
/// unknown amount (`None`) is an entry with a reason — never a zero.
#[derive(Debug, Clone, PartialEq, Eq)]
struct UnresolvedCharge {
    id: String,
    amount: Option<i64>,
    reason: String,
}

/// A single-currency budget over integer minor units.
#[derive(Debug, Clone)]
pub struct BudgetLedger {
    currency: String,
    limit: i64,
    reservations: Vec<Reservation>,
    unresolved: Vec<UnresolvedCharge>,
    reconciled: Vec<String>,
    spent: i64,
}

impl BudgetLedger {
    /// Create a ledger with a fixed currency and a non-negative limit.
    /// The currency must satisfy the contract pattern (`[A-Z]{3}`).
    pub fn new(limit: Money) -> Result<Self, BudgetError> {
        if limit.minor_units < 0 {
            return Err(BudgetError::InvalidAmount);
        }
        let bytes = limit.currency.as_bytes();
        if bytes.len() != 3 || !bytes.iter().all(|b| b.is_ascii_uppercase()) {
            return Err(BudgetError::CurrencyMismatch {
                expected: "three uppercase letters (contract money pattern)".to_string(),
                found: limit.currency.clone(),
            });
        }
        Ok(BudgetLedger {
            currency: limit.currency,
            limit: limit.minor_units,
            reservations: Vec::new(),
            unresolved: Vec::new(),
            reconciled: Vec::new(),
            spent: 0,
        })
    }

    /// The ledger's fixed currency.
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// The authorized bound.
    pub fn limit(&self) -> Money {
        self.money(self.limit)
    }

    /// Free capacity: `limit - spent - reserved - known unresolved amounts`
    /// (released capacity shows up here).
    pub fn available(&self) -> Money {
        self.money(
            self.available_minor()
                .expect("invariant: holds never exceed limit"),
        )
    }

    /// THE capacity formula — checked arithmetic, computed in exactly one
    /// place and shared by every reader and every capacity decision
    /// (reserve, mark_unresolved), so the check and the view can never
    /// disagree (review pass 1's bug).
    fn available_minor(&self) -> Result<i64, BudgetError> {
        let used = self
            .spent
            .checked_add(self.reserved_total())
            .and_then(|v| v.checked_add(self.unresolved_known_total()))
            .ok_or(BudgetError::InvalidAmount)?;
        self.limit
            .checked_sub(used)
            .ok_or(BudgetError::InvalidAmount)
    }

    /// Total of unresolved charges with a known amount (held capacity).
    pub fn unresolved(&self) -> Money {
        self.money(self.unresolved_known_total())
    }

    /// Number of unresolved charge *entries*, including amount-less ones —
    /// presence is visible even when the price is not (MASTER_SPEC:408).
    pub fn unresolved_count(&self) -> usize {
        self.unresolved.len()
    }

    /// Record an external charge awaiting reconciliation (R-056). Unknown
    /// price = `amount: None` plus a mandatory reason; a known amount must
    /// fit available capacity.
    pub fn mark_unresolved(
        &mut self,
        id: &str,
        amount: Option<Money>,
        reason: &str,
    ) -> Result<(), BudgetError> {
        if reason.trim().is_empty() {
            return Err(BudgetError::MissingReason);
        }
        if self.unresolved.iter().any(|c| c.id == id) {
            return Err(BudgetError::DuplicateCharge { id: id.to_string() });
        }
        if self.reservations.iter().any(|r| r.id == id) {
            return Err(BudgetError::DuplicateReservation { id: id.to_string() });
        }
        if let Some(amount) = amount.as_ref() {
            self.check_amount(amount)?;
            let available = self.available_minor()?;
            if amount.minor_units > available {
                return Err(BudgetError::InsufficientAvailable {
                    requested: amount.clone(),
                    available: self.money(available),
                });
            }
        }
        self.unresolved.push(UnresolvedCharge {
            id: id.to_string(),
            amount: amount.map(|m| m.minor_units),
            reason: reason.to_string(),
        });
        Ok(())
    }

    /// Settle an unresolved charge into spend exactly once (AT-056:
    /// "never blindly duplicated"). A second call refuses with
    /// [`BudgetError::AlreadyReconciled`] and mutates nothing.
    pub fn reconcile(&mut self, id: &str, settled: Money) -> Result<(), BudgetError> {
        if self.reconciled.iter().any(|r| r == id) {
            return Err(BudgetError::AlreadyReconciled { id: id.to_string() });
        }
        self.check_amount(&settled)?;
        let index = self
            .unresolved
            .iter()
            .position(|c| c.id == id)
            .ok_or_else(|| BudgetError::NotFound { id: id.to_string() })?;
        let other_unresolved: i64 = self
            .unresolved
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, c)| c.amount.unwrap_or(0))
            .try_fold(0i64, |acc, a| acc.checked_add(a))
            .ok_or(BudgetError::InvalidAmount)?;
        // What could be settled if this charge vanished — the honest
        // `available` payload (never a fabricated zero; review pass 1).
        let settle_room = self
            .limit
            .checked_sub(self.spent)
            .and_then(|v| v.checked_sub(self.reserved_total()))
            .and_then(|v| v.checked_sub(other_unresolved))
            .ok_or(BudgetError::InvalidAmount)?;
        if settled.minor_units > settle_room {
            return Err(BudgetError::InsufficientAvailable {
                requested: settled,
                available: self.money(settle_room.max(0)),
            });
        }
        let new_spent = self
            .spent
            .checked_add(settled.minor_units)
            .ok_or(BudgetError::InvalidAmount)?;
        // All checks precede the first mutation: refusals are pure.
        self.unresolved.remove(index);
        self.spent = new_spent;
        self.reconciled.push(id.to_string());
        Ok(())
    }

    /// Currently held by active reservations.
    pub fn reserved(&self) -> Money {
        self.money(self.reserved_total())
    }

    /// Committed spend.
    pub fn spent(&self) -> Money {
        self.money(self.spent)
    }

    /// All-or-nothing hold against available capacity (R-055).
    pub fn reserve(&mut self, id: &str, amount: Money) -> Result<(), BudgetError> {
        self.check_amount(&amount)?;
        if self.reservations.iter().any(|r| r.id == id) {
            return Err(BudgetError::DuplicateReservation { id: id.to_string() });
        }
        let available = self.available_minor()?;
        if amount.minor_units > available {
            return Err(BudgetError::InsufficientAvailable {
                requested: amount,
                available: self.money(available),
            });
        }
        self.reservations.push(Reservation {
            id: id.to_string(),
            amount: amount.minor_units,
        });
        Ok(())
    }

    /// Settle a reservation as spend, exactly up to its reserved amount;
    /// the remainder returns to available capacity.
    pub fn commit(&mut self, id: &str, actual: Money) -> Result<(), BudgetError> {
        self.check_amount(&actual)?;
        let index = self
            .reservations
            .iter()
            .position(|r| r.id == id)
            .ok_or_else(|| BudgetError::NotFound { id: id.to_string() })?;
        let reserved_amount = self.reservations[index].amount;
        if actual.minor_units > reserved_amount {
            return Err(BudgetError::ReservationExceeded {
                reserved: self.money(reserved_amount),
                actual,
            });
        }
        // Everything validated before the first mutation — no restore path
        // can be needed (review pass 1, latent).
        let new_spent = self
            .spent
            .checked_add(actual.minor_units)
            .ok_or(BudgetError::InvalidAmount)?;
        self.reservations.remove(index);
        self.spent = new_spent;
        Ok(())
    }

    /// Return a reservation's capacity untouched (no spend recorded).
    pub fn release(&mut self, id: &str) -> Result<(), BudgetError> {
        let index = self
            .reservations
            .iter()
            .position(|r| r.id == id)
            .ok_or_else(|| BudgetError::NotFound { id: id.to_string() })?;
        self.reservations.remove(index);
        Ok(())
    }

    fn check_amount(&self, amount: &Money) -> Result<(), BudgetError> {
        if amount.minor_units <= 0 {
            return Err(BudgetError::InvalidAmount);
        }
        if amount.currency != self.currency {
            return Err(BudgetError::CurrencyMismatch {
                expected: self.currency.clone(),
                found: amount.currency.clone(),
            });
        }
        Ok(())
    }

    fn unresolved_known_total(&self) -> i64 {
        self.unresolved
            .iter()
            .filter_map(|c| c.amount)
            .try_fold(0i64, |acc, a| acc.checked_add(a))
            .expect("invariant: unresolved amounts fit — each bounded by available")
    }

    fn reserved_total(&self) -> i64 {
        self.reservations
            .iter()
            .try_fold(0i64, |acc, r| acc.checked_add(r.amount))
            .expect("invariant: reservations sum fits — each is bounded by limit")
    }

    fn money(&self, minor_units: i64) -> Money {
        Money {
            currency: self.currency.clone(),
            minor_units,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd(minor_units: i64) -> Money {
        Money {
            currency: "USD".to_string(),
            minor_units,
        }
    }

    #[test]
    fn currency_pattern_is_enforced_on_the_limit() {
        assert!(BudgetLedger::new(usd(0)).is_ok(), "zero limit allowed");
        for bad in ["", "US", "USDD", "usd", "U5D"] {
            assert!(
                matches!(
                    BudgetLedger::new(Money {
                        currency: bad.to_string(),
                        minor_units: 1
                    }),
                    Err(BudgetError::CurrencyMismatch { .. })
                ),
                "rejects {bad:?}"
            );
        }
    }

    #[test]
    fn sums_are_bounded_by_the_limit_through_cycles() {
        let mut ledger = BudgetLedger::new(usd(10)).expect("new");
        for (round, spend) in [(0, 3), (1, 2), (2, 1)].into_iter() {
            let avail = ledger.available().minor_units;
            ledger
                .reserve("hold", usd(avail))
                .expect("reserve against current availability");
            ledger.commit("hold", usd(spend)).expect("commit under cap");
            assert_eq!(
                ledger.available().minor_units
                    + ledger.reserved().minor_units
                    + ledger.spent().minor_units,
                10,
                "cycle {round}: four-bucket sum holds"
            );
        }
        assert_eq!(ledger.spent(), usd(6));
        // The bound still refuses the remainder that is already spent:
        assert!(ledger.reserve("over", usd(5)).is_err());
        assert!(ledger.reserve("fits", usd(4)).is_ok());
    }

    #[test]
    fn boundary_limit_fits_the_integer_type() {
        // Arithmetic edge: a maximal legal limit behaves without overflow.
        let mut ledger = BudgetLedger::new(usd(i64::MAX)).expect("new");
        ledger
            .reserve("all", usd(i64::MAX))
            .expect("hold everything");
        assert_eq!(ledger.available().minor_units, 0);
        ledger.release("all").expect("release");
        assert_eq!(ledger.available().minor_units, i64::MAX);
        ledger.reserve("again", usd(i64::MAX)).expect("re-hold");
        assert_eq!(ledger.reserved().minor_units, i64::MAX);
    }

    #[test]
    fn refusals_leave_state_untouched() {
        let mut ledger = BudgetLedger::new(usd(50)).expect("new");
        ledger.reserve("R", usd(50)).expect("reserve");
        let snapshot = format!("{:?}", ledger);
        for _ in 0..3 {
            assert!(ledger.reserve("R", usd(1)).is_err());
            assert!(ledger.reserve("X", usd(51)).is_err());
            assert!(ledger.commit("R", usd(51)).is_err());
            assert!(ledger.release("GONE").is_err());
        }
        assert_eq!(format!("{:?}", ledger), snapshot, "refusals are pure");
    }
}
