//! The typed task DAG and its fail-closed validation (T-009).
//!
//! Plan T-009's validation list: acyclicity, dependency existence,
//! dependency-output coverage for every read, read/write-set conflicts,
//! and well-formed declarations. Validation is pure; an invalid DAG never
//! reaches the scheduler.

use crate::contracts::generated::Money;

/// Priority classes in drain order (MASTER_SPEC:367's hungry classes).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum PriorityClass {
    /// Critical path work.
    Critical,
    /// Default work.
    Normal,
    /// Verification work (must not starve).
    Verification,
    /// Exploration work (must not starve).
    Exploration,
}

/// Bounded retry policy (no backoff — the scheduler owns no clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RetryPolicy {
    /// Total attempts allowed; `>= 1`. Non-idempotent tasks are capped at
    /// one attempt by the scheduler regardless (MASTER_SPEC:371).
    pub max_attempts: u32,
}

/// One node of the DAG.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Task {
    /// Unique id within the DAG.
    pub id: String,
    /// Priority class (drain order).
    pub priority: PriorityClass,
    /// Ids of tasks that must complete first.
    pub depends_on: Vec<String>,
    /// Keys this task reads.
    pub inputs: Vec<String>,
    /// Keys this task writes.
    pub outputs: Vec<String>,
    /// Retry bound.
    pub retry: RetryPolicy,
    /// Timeout in milliseconds (> 0); enforced by the executor, validated here.
    pub timeout_ms: u64,
    /// Declared cost reserved before dispatch (>= 0).
    pub cost: Money,
    /// Trivial deterministic work eligible for `run_batch` grouping.
    pub trivial: bool,
    /// Whether automatic retries are safe (false ⇒ exactly one attempt).
    pub retryable: bool,
    /// Resource units consumed while running (0 = pure).
    pub resource_units: u32,
    /// Memory (MiB) the task claims. Zero means unspecified — the task
    /// takes no memory reservation. Oversize claims are refused at
    /// admission, exactly like CPU units (S1: two-dimensional envelope;
    /// a wave must fit both dimensions).
    pub memory_mb: u64,
    /// Runs alone: no other task may share its dispatch wave.
    pub exclusive: bool,
}

/// One reason a DAG cannot be scheduled. Every variant names offenders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagViolation {
    /// A dependency cycle; `members` lists its tasks in declaration order.
    Cycle {
        /// The tasks caught in the cycle.
        members: Vec<String>,
    },
    /// `task` depends on `missing`, which is not in the DAG.
    MissingDependency {
        /// The dependent task.
        task: String,
        /// The unknown dependency.
        missing: String,
    },
    /// `task` reads `key`, which no transitive dependency outputs and no
    /// initial input provides.
    UnsatisfiedRead {
        /// The reading task.
        task: String,
        /// The uncovered key.
        key: String,
    },
    /// Two unordered tasks both write `key`.
    UnorderedWriteConflict {
        /// The contended key.
        key: String,
        /// The conflicting writers in declaration order.
        tasks: Vec<String>,
    },
    /// A declaration field is malformed.
    InvalidDeclaration {
        /// The offending task.
        task: String,
        /// What is wrong (one entry per bad field).
        what: String,
    },
}

/// A typed DAG of [`Task`]s with initial inputs available before any task
/// runs.
#[derive(Debug, Clone, Default)]
pub struct TaskDag {
    initial_inputs: Vec<String>,
    tasks: Vec<Task>,
}

impl TaskDag {
    /// Create a DAG whose reads may be satisfied by `initial_inputs`.
    pub fn new(initial_inputs: Vec<String>) -> Self {
        TaskDag {
            initial_inputs,
            tasks: Vec::new(),
        }
    }

    /// Add a task; duplicate ids are refused (fail-closed).
    pub fn add(&mut self, task: Task) -> Result<(), DagViolation> {
        if self.tasks.iter().any(|t| t.id == task.id) {
            return Err(DagViolation::InvalidDeclaration {
                task: task.id,
                what: "duplicate id".to_string(),
            });
        }
        self.tasks.push(task);
        Ok(())
    }

    /// Tasks in declaration order.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Fail-closed validation: every violation found, in a deterministic
    /// order (declarations, then dependencies, then cycles, then reads,
    /// then write conflicts). `Ok(())` means schedulable.
    pub fn validate(&self) -> Result<(), Vec<DagViolation>> {
        let mut out = Vec::new();

        // 1. Declarations.
        for task in &self.tasks {
            if task.retry.max_attempts < 1 {
                out.push(DagViolation::InvalidDeclaration {
                    task: task.id.clone(),
                    what: format!("max_attempts {} < 1", task.retry.max_attempts),
                });
            }
            if task.timeout_ms == 0 {
                out.push(DagViolation::InvalidDeclaration {
                    task: task.id.clone(),
                    what: "timeout_ms must be > 0".to_string(),
                });
            }
            if task.cost.minor_units < 0 {
                out.push(DagViolation::InvalidDeclaration {
                    task: task.id.clone(),
                    what: format!("cost {} < 0", task.cost.minor_units),
                });
            }
        }

        // 2. Dependency existence.
        for task in &self.tasks {
            for dep in &task.depends_on {
                if !self.tasks.iter().any(|t| &t.id == dep) {
                    out.push(DagViolation::MissingDependency {
                        task: task.id.clone(),
                        missing: dep.clone(),
                    });
                }
            }
        }

        // 3. Acyclicity (Kahn); leftover nodes form the cycle set.
        let members = self.cycle_members();
        if !members.is_empty() {
            out.push(DagViolation::Cycle { members });
            // Reach-based checks below would be ill-defined on a cyclic
            // graph — the cycle is enough to refuse.
            return Err(out);
        }

        // 4. Dependency-output coverage for every read.
        for task in &self.tasks {
            let available = self.transitive_outputs(task);
            for key in &task.inputs {
                if !available.iter().any(|k| k == key) {
                    out.push(DagViolation::UnsatisfiedRead {
                        task: task.id.clone(),
                        key: key.clone(),
                    });
                }
            }
        }

        // 5. Unordered write-write conflicts.
        for (i, task) in self.tasks.iter().enumerate() {
            for key in &task.outputs {
                let writers: Vec<&Task> = self
                    .tasks
                    .iter()
                    .filter(|t| t.outputs.iter().any(|o| o == key))
                    .collect();
                let conflicting: Vec<String> = writers
                    .iter()
                    .filter(|other| {
                        other.id != task.id
                            && !self.reaches(&other.id, &task.id)
                            && !self.reaches(&task.id, &other.id)
                    })
                    .map(|t| t.id.clone())
                    .collect();
                if !conflicting.is_empty() {
                    let mut tasks = vec![task.id.clone()];
                    tasks.extend(conflicting);
                    tasks.sort();
                    tasks.dedup();
                    // Report once per key (from the earliest writer).
                    let earliest = self
                        .tasks
                        .iter()
                        .position(|t| t.id == tasks[0])
                        .unwrap_or(0);
                    if earliest == i {
                        out.push(DagViolation::UnorderedWriteConflict {
                            key: key.clone(),
                            tasks,
                        });
                    }
                }
            }
        }

        if out.is_empty() { Ok(()) } else { Err(out) }
    }

