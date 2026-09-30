//! Capability grants and the deterministic policy engine (T-007).
//!
//! The runtime decision core: typed grants and a pure evaluation function.
//! Contract-layer JSON checks stay in [`crate::security`]; this module never
//! reads a system clock (time comes from the caller's context) and carries no
//! credential or provider inputs (R-052).

pub mod engine;
pub mod grant;

pub use engine::{MissionState, PolicyDecision, PolicyEngine, PolicyRequest, ReasonCode};
pub use grant::{CapabilityGrant, GrantError, GrantSpec};
