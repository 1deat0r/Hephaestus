# v1 gaps register — v0.2.0 (2026-10-12 UTC)

One row per known gap. Entries close only with evidence or an
explicit owner call. Consent-flagged rows wait on the owner.

Consent values: yes (owner only), no (agent workable), partial
(agent part doable now; remainder needs the owner — split named
in the row).

| # | Gap | Owner | Consent? | Unblocked by | Gates claim | Source |
|---|-----|-------|----------|--------------|-------------|--------|
| 1 | Live missions unflown | owner | yes | owner consent + spend | pilot variance in the wild | dossier Q1 |
| 2 | Model arms unrun (finding model-arms-absent open, Minor) | owner | yes | provider keys + contracts | any model-vs-deterministic claim | dossier Q2, finding; posture finding |
| 3 | Metered currency absent | owner | yes | metered provider or budget source | cost per qualified outcome | dossier Q3 |
| 4 | Confirmatory campaign unflown; qualifying scope undecided | owner | yes | frozen campaign + spend + scope decision | QUALIFIED scope | dossier Q4 |
| 5 | Confirmatory campaign security gates unrun | owner | yes | campaign flown (gap 4) | security gates completeness | posture untested 3 |
| 6 | Cross-machine reproduction unproven | agent | partial | agent: CI arm now; owner: second machine | independent_reproduction input | posture untested 2 |
| 7 | Live adversarial testing unrun | owner | yes | test scope + target | production security posture | posture untested 1 |
| 8 | Supply-chain audit beyond lockfile | agent | no | audit pass over deps; no ticket exists yet | security gates completeness | posture untested 4 |
| 9 | Provisional targets unmeasured | agent | partial | agent: mark stays-provisional; owner: campaign measures | any speed/novelty claim | release packet ProvisionalTarget claims |
| 10 | Sandbox CI flake cause unknown | agent | no | repeated runs / runner data; no ticket exists yet | CI trust (named, not hidden) | posture audit flake note |

## Resolved (evidence linked)

- compute-wall-time-metered (Minor): metered-cost ticket 02,
  receipt `.scratch/metered-cost/receipts/02-green.md`; dossier
  lists it resolved with wall-time and cpu-time quantities.

## Rules

- No row closes silently. Evidence or owner call, recorded here.
- New gaps append with the next number. Numbers never reuse.
- Check: row numbers unique —
  `awk -F'|' '/^\| [0-9]+/ {print $2}' docs/V1_GAPS.md | sort -n -c -u`.
