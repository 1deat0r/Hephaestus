# 01: Typed capability grants with mint, validate, revoke, and contract round-trip

**What to build:** Callers can mint an approved capability grant that binds one operation, its capability scope, destination, artifact identity, policy version, validity window, and approved cost; validate it, revoke it, and convert it losslessly to and from the authorization_grant contract — the standing local authority AT-099 needs, with no user prompt per use.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] `mint` produces an approved grant binding operation id, capabilities, destination, artifact sha256 (nullable), policy version, issued/expires, max cost, issuer id
- [ ] `validate` rejects a reversed or empty validity window and malformed binding fields at every public constructor (`mint`, `from_contract`) — no contract- or window-violating grant is buildable through the API (fields remain public for interop; the engine re-validates the window at evaluation time)
- [ ] `revoke` transitions state to revoked; revoked grants are distinguishable from approved at the type/JSON level
- [ ] `to_contract` → JSON → `from_contract` round-trips without loss, and the JSON passes the generated contract's own validation
- [ ] Zero new dependencies
