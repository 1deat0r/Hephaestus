//! Scoped grant authorization (T-004, R-099 dispatch-time rules).
//!
//! Port of `reference/qualification.py::check_grant` plus the grant interval
//! rule. Fail-closed: trust comes from the [`TrustContext`], clock facts from
//! its `evaluated_at`, and every binding must match exactly. These checks
//! authorize *before* dispatch or effect — no worker grants itself anything.
//!
//! The facet checks live in the pure predicates below (extracted by T-007 so
//! the [`crate::policy`] engine shares this single implementation instead of
//! forking it). This function keeps its aggregate reason codes; the engine
//! reports the per-facet granularity AT-060 asks for. On contract-shaped
//! input the two must agree — a consistency test pins that.

use serde_json::Value;
use time::OffsetDateTime;

use super::trust::TrustContext;

/// Everything a single grant check compares against.
pub struct GrantCheck<'a> {
    /// The record seeking authorization (task / plan / improvement).
    pub record: &'a Value,
    /// The mission the grant must bind to.
    pub mission: &'a Value,
    /// The resolved `authorization_grant` record.
    pub grant: &'a Value,
    /// Approval/intervention artifact digest (nullable per contract).
    pub artifact: Option<&'a str>,
    /// Capabilities the operation requires.
    pub capabilities: &'a [&'a str],
    /// Operation identity the grant must cover.
    pub operation: &'a str,
    /// Execution destination (nullable per contract).
    pub destination: Option<&'a str>,
}

fn record_id(record: &Value) -> String {
    record
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>")
        .to_string()
}

fn error(code: &str, record: &Value) -> String {
    format!("{code}: {}", record_id(record))
}

fn ref_of(record: &Value) -> Value {
    serde_json::json!({
        "id": record.get("id").and_then(Value::as_str).unwrap_or(""),
        "version": record.get("record_version").and_then(Value::as_i64).unwrap_or(-1),
    })
}

// --- Shared facet predicates (T-007 extraction) -----------------------------
//
// Pure: facts in, verdict out. The clock is a parameter, trust and I/O are
// absent by construction. Used by `authorize_grant` (aggregate codes) and by
// the deterministic policy engine (specific reason codes).

/// What a grant's destination field holds, in either layer's vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationBinding<'a> {
    /// The field is absent.
    Missing,
    /// JSON null / typed `None`: no destination is bound.
    Unbound,
    /// A destination string, or — when borrowed from malformed JSON —
    /// present but not a string (`Set(None)`), which never binds.
    Set(Option<&'a str>),
}

/// `mission_ref` binds iff the grant carries exactly this `{id, version}`.
/// Contract-shaped input only: a grant ref with extra keys is out of
/// contract (the typed layer cannot produce one).
pub fn refs_bound(grant_mission_ref: Option<(&str, i64)>, mission: (&str, i64)) -> bool {
    grant_mission_ref == Some(mission)
}

/// Grant state must be `approved`.
pub fn grant_state_approved(state: Option<&str>) -> bool {
    state == Some("approved")
}

/// Mission authorization state must be `approved`.
pub fn mission_state_approved(state: Option<&str>) -> bool {
    state == Some("approved")
}

/// The policy chain must be fully linked: grant present, equal to the
/// mission's policy, and equal to the context's current policy for the
/// mission.
pub fn policy_chain_bound(
    grant_policy: Option<&str>,
    mission_policy: Option<&str>,
    context_policy: Option<&str>,
) -> bool {
    grant_policy.is_some()
        && grant_policy == mission_policy
        && context_policy.is_some()
        && context_policy == grant_policy
}

/// The grant's operation must be exactly the wanted operation.
pub fn operation_bound(grant_operation: Option<&str>, wanted: &str) -> bool {
    grant_operation == Some(wanted)
}

/// Artifact binding: both sides must agree, including both-unbound.
pub fn artifact_bound(grant_artifact: Option<&str>, wanted: Option<&str>) -> bool {
    grant_artifact == wanted
}

/// Destination binding: `Unbound` binds only a `None` request; `Set` binds
/// an equal string request; `Missing` and malformed bind nothing.
pub fn destination_bound(grant_destination: DestinationBinding<'_>, wanted: Option<&str>) -> bool {
    match wanted {
        Some(wanted) => {
            matches!(grant_destination, DestinationBinding::Set(Some(got)) if got == wanted)
        }
        None => matches!(grant_destination, DestinationBinding::Unbound),
    }
}

/// The mission's `allowed_destinations` must contain the destination.
pub fn destination_allowed(allowed: &[&str], wanted: &str) -> bool {
    allowed.contains(&wanted)
}

