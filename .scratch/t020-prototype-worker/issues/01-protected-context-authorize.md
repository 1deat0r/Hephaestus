# 01: Protected context, PrototypeChange, authorize, receipts

**Status:** done

**What to build:** `prototype` module: `ProtectedContext` (evaluator +
baseline digests, verified artifact registry), `PrototypeChange` (target,
new digest, claims, receipts), `authorize` (rejects protected targets and
unverified receipts; accepts verified candidate changes), rejection
records (R-024 pattern: retained reason).

**Acceptance:**
- [x] Verified candidate change accepted (spec AC 1)
- [x] Evaluator/baseline targets rejected by name (spec AC 2)
- [x] Unverified receipt rejected (spec AC 3)

## Comments
