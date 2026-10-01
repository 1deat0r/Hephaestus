//! T-009 ticket 01 — task DAG validation (deny-first, seam: `hephaestus::scheduler`).
//!
//! Plan T-009: acyclicity, dependency outputs, read/write sets, and
//! well-formed declarations — an invalid DAG never reaches a scheduler.
//! Honest absence: DAG validation duties are plan-level; no separately
//! mapped R-ID applies here (the scheduler's R-055/R-056 live in
//! `scheduler_run.rs`).

use hephaestus::contracts::generated::Money;
use hephaestus::scheduler::{DagViolation, PriorityClass, RetryPolicy, Task, TaskDag};

fn usd(minor_units: i64) -> Money {
    Money {
        currency: "USD".to_string(),
        minor_units,
    }
}

fn task(id: &str) -> Task {
    Task {
        id: id.to_string(),
        priority: PriorityClass::Normal,
        depends_on: vec![],
        inputs: vec![],
        outputs: vec![format!("out-{id}")],
        retry: RetryPolicy { max_attempts: 1 },
        timeout_ms: 1_000,
        cost: usd(0),
        trivial: false,
        retryable: true,
        resource_units: 0,
        exclusive: false,
    }
}

fn violations(dag: &TaskDag) -> Vec<DagViolation> {
    dag.validate().err().unwrap_or_default()
}

#[test]
fn valid_linear_and_diamond_dags_validate_clean() {
    let mut dag = TaskDag::new(vec!["seed".to_string()]);
    let mut a = task("A");
    a.inputs = vec!["seed".to_string()];
    let mut b = task("B");
    b.depends_on = vec!["A".to_string()];
    b.inputs = vec!["out-A".to_string()];
    let mut c = task("C");
    c.depends_on = vec!["A".to_string()];
    c.inputs = vec!["out-A".to_string()];
    let mut d = task("D");
    d.depends_on = vec!["B".to_string(), "C".to_string()];
    d.inputs = vec!["out-B".to_string(), "out-C".to_string()];
    for t in [a, b, c, d] {
        dag.add(t).expect("add");
    }
    assert!(dag.validate().is_ok(), "diamond must validate");
}

#[test]
fn cycles_are_named_and_every_other_gap_is_denied() {
    // Cycle:
    let mut dag = TaskDag::new(vec![]);
    let mut a = task("A");
    a.depends_on = vec!["B".to_string()];
    let mut b = task("B");
    b.depends_on = vec!["A".to_string()];
    dag.add(a).unwrap();
    dag.add(b).unwrap();
    assert!(
        violations(&dag)
            .iter()
            .any(|v| matches!(v, DagViolation::Cycle { members } if members.contains(&"A".to_string()) && members.contains(&"B".to_string()))),
        "cycle members named"
    );

    // Dangling dependency:
    let mut dag = TaskDag::new(vec![]);
    let mut a = task("A");
    a.depends_on = vec!["GHOST".to_string()];
    dag.add(a).unwrap();
    assert!(violations(&dag).iter().any(
        |v| matches!(v, DagViolation::MissingDependency { task, missing }
            if task == "A" && missing == "GHOST")
    ));

    // Unsatisfied read (dependency outputs):
    let mut dag = TaskDag::new(vec![]);
    let mut a = task("A");
    a.inputs = vec!["never-produced".to_string()];
    dag.add(a).unwrap();
    assert!(
        violations(&dag)
            .iter()
            .any(|v| matches!(v, DagViolation::UnsatisfiedRead { task, key }
            if task == "A" && key == "never-produced"))
    );

    // Write-write conflict without ordering:
    let mut dag = TaskDag::new(vec![]);
    let mut a = task("A");
    a.outputs = vec!["K".to_string()];
    let mut b = task("B");
    b.outputs = vec!["K".to_string()];
    dag.add(a).unwrap();
    dag.add(b).unwrap();
    assert!(violations(&dag).iter().any(
        |v| matches!(v, DagViolation::UnorderedWriteConflict { key, tasks }
            if key == "K" && tasks.len() == 2)
    ));

    // Malformed declarations:
    let mut dag = TaskDag::new(vec![]);
    let mut a = task("A");
    a.retry = RetryPolicy { max_attempts: 0 };
    a.timeout_ms = 0;
    a.cost = usd(-1);
    dag.add(a).unwrap();
    let found = violations(&dag);
    assert!(
        found
            .iter()
            .any(|v| matches!(v, DagViolation::InvalidDeclaration { task, .. } if task == "A")),
        "declarations denied: {found:?}"
    );
    assert!(found.len() >= 3, "each bad field reported: {found:?}");

    // Duplicate ids refused at add time:
    let mut dag = TaskDag::new(vec![]);
    dag.add(task("SAME")).expect("first");
    assert!(dag.add(task("SAME")).is_err(), "duplicate id refused");
}
