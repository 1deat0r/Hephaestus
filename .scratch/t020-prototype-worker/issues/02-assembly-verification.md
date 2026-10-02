# 02: Assembly verification

**Status:** done

**What to build:** `Component` (name, version, interfaces, budget,
invariants, provides/requires), `assemble` (pairwise interface+version
match, named missing edges, resource aggregation vs mission limit,
total-system-cost accounting, global invariants, end-to-end guardrails;
failures name the component), `AssemblyReport`.

**Acceptance:**
- [x] Interface/version verified; missing edge named (spec AC 4)
- [x] Resource aggregation + cost-shift accounting (spec AC 5)
- [x] Invariants + guardrails name failures (spec AC 6)
- [x] Twin-run byte-identical (spec AC 7)

## Comments
