# 02: Preregistration gate

**Status:** done

**What to build:** `preregister`: refuses n=None, unqualified method,
unresolved oracle, missing calculation reference / alpha allocation /
reference distribution / power-precision target / sensitivity assumptions -
each a named `PreregError`; returns a frozen `Preregistration` record.

**Acceptance:**
- [x] All named refusals (spec AC 4)
- [x] Happy path freezes the manifest record
- [x] Twin-run byte-identical (spec AC 6)

## Comments
