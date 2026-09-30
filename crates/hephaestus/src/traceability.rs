//! Traceability validation (T-005, R-113).
//!
//! Rust port of `tools/traceability.py`: imports the 119 requirement/test
//! mappings and the release-scope register, and fails on missing, orphaned,
//! non-reciprocal, unmapped-clause, or scope-coverage problems. The Python
//! reference remains the package's own check; this port makes the same
//! obligations enforceable from the runtime side, with parity tests in
//! `tests/traceability.rs`.

use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;

/// One obligation from `requirements.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub statement: String,
    pub spec_section: i64,
    pub owner: String,
    pub milestone: String,
    pub contract_milestone: String,
    pub runtime_milestone: String,
    pub clause_anchor: String,
    pub source_anchor: String,
    pub enforcement_service: String,
    pub acceptance_tests: Vec<String>,
    #[serde(default)]
    pub supplement: Option<String>,
}

/// One acceptance case from `acceptance-tests.json` (positive/negative
/// coverage rationale is part of the required contract).
#[derive(Debug, Clone, Deserialize)]
pub struct AcceptanceCase {
    pub id: String,
    pub title: String,
    pub requirement_ids: Vec<String>,
    pub setup: String,
    pub action: String,
    pub expected: String,
    pub positive_case: String,
    pub negative_case: String,
}

/// One milestone entry from `release-scopes.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Scope {
    pub id: String,
    pub contract_requirements: Vec<String>,
    pub runtime_requirements: Vec<String>,
    pub runtime_tests: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseScopes {
    pub scopes: Vec<Scope>,
}

#[derive(Debug, Clone, Deserialize)]
struct RequirementsFile {
    requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, Deserialize)]
struct TestsFile {
    tests: Vec<AcceptanceCase>,
}

/// Loaded package mappings.
#[derive(Debug, Clone)]
pub struct TraceabilityPackage {
    pub requirements: Vec<Requirement>,
    pub tests: Vec<AcceptanceCase>,
    pub scopes: ReleaseScopes,
}

/// Load the mapping files from a package root.
pub fn load_package(root: &Path) -> TraceabilityPackage {
    let read = |rel: &str| {
        std::fs::read_to_string(root.join(rel))
            .unwrap_or_else(|e| panic!("read {rel} from {}: {e}", root.display()))
    };
    let requirements: RequirementsFile =
        serde_json::from_str(&read("requirements.json")).expect("requirements.json valid");
    let tests: TestsFile =
        serde_json::from_str(&read("acceptance-tests.json")).expect("acceptance-tests.json valid");
    let scopes: ReleaseScopes =
        serde_json::from_str(&read("release-scopes.json")).expect("release-scopes.json valid");
    TraceabilityPackage {
        requirements: requirements.requirements,
        tests: tests.tests,
        scopes,
    }
}

fn anchor_present(root: &Path, target: &str) -> bool {
    let Some((path, anchor)) = target.split_once('#') else {
        return false;
    };
    if anchor.is_empty() {
        return false;
    }
    let Ok(text) = std::fs::read_to_string(root.join(path)) else {
        return false;
    };
    text.contains(&format!("id=\"{anchor}\""))
}

