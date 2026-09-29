# Architecture decisions

ADR-001 — Broad goals are the product input. Autonomous discovery is not optional. Rejected alternative: ask the user to supply a high-quality hypothesis.

ADR-002 — Software-first, reversible, sandboxed experiments form the first validated scope. Physical domains require separate packs and authorization. Rejected alternative: universal lab autonomy on day one.

ADR-003 — Provider-neutral Rust authority with isolated Python workers. Rejected alternative: provider-specific message types or one large conversational loop as the operating system.

ADR-004 — Evidence ledger is authoritative; graphs and search indices are rebuildable views. Rejected alternative: a giant initial world graph as a launch prerequisite.

ADR-005 — Jev is optional advisory inference. Deterministic code governs policy, exact arithmetic, state transitions, and promotion preconditions.

ADR-006 — Hypothesis claims and engineering targets are separate. An inconclusive experiment is valid output. Rejected alternative: binary success/failure for every research question.

ADR-007 — Micro/nano units compile to typed bounded DAG operations. Rejected alternative: mandatory nested LLMs or native compilation per nano-task.

ADR-008 — Protected confirmation and independently controlled evaluators are release requirements. Rejected alternative: self-scored science.

ADR-009 — Scope-limited novelty reports replace global novelty scores. External disclosure is separately authorized.

ADR-010 — Self-improvement is versioned champion/challenger evaluation under fixed authority. Rejected alternative: agents may rewrite their own budget or gate.

ADR-011 — Local embedded state and isolated workers precede distributed backends. Tachyon integration remains optional until actual compatibility is verified.

ADR-012 — A useful negative or inconclusive dossier can complete a bounded mission. No artificial pressure to invent a positive conclusion.

## Version 1.1 decisions · 30 September 2026

These decisions implement the user's authorization to apply the dated review. They refine the baseline; they do not approve external execution or certify runtime behavior.

ADR-013 — Protected trust is external to model/candidate records. Grants, methods and execution/analysis/reproduction/promotion receipts bind immutable identity and authenticated content. Assessments additionally bind the exact subject payload; no field can inherit approval after changing. Default reference trust is empty. Impact: schema 1.1 adds five record types and required nullable draft references. Archived 1.0 records are retained; future migrations need explicit receipts. Rejected alternative: trusting well-formed hashes or qualification enums.

ADR-014 — Scientific/engineering qualification yields `validated_solution`, including prominently labeled useful rediscoveries. `validated_candidate` adds independently controlled assessment of scoped claim differences and a verified claim chart. Impact: known prior art does not change scientific conclusions; neither label implies global novelty. Rejected alternative: excluding useful known solutions from validation or treating no-hit search as invention.

ADR-015 — Initial confirmation uses a fixed, registered Bonferroni family with protected membership/selection/access history and sealed feedback until campaign closure. Renaming a family or mission cannot reset partition exposure. Impact: adaptive/sequential and multi-hypothesis reference promotion are unavailable until separately qualified; fresh confirmation remains supported. Rejected alternative: freeform method strings and unlimited sealed-benchmark feedback.

ADR-016 — The first qualified computational domain is conservative Python import-closure context assembly. A protected independent oracle, tuned exact-caching baseline, repository-level inference and complete failure/cost accounting are required. The 20% latency, 10% memory and 1% controlled false-promotion targets are provisional design decisions, not forecasts. Sample size and method qualification must be established before confirmation. Impact: claims are bounded to this operational context definition; broader semantic relevance and physical science need new qualification.

ADR-017 — Contract readiness and runtime acceptance have separate milestone fields and an explicit cumulative release manifest. Exact obligation anchors, reciprocal mappings and generated-document drift are checked. Impact: R-005 and R-080 end-to-end runtime acceptance moves to M3, new-domain activation R-006 to M6, and early scientific behaviors to their implementing milestones. Rejected alternative: claiming full workflow execution at M0.

## Version 1.2 decision · 30 September 2026

ADR-018 — Mandatory autonomous self-improvement, version 1.2. The user's explicit requirement makes the core champion/challenger, persisted-learning, permitted deployment and rollback loop an M3 release gate. R-070–R-072 move from M6 to M3; R-115–R-119 define the required autonomous demonstration. Suggestions alone are insufficient. Standing scoped authorization permits routine reversible configuration changes; code deployment requires its own capability and checks. Protected budgets, permissions, scientific methods and evaluators remain outside the loop. M6 retains only additional domains and advanced optional learning. Schema 1.2 adds the improvement-candidate record; earlier schemas/fixtures/evidence are archived without inherited approval.
