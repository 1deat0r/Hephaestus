//! AE-07 S1 — protect generation and evaluation separation.
//!
//! Generators never reach protected evaluation artifacts.
//! Advisory-only authority stays intact. Qualification gates bind
//! the current artifact version; stale versions refuse.

use hephaestus::security::sealed::{
    AccessError, AccessRequest, ArtifactRef, Role, authorize_access,
};

fn artifact(id: &str, version: i64, protected: bool) -> ArtifactRef {
    ArtifactRef {
        id: id.to_string(),
        version,
        protected,
    }
}

fn req(role: Role, artifact: ArtifactRef, current: i64) -> AccessRequest {
    AccessRequest {
        role,
        artifact,
        current_version: current,
    }
}

#[test]
fn s1_isolation_allows_legitimate_access() {
    // AC 19 positive: evaluators and reviewers read protected
    // current artifacts; generators read unprotected current work.
    assert!(authorize_access(&req(Role::Evaluator, artifact("eval-bundle", 3, true), 3,)).is_ok());
    assert!(authorize_access(&req(Role::Reviewer, artifact("eval-bundle", 3, true), 3,)).is_ok());
    assert!(authorize_access(&req(Role::Generator, artifact("work-draft", 7, false), 7,)).is_ok());
}

#[test]
fn s1_isolation_refuses_generator_and_stale_versions() {
    // AC 19 refusal: generator access to protected artifacts fails
    // with the artifact named, at any version.
    match authorize_access(&req(Role::Generator, artifact("eval-bundle", 3, true), 3)) {
        Err(AccessError::GeneratorDeniedProtected { artifact: id }) => {
            assert_eq!(id, "eval-bundle");
        }
        other => panic!("expected GeneratorDeniedProtected, got: {other:?}"),
    }
    // Stale versions refuse for every role, with versions named.
    for role in [Role::Generator, Role::Evaluator, Role::Reviewer] {
        match authorize_access(&req(role, artifact("work-draft", 6, false), 7)) {
            Err(AccessError::StaleVersion { requested, current }) => {
                assert_eq!((requested, current), (6, 7));
            }
            other => panic!("expected StaleVersion, got: {other:?}"),
        }
    }
}

#[test]
fn s1_isolation_denial_ignores_version_currency() {
    // AC 19 regression: role denial precedes version checks, so a
    // generator never learns whether its version is current; twin
    // calls agree on every outcome.
    let stale = req(Role::Generator, artifact("eval-bundle", 2, true), 3);
    let first = authorize_access(&stale);
    let second = authorize_access(&stale);
    assert_eq!(first, second);
    assert!(matches!(
        first,
        Err(AccessError::GeneratorDeniedProtected { .. })
    ));
}