/// Validate the mappings. Returns reason-coded errors; an empty vec means
/// the package's traceability holds.
pub fn validate_traceability(
    requirements: &[Requirement],
    tests: &[AcceptanceCase],
    scopes: &ReleaseScopes,
    root: &Path,
) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    let reqs: std::collections::HashMap<&str, &Requirement> =
        requirements.iter().map(|r| (r.id.as_str(), r)).collect();
    let cases: std::collections::HashMap<&str, &AcceptanceCase> =
        tests.iter().map(|t| (t.id.as_str(), t)).collect();
    if reqs.len() != requirements.len() || cases.len() != tests.len() {
        errors.push("DUPLICATE_TRACEABILITY_ID".to_string());
    }

    for r in requirements {
        let complete = !r.acceptance_tests.is_empty()
            && r.milestone == r.runtime_milestone
            && !r.owner.is_empty()
            && !r.enforcement_service.is_empty()
            && !r.clause_anchor.is_empty()
            && !r.source_anchor.is_empty()
            && r.contract_milestone == "M0"
            && matches!(
                r.runtime_milestone.as_str(),
                "M0" | "M1" | "M2" | "M3" | "M4" | "M5" | "M6"
            );
        if !complete {
            errors.push(format!("INCOMPLETE_CLAUSE: {}", r.id));
        }
        for tid in &r.acceptance_tests {
            let reciprocal = cases
                .get(tid.as_str())
                .is_some_and(|t| t.requirement_ids.iter().any(|id| id == &r.id));
            if !reciprocal {
                errors.push(format!("NONRECIPROCAL_ACCEPTANCE: {}", r.id));
            }
        }
        for field in [&r.clause_anchor, &r.source_anchor] {
            if !anchor_present(root, field) {
                errors.push(format!("MISSING_CLAUSE_ANCHOR: {}", r.id));
            }
        }
        if let Some(supplement) = &r.supplement
            && !root.join(supplement).is_file()
        {
            errors.push(format!("MISSING_SUPPLEMENT: {}", r.id));
        }
    }

    for t in tests {
        let complete = !t.requirement_ids.is_empty()
            && !t.setup.is_empty()
            && !t.action.is_empty()
            && !t.expected.is_empty()
            && !t.positive_case.is_empty()
            && !t.negative_case.is_empty();
        if !complete {
            errors.push(format!("INCOMPLETE_ACCEPTANCE_CASE: {}", t.id));
        }
        for rid in &t.requirement_ids {
            let reciprocal = reqs
                .get(rid.as_str())
                .is_some_and(|r| r.acceptance_tests.iter().any(|id| id == &t.id));
            if !reciprocal {
                errors.push(format!("NONRECIPROCAL_REQUIREMENT: {}", t.id));
            }
        }
    }

    errors.extend(validate_scopes(requirements, scopes));

    errors
}

/// Release-scope coverage: seven milestone scopes, contract completeness,
/// and due-gate runtime coverage (M0..M6, string-ordered like the
/// reference).
pub fn validate_scopes(requirements: &[Requirement], scopes: &ReleaseScopes) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let reqs: std::collections::HashMap<&str, &Requirement> =
        requirements.iter().map(|r| (r.id.as_str(), r)).collect();
    let all_ids: HashSet<&str> = reqs.keys().copied().collect();

    let scope_ids: HashSet<&str> = scopes.scopes.iter().map(|s| s.id.as_str()).collect();
    let expected_ids: HashSet<String> = (0..7).map(|m| format!("M{m}")).collect();
    let expected_ids: HashSet<&str> = expected_ids.iter().map(String::as_str).collect();
    if scope_ids != expected_ids || scopes.scopes.len() != 7 {
        errors.push("RELEASE_SCOPE_SET_INCOMPLETE".to_string());
    }

    for scope in &scopes.scopes {
        let sid = &scope.id;
        if !matches!(sid.as_str(), "M0" | "M1" | "M2" | "M3" | "M4" | "M5" | "M6") {
            errors.push(format!("RELEASE_SCOPE_INVALID: {sid}"));
            continue;
        }
        let due: HashSet<&str> = requirements
            .iter()
            .filter(|r| r.runtime_milestone.as_str() <= sid.as_str())
            .map(|r| r.id.as_str())
            .collect();
        let expected_tests: HashSet<&str> = due
            .iter()
            .flat_map(|rid| reqs[*rid].acceptance_tests.iter().map(String::as_str))
            .collect();
        let contract: HashSet<&str> = scope
            .contract_requirements
            .iter()
            .map(String::as_str)
            .collect();
        let runtime: HashSet<&str> = scope
            .runtime_requirements
            .iter()
            .map(String::as_str)
            .collect();
        let runtime_tests: HashSet<&str> = scope.runtime_tests.iter().map(String::as_str).collect();
        if contract != all_ids || runtime != due || runtime_tests != expected_tests {
            errors.push(format!("RELEASE_SCOPE_COVERAGE_MISMATCH: {sid}"));
        }
    }

    errors
}
