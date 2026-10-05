//! Parity tests against `tests/test_traceability.py` (T-005,
//! R-113/AT-113: removed anchors, reversed mappings, or missing
//! positive/negative cases fail the package checks).
//!
//! Mirrors every Python reference test with the same reason codes, plus
//! extra negatives for codes the reference defines but does not exercise.

use std::path::PathBuf;

use hephaestus::traceability::{AcceptanceCase, Requirement, load_package, validate_traceability};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn package() -> (
    Vec<Requirement>,
    Vec<AcceptanceCase>,
    hephaestus::traceability::ReleaseScopes,
) {
    let pkg = load_package(&root());
    (pkg.requirements, pkg.tests, pkg.scopes)
}

fn errors(
    reqs: &[Requirement],
    tests: &[AcceptanceCase],
    scopes: &hephaestus::traceability::ReleaseScopes,
) -> Vec<String> {
    validate_traceability(reqs, tests, scopes, &root(), true)
}

fn rejects(errors: &[String], code: &str) {
    assert!(
        errors.iter().any(|e| e.starts_with(code)),
        "expected {code}, got {errors:?}"
    );
}

#[test]
fn current_package_consistent() {
    let (reqs, tests, scopes) = package();
    let errors = errors(&reqs, &tests, &scopes);
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(reqs.len(), 119);
    assert_eq!(tests.len(), 119);
    assert_eq!(scopes.scopes.len(), 7);
}

#[test]
fn reverse_requirement_mapping_required() {
    let (reqs, mut tests, scopes) = package();
    tests[0].requirement_ids = vec!["R-002".to_string()];
    rejects(&errors(&reqs, &tests, &scopes), "NONRECIPROCAL_ACCEPTANCE");
}

#[test]
fn reverse_test_mapping_required() {
    let (reqs, mut tests, scopes) = package();
    tests[0].requirement_ids.push("R-002".to_string());
    rejects(&errors(&reqs, &tests, &scopes), "NONRECIPROCAL_REQUIREMENT");
}

#[test]
fn clause_anchor_must_exist() {
    // R-081/AT-081 negative case: run traceability validation over the
    // specification package -- a missing clause anchor fails validation.
    let (mut reqs, tests, scopes) = package();
    reqs[0].clause_anchor = "docs/OBLIGATIONS.md#missing-clause".to_string();
    rejects(&errors(&reqs, &tests, &scopes), "MISSING_CLAUSE_ANCHOR");
}

#[test]
fn source_anchor_missing_file_also_denies() {
    let (mut reqs, tests, scopes) = package();
    reqs[0].source_anchor = "NOPE.md#section-1".to_string();
    rejects(&errors(&reqs, &tests, &scopes), "MISSING_CLAUSE_ANCHOR");
}

#[test]
fn positive_and_negative_cases_required() {
    for field in ["positive_case", "negative_case"] {
        let (reqs, mut tests, scopes) = package();
        if field == "positive_case" {
            tests[0].positive_case = String::new();
        } else {
            tests[0].negative_case = String::new();
        }
        rejects(
            &errors(&reqs, &tests, &scopes),
            "INCOMPLETE_ACCEPTANCE_CASE",
        );
    }
}

#[test]
fn incomplete_clause_detected() {
    let (mut reqs, tests, scopes) = package();
    reqs[0].owner = String::new();
    rejects(&errors(&reqs, &tests, &scopes), "INCOMPLETE_CLAUSE");
}

#[test]
fn duplicate_ids_detected() {
    let (mut reqs, tests, scopes) = package();
    let clone = reqs[0].clone();
    reqs.push(clone);
    rejects(&errors(&reqs, &tests, &scopes), "DUPLICATE_TRACEABILITY_ID");
}

#[test]
fn end_to_end_obligation_is_not_m0() {
    let (reqs, _tests, scopes) = package();
    let r080 = reqs.iter().find(|r| r.id == "R-080").expect("R-080 exists");
    assert_eq!(r080.runtime_milestone, "M3");
    assert!(!scopes.scopes[0].runtime_tests.iter().any(|t| t == "AT-080"));
}

#[test]
fn release_cannot_omit_a_due_gate() {
    let (reqs, tests, mut scopes) = package();
    scopes.scopes[4].runtime_requirements.pop();
    rejects(
        &errors(&reqs, &tests, &scopes),
        "RELEASE_SCOPE_COVERAGE_MISMATCH",
    );
}

#[test]
fn release_scope_set_must_be_complete() {
    let (reqs, tests, mut scopes) = package();
    scopes.scopes.pop();
    rejects(
        &errors(&reqs, &tests, &scopes),
        "RELEASE_SCOPE_SET_INCOMPLETE",
    );
}

#[test]
fn m5_milestone_not_required_for_m0_through_m4_completion() {
    // R-079/AT-079 negative case: inspect the milestone dependencies --
    // no requirement with an M5 (or later) runtime milestone may appear
    // in the M0..M4 runtime sets, so M5 is never required for M0-M4
    // core completion.
    let (reqs, _tests, scopes) = package();
    let late: Vec<&str> = reqs
        .iter()
        .filter(|r| r.runtime_milestone.as_str() > "M4")
        .map(|r| r.id.as_str())
        .collect();
    assert!(!late.is_empty(), "package has post-M4 requirements");
    for scope in scopes.scopes.iter().filter(|s| s.id.as_str() <= "M4") {
        for id in &late {
            assert!(
                !scope.runtime_requirements.iter().any(|r| r == id),
                "{id} must not gate {}",
                scope.id
            );
        }
    }
}

#[test]
fn invalid_scope_id_denied() {
    let (reqs, tests, mut scopes) = package();
    scopes.scopes[0].id = "M9".to_string();
    rejects(&errors(&reqs, &tests, &scopes), "RELEASE_SCOPE_INVALID");
}

#[test]
fn release_scope_cannot_lose_contract_coverage() {
    let (reqs, tests, mut scopes) = package();
    scopes.scopes[0].contract_requirements.pop();
    rejects(
        &errors(&reqs, &tests, &scopes),
        "RELEASE_SCOPE_COVERAGE_MISMATCH",
    );
}

#[test]
fn missing_supplement_denied() {
    let (mut reqs, tests, scopes) = package();
    let idx = reqs
        .iter()
        .position(|r| r.supplement.is_some())
        .expect("package has supplemented requirements");
    reqs[idx].supplement = Some("docs/NOT_A_SUPPLEMENT.md".to_string());
    rejects(&errors(&reqs, &tests, &scopes), "MISSING_SUPPLEMENT");
}

#[test]
fn generated_text_drift_detected() {
    let (mut reqs, tests, scopes) = package();
    reqs[0].statement = "Silently changed meaning.".to_string();
    rejects(&errors(&reqs, &tests, &scopes), "GENERATED_DOCUMENT_DRIFT");
}

#[test]
fn rendered_documents_match_the_committed_files() {
    let (reqs, tests, _scopes) = package();
    for (path, text) in hephaestus::traceability::render_documents(&reqs, &tests) {
        let existing = std::fs::read_to_string(root().join(&path))
            .unwrap_or_else(|e| panic!("{path} readable: {e}"));
        assert_eq!(existing, text, "drift in {path}");
    }
}