/// Every required capability must appear in the allowed list; a non-list
/// allowed side allows nothing.
pub fn capabilities_subset(required: &[&str], allowed: Option<&[&str]>) -> bool {
    allowed.is_some_and(|allowed| required.iter().all(|cap| allowed.contains(cap)))
}

/// Cost must match currency and stay within the cap (`used <= cap`).
/// Missing either side is not within cap.
pub fn minor_cost_within(cap: Option<(&str, i64)>, used: Option<(&str, i64)>) -> bool {
    match (cap, used) {
        (Some((cap_currency, cap_units)), Some((used_currency, used_units))) => {
            cap_currency == used_currency && used_units <= cap_units
        }
        _ => false,
    }
}

/// `issued <= now < expires`, all present and parseable — otherwise false
/// (fail-closed: an unknown clock or bad timestamps never authorize).
pub fn window_valid(
    issued: Option<&str>,
    expires: Option<&str>,
    now: Option<OffsetDateTime>,
) -> bool {
    let (Some(now), Some(issued), Some(expires)) = (now, issued, expires) else {
        return false;
    };
    match (parse_rfc3339(issued), parse_rfc3339(expires)) {
        (Ok(issued), Ok(expires)) => issued <= now && now < expires,
        _ => false,
    }
}

/// A non-empty, ordered window: `issued < expires`, parseable, else false.
pub fn interval_ordered(issued: Option<&str>, expires: Option<&str>) -> bool {
    match (issued, expires) {
        (Some(issued), Some(expires)) => match (parse_rfc3339(issued), parse_rfc3339(expires)) {
            (Ok(issued), Ok(expires)) => issued < expires,
            _ => false,
        },
        _ => false,
    }
}

fn parse_rfc3339(value: &str) -> Result<OffsetDateTime, time::error::Parse> {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
}

// --- JSON adapters for the contract layer ----------------------------------

/// Exact `{id, version}` shape and nothing else: the JSON layer used to
/// compare whole values, so a ref carrying extra keys must NOT bind. This
/// keeps the T-007 extraction behaviorally identical to the pre-extraction
/// check (review pass 1: fail-open restoration).
fn ref_pair(value: &Value) -> Option<(&str, i64)> {
    let object = value.as_object()?;
    if object.len() != 2 {
        return None;
    }
    Some((
        object.get("id")?.as_str()?,
        object.get("version")?.as_i64()?,
    ))
}

fn string_list(value: &Value) -> Option<Vec<&str>> {
    value
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_str).collect())
}

fn money_pair(value: &Value) -> Option<(&str, i64)> {
    Some((
        value.get("currency")?.as_str()?,
        value.get("minor_units")?.as_i64()?,
    ))
}

fn destination_facet(value: Option<&Value>) -> DestinationBinding<'_> {
    match value {
        None => DestinationBinding::Missing,
        Some(Value::Null) => DestinationBinding::Unbound,
        Some(other) => DestinationBinding::Set(other.as_str()),
    }
}

