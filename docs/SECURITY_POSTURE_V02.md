# Security posture — v0.2.0 audit (2026-10-12 UTC)

Scope: local deny and containment suites plus dossier findings.
This audit records posture with fresh receipts. It does not
certify security. No live adversarial testing was run.

## Fresh suite receipts (this session)

Full outputs: `.scratch/security-posture/receipts/suites.log`.

- sandbox_deny: 17 passed, 0 failed — three consecutive runs,
  identical counts (see receipts/suites.log). One earlier CI
  failure in this suite passed alone and on retry; cause not
  established. Named here, not hidden.
- sealed_deny: 14 passed, 0 failed.
- grant_deny: 18 passed, 0 failed.
- evaluator_access: 4 passed, 0 failed.
- approval_deny: 8 passed, 0 failed.
- receipt_deny: 12 passed, 0 failed.
- containment_races: 5 passed, 0 failed.
- memory_quarantine: 4 passed, 0 failed.
- trust_deny: 6 passed, 0 failed.
- trust_propagation: 4 passed, 0 failed.

Total: 92 deny and containment checks green, zero failures.

## Open findings

- model-arms-absent (Minor, open, in scope): no consented
  provider contract exists. Blocks no gate; blocks model claims.
- compute currency (measurement gap, stated): local wall and CPU
  metered; provider currency absent by construction.

## Untested areas (explicit)

- Live adversarial behavior (prompt injection beyond fixtures,
  real egress, privilege escalation).
- Cross-machine reproduction of evidence artifacts.
- Confirmatory campaign security gates (no campaign flown).
- Dependency and supply-chain audit beyond the lockfile.

## Resolved this cycle

- compute-wall-time-metered (Minor, resolved): wall and CPU
  time metered per run. Evidence: metered-cost ticket 02
  (receipt `.scratch/metered-cost/receipts/02-green.md`);
  `docs/RELEASE_DOSSIER_V1.md` lists the finding as resolved
  with wall-time and cpu-time quantities.

## Disposition

Deny posture holds for the fixture scope. Nothing here
supports a production security claim. The next audit re-runs
these suites and extends into the untested areas above.
