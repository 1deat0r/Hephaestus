# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`GLOSSARY.md`** at the repo root — **runtime-era file, editable with a
  decision row**: it is *not* covered by `MANIFEST.sha256` (verified
  2026-09-30; the earlier "MANIFEST-frozen" claim here was stale) and appears
  in `tools/runtime_allowlist.txt` as a runtime file outside the frozen spec
  envelope (ADR-024 L4). New runtime vocabulary may be appended, but the edit
  must be recorded as a decision row in the workflow's decision log first;
  spec-package wording inside existing entries still belongs to a versioned
  reseal.
- **ADR registers** (this repo deliberately has no `docs/adr/` — see ADR-019):
  - `docs/DECISIONS.md` — frozen register, ADR-001–018 (read-only, MANIFEST-covered).
  - `docs/RUNTIME_DECISIONS.md` — live register, ADR-019 onward (runtime-era
    decisions land here, one-paragraph style with rejected alternatives).
- If any of these files don't exist, proceed silently; `/domain-modeling` creates
  missing docs lazily.

## Use the glossary's vocabulary

When your output names a domain concept (issue title, refactor proposal,
hypothesis, test name), use the term as defined in `GLOSSARY.md`. Don't drift
to synonyms the glossary explicitly avoids. If the concept isn't there, either
you're inventing language the project doesn't use, or it's a real gap (note it
for `/domain-modeling` — but follow the editable-with-a-decision-row rule above).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than
silently overriding:

> _Contradicts ADR-00NN (...), but worth reopening because…_