/// Authorize one operation against its grant. Returns reason-coded errors;
/// empty means the exact scoped grant currently holds.
pub fn authorize_grant(check: &GrantCheck<'_>, ctx: &TrustContext) -> Vec<String> {
    let GrantCheck {
        record,
        mission,
        grant,
        artifact,
        capabilities,
        operation,
        destination,
    } = check;
    let mut errors = Vec::new();

    // Trust: the grant must be externally trusted, not merely present.
    let trusted = record
        .get("grant_ref")
        .and_then(|r| {
            let id = r.get("id")?.as_str()?;
            let version = r.get("version")?.as_i64()?;
            Some(ctx.trusted_grant(id, version, grant))
        })
        .unwrap_or(false);
    if !trusted {
        errors.push(error("GRANT_NOT_TRUSTED", record));
        return errors;
    }

    let auth = mission.get("authorization");
    let mission_ref_value = ref_of(mission);
    let mission_ref = ref_pair(&mission_ref_value).unwrap_or(("", -1));
    let mission_ref_ok = refs_bound(grant.get("mission_ref").and_then(ref_pair), mission_ref);

    let grant_policy = grant.get("policy_version").and_then(Value::as_str);
    let mission_policy = auth
        .and_then(|a| a.get("policy_version"))
        .and_then(Value::as_str);
    let ctx_policy = mission
        .get("id")
        .and_then(Value::as_str)
        .and_then(|mission_id| ctx.current_policy(mission_id));

    let binding_ok = mission_ref_ok
        && grant_state_approved(grant.get("state").and_then(Value::as_str))
        && policy_chain_bound(grant_policy, mission_policy, ctx_policy)
        && artifact_bound(
            grant.get("artifact_sha256").and_then(Value::as_str),
            *artifact,
        )
        && operation_bound(grant.get("operation_id").and_then(Value::as_str), operation)
        && destination_bound(destination_facet(grant.get("destination")), *destination)
        && capabilities_subset(
            capabilities,
            grant.get("capabilities").and_then(string_list).as_deref(),
        )
        && capabilities_subset(
            capabilities,
            auth.and_then(|a| a.get("allowed_capabilities"))
                .and_then(string_list)
                .as_deref(),
        )
        && mission_state_approved(auth.and_then(|a| a.get("state")).and_then(Value::as_str));
    if !binding_ok {
        errors.push(error("GRANT_BINDING_MISMATCH", record));
    }

    if let Some(wanted) = destination {
        let allowed = auth
            .and_then(|a| a.get("allowed_destinations"))
            .and_then(string_list)
            .unwrap_or_default();
        if !destination_allowed(&allowed, wanted) {
            errors.push(error("DESTINATION_NOT_GRANTED", record));
        }
    }

    // Budget cap against the grant (per-record cap, not reservations).
    let Some(budget) = record.get("budget") else {
        // Schema-valid records always carry a budget where required; a
        // missing one here fails closed rather than skipping the cost rule.
        errors.push(error("GRANT_COST_EXCEEDED", record));
        return errors;
    };
    let Some(max_cost) = grant.get("max_cost") else {
        errors.push(error("GRANT_COST_EXCEEDED", record));
        return errors;
    };
    if !minor_cost_within(money_pair(max_cost), money_pair(budget)) {
        errors.push(error("GRANT_COST_EXCEEDED", record));
    }

    // Time window: issued <= now < expires, with an external clock.
    let window_ok = window_valid(
        grant.get("issued_at").and_then(Value::as_str),
        grant.get("expires_at").and_then(Value::as_str),
        ctx.evaluated_at(),
    );
    if !window_ok {
        errors.push(error("GRANT_EXPIRED_OR_TIME_UNKNOWN", record));
    }

    errors
}

