//! Protected-evaluator access tests (T-003, M0 exit).
//!
//! The permission scope separating hidden ground truth from the control
//! plane is the Cargo dependency graph: `synthetic-evaluator` must never
//! appear in `hephaestus`'s manifest, resolved dependency list, or source.
//! If any of these fail, the control plane has grown a path to hidden truth.

use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn control_plane_manifest_has_no_evaluator_dependency() {
    let manifest =
        std::fs::read_to_string(manifest_dir().join("Cargo.toml")).expect("Cargo.toml readable");
    assert!(
        !manifest.contains("synthetic-evaluator"),
        "hephaestus/Cargo.toml must not depend on the hidden evaluator:\n{manifest}"
    );
}

#[test]
fn resolved_dependency_graph_has_no_evaluator_edge() {
    let lock = std::fs::read_to_string(manifest_dir().join("../../Cargo.lock"))
        .expect("Cargo.lock readable");
    // Isolate the resolved dependency list of the hephaestus package.
    let start = lock
        .find("[[package]]\nname = \"hephaestus\"")
        .expect("hephaestus package entry in lockfile");
    let rest = &lock[start..];
    let block = &rest[..rest.find("[[package]]").unwrap_or(rest.len())];
    assert!(
        !block.contains("synthetic-evaluator"),
        "lockfile resolved synthetic-evaluator for hephaestus:\n{block}"
    );
}

#[test]
fn control_plane_source_never_names_the_evaluator() {
    let src = manifest_dir().join("src");
    let mut files = Vec::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src readable") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    assert!(!files.is_empty());
    for file in files {
        let text = std::fs::read_to_string(&file).expect("source readable");
        assert!(
            !text.contains("synthetic_evaluator") && !text.contains("synthetic-evaluator"),
            "{} references the hidden evaluator",
            file.display()
        );
    }
}

#[test]
fn evaluator_corpus_is_not_shipped_with_the_control_plane() {
    let corpus = manifest_dir().join("corpus");
    assert!(
        !corpus.exists(),
        "hidden corpus must live only under crates/synthetic-evaluator"
    );
    let evaluator_corpus = manifest_dir().join("../synthetic-evaluator/corpus");
    assert!(
        evaluator_corpus.exists(),
        "evaluator corpus exists in its own crate"
    );
}
