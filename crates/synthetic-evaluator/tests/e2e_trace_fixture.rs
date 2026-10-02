//! E2E trace fixture provenance (T-061): the checked-in control-plane
//! fixture file is bound byte-for-byte to the hidden worlds — the
//! dependency edge forbidden the other way (evaluator_access guard) is
//! replaced by this regeneration contract.

use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../hephaestus/tests/fixtures/e2e-trace.log")
}

#[test]
fn e2e_trace_fixture_matches_the_hidden_worlds() {
    let generated = synthetic_evaluator::trace_fixture();
    let path = fixture_path();
    if std::env::var("HEPHAESTUS_FIXTURE_WRITE").as_deref() == Ok("1") {
        std::fs::create_dir_all(path.parent().expect("fixtures dir")).expect("mkdir");
        std::fs::write(&path, &generated).expect("write fixture");
        return;
    }
    let on_disk = std::fs::read_to_string(&path)
        .expect("checked-in fixture present (regenerate: HEPHAESTUS_FIXTURE_WRITE=1 cargo test -p synthetic-evaluator --test e2e_trace_fixture)");
    assert_eq!(
        on_disk, generated,
        "fixture drifted from load_corpus() worlds — regenerate with HEPHAESTUS_FIXTURE_WRITE=1"
    );
    // The fixture is declared synthetic data, never evidence (examples convention).
    assert!(generated.starts_with("step="), "trace-shaped");
}