/// `GRANT_INVALID_INTERVAL`: a grant whose window is empty or reversed can
/// never authorize anything. Returns the error or `None`.
pub fn check_grant_interval(grant: &Value) -> Option<String> {
    let issued = grant.get("issued_at").and_then(Value::as_str);
    let expires = grant.get("expires_at").and_then(Value::as_str);
    if interval_ordered(issued, expires) {
        None
    } else {
        Some(error("GRANT_INVALID_INTERVAL", grant))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::format_description::well_known::Rfc3339;

    fn now() -> OffsetDateTime {
        OffsetDateTime::parse("2026-09-30T12:00:00Z", &Rfc3339).expect("fixture clock")
    }

    #[test]
    fn cost_cap_boundaries() {
        let cap = Some(("USD", 500));
        assert!(
            minor_cost_within(cap, Some(("USD", 500))),
            "equal is within"
        );
        assert!(
            !minor_cost_within(cap, Some(("USD", 501))),
            "over cap denies"
        );
        assert!(
            !minor_cost_within(cap, Some(("EUR", 400))),
            "currency mismatch denies"
        );
        assert!(!minor_cost_within(cap, None), "missing used denies");
        assert!(
            !minor_cost_within(None, Some(("USD", 1))),
            "missing cap denies"
        );
    }

    #[test]
    fn window_edges_are_exactly_the_contract() {
        let now = now();
        assert!(window_valid(
            Some("2026-09-30T00:00:00Z"),
            Some("2026-10-01T00:00:00Z"),
            Some(now)
        ));
        assert!(
            window_valid(
                Some("2026-09-30T12:00:00Z"),
                Some("2026-10-01T00:00:00Z"),
                Some(now),
            ),
            "issued <= now holds"
        );
        assert!(
            !window_valid(
                Some("2026-09-30T12:00:00Z"),
                Some("2026-09-30T12:00:00Z"),
                Some(now),
            ),
            "now < expires is strict"
        );
        assert!(!window_valid(None, Some("2026-10-01T00:00:00Z"), Some(now)));
        assert!(
            !window_valid(
                Some("2026-09-30T00:00:00Z"),
                Some("2026-10-01T00:00:00Z"),
                None
            ),
            "unknown clock denies"
        );
        assert!(
            !window_valid(Some("garbage"), Some("2026-10-01T00:00:00Z"), Some(now)),
            "unparseable denies"
        );
    }

    #[test]
    fn interval_requires_two_ordered_timestamps() {
        assert!(interval_ordered(
            Some("2026-09-30T00:00:00Z"),
            Some("2026-10-01T00:00:00Z")
        ));
        assert!(
            !interval_ordered(Some("2026-10-01T00:00:00Z"), Some("2026-10-01T00:00:00Z")),
            "empty window"
        );
        assert!(
            !interval_ordered(Some("2026-10-02T00:00:00Z"), Some("2026-10-01T00:00:00Z")),
            "reversed window"
        );
        assert!(!interval_ordered(None, Some("2026-10-01T00:00:00Z")));
    }

    #[test]
    fn ref_pair_requires_the_exact_ref_shape() {
        // T-007 review pass 1: the JSON adapter must keep whole-value
        // strictness — a mission_ref carrying extra keys must not bind.
        assert_eq!(
            ref_pair(&serde_json::json!({"id": "MIS-1", "version": 1})),
            Some(("MIS-1", 1))
        );
        assert_eq!(
            ref_pair(&serde_json::json!({"id": "MIS-1", "version": 1, "x": 1})),
            None,
            "extra keys must not bind"
        );
        assert_eq!(ref_pair(&serde_json::json!({"id": "MIS-1"})), None);
        assert_eq!(ref_pair(&serde_json::json!("MIS-1")), None);
        assert_eq!(ref_pair(&serde_json::json!(null)), None);
    }

    #[test]
    fn malformed_cost_never_authorizes() {
        // Documented tightening vs pre-extraction T-004: the old inline check
        // compared raw currency Values, so a record with BOTH currencies
        // absent could pass `currency_ok` (Null == Null). The shared
        // predicate requires both sides present — strictly fail-closed
        // (review pass 1, decision row recorded).
        assert!(
            !minor_cost_within(None, None),
            "absent sides never authorize"
        );
        assert!(!minor_cost_within(Some(("USD", 1)), None));
        assert!(minor_cost_within(Some(("USD", 1)), Some(("USD", 1))));
    }

    #[test]
    fn subset_and_destination_facets() {
        assert!(capabilities_subset(&["a"], Some(&["a", "b"])));
        assert!(
            capabilities_subset(&[], Some(&["a"])),
            "vacuous subset holds"
        );
        assert!(!capabilities_subset(&["c"], Some(&["a"])));
        assert!(
            !capabilities_subset(&["a"], None),
            "non-list allows nothing"
        );

        let set = DestinationBinding::Set(Some("local"));
        assert!(destination_bound(set, Some("local")));
        assert!(!destination_bound(set, Some("remote")));
        assert!(!destination_bound(set, None));
        assert!(destination_bound(DestinationBinding::Unbound, None));
        assert!(!destination_bound(
            DestinationBinding::Unbound,
            Some("local")
        ));
        assert!(
            !destination_bound(DestinationBinding::Missing, None),
            "missing never binds"
        );
        assert!(
            !destination_bound(DestinationBinding::Set(None), None),
            "malformed never binds"
        );
    }

    #[test]
    fn policy_chain_and_refs() {
        assert!(policy_chain_bound(Some("PV-1"), Some("PV-1"), Some("PV-1")));
        assert!(!policy_chain_bound(
            Some("PV-1"),
            Some("PV-2"),
            Some("PV-1")
        ));
        assert!(
            !policy_chain_bound(Some("PV-1"), Some("PV-1"), None),
            "no context policy"
        );
        assert!(!policy_chain_bound(None, None, None), "absent grant policy");
        assert!(refs_bound(Some(("MIS-1", 1)), ("MIS-1", 1)));
        assert!(!refs_bound(Some(("MIS-1", 2)), ("MIS-1", 1)));
        assert!(!refs_bound(None, ("MIS-1", 1)));
    }
}

#[test]
fn destination_facet_maps_contract_shapes() {
    // Typed grants always carry the field; the JSON layer must still
    // distinguish absent / null / string / malformed exactly as the
    // pre-extraction code did (pass-2 residual: Missing never exercised).
    assert_eq!(destination_facet(None), DestinationBinding::Missing);
    assert_eq!(
        destination_facet(Some(&serde_json::Value::Null)),
        DestinationBinding::Unbound
    );
    assert_eq!(
        destination_facet(Some(&serde_json::json!("local"))),
        DestinationBinding::Set(Some("local"))
    );
    assert_eq!(
        destination_facet(Some(&serde_json::json!(7))),
        DestinationBinding::Set(None),
        "present-but-not-a-string never binds"
    );
}
