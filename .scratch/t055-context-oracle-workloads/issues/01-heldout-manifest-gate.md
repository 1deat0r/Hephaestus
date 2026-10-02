# 01: Held-out manifest access gate + declared clustering unit

**What to build:** The R-084 gate pair in the pilot module: workers
requesting a confirmatory (held-out) repository's workload manifest
are denied by name while the protected evaluator reads it, and the
pilot plan declares a non-empty clustering unit that
`record_outcome` transports verbatim into the variance estimate — so
the analysis provably retains its declared unit instead of an
undeclared default.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] AT-084: Worker + confirmatory repo →
      `HeldOutManifestDenied`; ProtectedEvaluator → Ok; Worker +
      pilot repo → Ok; unknown repo named
- [x] Empty `clustering_unit` → `plan_pilot` refuses
      (`MissingClusteringUnit`)
- [x] `record_outcome` estimate carries the declared unit verbatim
- [x] Glossary rows (Held-out workload manifest, Clustering unit);
      allowlist +3 + reseal; gates green; tests cite R-084/AT-084

## Comments

Grill: `.scratch/t055-context-oracle-workloads/grill.md`.
Spec: `.scratch/t055-context-oracle-workloads/spec.md`.
