# 01: Guard the dossier renderer against provisional leaks

**Status:** done
**Verify:** cargo test -p hephaestus --test release_packet provisional_never_renders
**Covers:** 1, 2, 3
**Blocked by:** None (can start immediately)

## Scope

Add the render-level guard test, inventory touchpoints, and
update gaps row 9.

**Small tasks:**

1. [x] **S1** Add guard and update row
   **Status:** done
   **Verify:** cargo test -p hephaestus --test release_packet provisional_never_renders
   Guard green; touchpoints inventoried; row updated.
   **Micro-tasks:**
   1. [x] **M1** Guard and record
      **Verify:** cargo test -p hephaestus --test release_packet provisional_never_renders
      atomic

## Comments

Touchpoints: release/mod.rs gate loop, achieved_benchmarks
filter, v1 assembly, dossier render. No other readers exist.
