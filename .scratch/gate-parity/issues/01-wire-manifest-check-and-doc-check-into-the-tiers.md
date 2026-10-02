# 01: Wire manifest-check into ci-fast and add the single-sourced doc-check target

**What to build:** Every commit runs the manifest-coverage classifier before
it lands (via the existing pre-commit → `make ci-fast` path), `make
doc-check` exists as the one definition of the rustdoc check that both a
human and the remote docs job invoke, and the tier change is recorded as
ADR-025 with the gate seal regenerated — so the exact failure that produced
run 3's two red pushes is impossible to reach by accident.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `ci-fast` includes `manifest-check`; the pre-commit hook is untouched
      (it already delegates to `ci-fast`)
- [x] `manifest-check` remains explicitly listed on the `ci` target
      (registry inventory preserved)
- [x] `doc-check` target runs `RUSTDOCFLAGS="-D warnings" cargo doc
      --workspace --no-deps`, is wired into **no** tier, and carries a
      comment citing the ADR-024 step-10 latency decision
- [x] The workflow's docs job runs `make doc-check` instead of the inline
      command; no other functional workflow change (a 2-line comment
      documenting the single-sourcing is allowed and was added — pass-2
      wording correction)
- [x] ADR-025 appended to `docs/RUNTIME_DECISIONS.md` (one paragraph,
      rejected alternatives, named superseded ADR-024 appendix rows,
      measured `ci-fast` overhead); ADR-024 text untouched
- [x] Gate seal regenerated (`gate_seal.py --write`) and `--check` passes
- [x] Evidence in the state LOG: `make ci-fast` green; **negative probe**
      (staged unlisted file makes `ci-fast` fail with the classifier hint,
      then green after removal); `make doc-check` rc=0; `make ci` EXIT 0;
      before/after `ci-fast` timings — report.md's allowlist entry is
      re-added when the rotation is next STAGED (landing time), not on
      mere rewrite: `git ls-files` reads the index, so gates stay green
      while the rotated file is untracked (pass-2 precision)
- [x] No requirement ID exists for this work — recorded as such (gate
      hygiene, ADR-024/025), never invented

## Comments

2026-09-30T09:20:40Z — Done. Evidence: ci-fast 0.950/0.928s -> 0.954s (~0.02s, in noise); negative probe EXIT=2 with the classifier hint then EXIT=0 after removal; make doc-check EXIT=0; make ci attempt 1 RED on a real rotation divergence (gate correct), attempt 2 EXIT=0; seal regenerated; ADR-025 appended with measured numbers. No requirement ID exists — gate hygiene (ADR-024/025), stated honestly.

## Comments (pass 2)

2026-09-30T09:28:54Z — AC4/AC7 wording corrected per pass-2 partials. All 8 ACs otherwise MET (reviewers re-verified seal hashes, ADR append-only, timings vs LOG). Pass 3 not spent: cycle-2 fixes are documentation-text only (no code/gate surface) — logged as a known limitation in the report rather than an unreviewed gap.

Closed 2026-10-02 — work landed earlier; verified green: manifest-check and md-links run inside make ci; green at 602 tests.