    fn index_of(&self, id: &str) -> Option<usize> {
        self.tasks.iter().position(|t| t.id == id)
    }

    /// True when `from` can reach `to` through dependency edges
    /// (self-reach is false — cycles are refused before this runs).
    fn reaches(&self, from: &str, to: &str) -> bool {
        let mut stack = vec![from.to_string()];
        let mut seen: Vec<String> = Vec::new();
        while let Some(id) = stack.pop() {
            if seen.contains(&id) {
                continue;
            }
            seen.push(id.clone());
            if let Some(task) = self.tasks.iter().find(|t| t.id == id) {
                for dep in &task.depends_on {
                    if *dep == to {
                        return true;
                    }
                    stack.push(dep.clone());
                }
            }
        }
        false
    }

    /// All keys available to `task`: initial inputs + outputs of every
    /// transitive dependency.
    fn transitive_outputs(&self, task: &Task) -> Vec<String> {
        let mut available = self.initial_inputs.clone();
        let mut stack: Vec<String> = task.depends_on.clone();
        let mut seen: Vec<String> = Vec::new();
        while let Some(id) = stack.pop() {
            if seen.contains(&id) {
                continue;
            }
            seen.push(id.clone());
            if let Some(dep) = self.tasks.iter().find(|t| t.id == id) {
                available.extend(dep.outputs.iter().cloned());
                stack.extend(dep.depends_on.iter().cloned());
            }
        }
        available
    }

    /// Kahn's algorithm; returns sorted leftover ids when a cycle exists.
    fn cycle_members(&self) -> Vec<String> {
        let mut indegree: Vec<usize> = self
            .tasks
            .iter()
            .map(|t| {
                t.depends_on
                    .iter()
                    .filter(|d| self.index_of(d).is_some())
                    .count()
            })
            .collect();
        let mut queue: Vec<usize> = (0..self.tasks.len())
            .filter(|i| indegree[*i] == 0)
            .collect();
        let mut done = vec![false; self.tasks.len()];
        while let Some(i) = queue.pop() {
            done[i] = true;
            for (j, t) in self.tasks.iter().enumerate() {
                if !done[j] && t.depends_on.iter().any(|d| d == &self.tasks[i].id) {
                    indegree[j] -= 1;
                    if indegree[j] == 0 {
                        queue.push(j);
                    }
                }
            }
        }
        let mut leftover: Vec<String> = self
            .tasks
            .iter()
            .enumerate()
            .filter(|(i, _)| !done[*i])
            .map(|(_, t)| t.id.clone())
            .collect();
        leftover.sort();
        leftover
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str) -> Task {
        Task {
            id: id.to_string(),
            priority: PriorityClass::Normal,
            depends_on: vec![],
            inputs: vec![],
            outputs: vec![format!("out-{id}")],
            retry: RetryPolicy { max_attempts: 1 },
            timeout_ms: 1,
            cost: Money {
                currency: "USD".to_string(),
                minor_units: 0,
            },
            trivial: false,
            retryable: true,
            resource_units: 0,
            memory_mb: 0,
            exclusive: false,
        }
    }

    #[test]
    fn reachability_follows_dependency_edges() {
        let mut dag = TaskDag::new(vec![]);
        let mut b = task("B");
        b.depends_on = vec!["A".to_string()];
        let mut c = task("C");
        c.depends_on = vec!["B".to_string()];
        for t in [task("A"), b, c] {
            dag.add(t).unwrap();
        }
        assert!(dag.reaches("C", "A"), "transitive");
        assert!(!dag.reaches("A", "C"), "no back edges");
        assert!(!dag.reaches("A", "A"), "self is not a reach");
    }

    #[test]
    fn kahn_reports_the_leftover_as_cycle_members() {
        let mut dag = TaskDag::new(vec![]);
        let mut a = task("A");
        a.depends_on = vec!["B".to_string()];
        let mut b = task("B");
        b.depends_on = vec!["A".to_string()];
        let mut ok = task("OK");
        ok.depends_on = vec![];
        for t in [a, b, ok] {
            dag.add(t).unwrap();
        }
        assert_eq!(dag.cycle_members(), vec!["A".to_string(), "B".to_string()]);
    }
}
