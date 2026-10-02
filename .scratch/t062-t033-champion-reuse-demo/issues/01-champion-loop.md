# 01: Champion resolution + proposal edge + the R-119 demonstration

**What to build:** `propose_from_observations` (bounded, refuses
unseeded/self-budgeted proposals) and `active_champion` (deployed →
challenger, rolled_back → incumbent, empty → default) in selfimprove,
plus `tests/champion_reuse.rs` demonstrating the full R-119 loop with
receipts: non-seeded opportunity → proposal → qualified promotion →
persisted champion → later mission uses it (digest-verified) → restart
preserves state → injected regression → rollback → incumbent reused,
and four disqualified challengers refused by name.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `propose_from_observations` refuses unseeded + self-budgeting
      (typed); accepts observation-backed candidate
- [x] `active_champion`: deployed / rolled-back / empty cases
- [x] champion_reuse green: A(default) → B(championed, wider sweep
      receipt) → restart-identical → rollback → C/D(default);
      4 disqualified challengers each rejected by name
- [x] Existing suites green; allowlist +2 + reseal; gates green

## Comments

Grill: `.scratch/t062-t033-champion-reuse-demo/grill.md`.
Spec: `.scratch/t062-t033-champion-reuse-demo/spec.md`.
