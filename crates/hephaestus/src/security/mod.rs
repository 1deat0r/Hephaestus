//! Security model foundations (T-004): digest profile, artifact identity,
//! keyed approvals, external trust context, grant/receipt/sealed-access
//! rules. Fail-closed by construction — candidate records cannot supply the
//! trust, keys, or clocks these checks rely on.

pub mod approval;
pub mod artifact;
pub mod digest;
pub mod grant;
pub mod receipt;
pub mod trust;
