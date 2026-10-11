# 04: Version bump — ship v0.2.0 (v1 unearned)

**Status:** done
**Verify:** grep -q '^version = "0.2.0"' Cargo.toml
**Covers:** 7, 8
**Blocked by:** 03 (needs the assembled packet)
**Requirements:** R-077

## Scope

Bump the workspace version to 0.2.0, write the changelog entry,
move release scopes off "Future gates", and set the scope label.
The label defaults to EXPERIMENTAL; QUALIFIED needs all five
qualification inputs present in the packet.

**Small tasks:**

1. [x] **S1** Bump version and record the release
   **Status:** done
   **Verify:** grep -q '^version = "0.2.0"' Cargo.toml
   Update version, changelog, and release scopes in one commit;
   keep the label honest to the packet.
   **Micro-tasks:**
   1. [x] **M1** Land the version commit
      **Verify:** grep -q '^version = "0.2.0"' Cargo.toml
      atomic

## Comments

Plan track for the v0.0.1 to v1 road. Blocked until 03 assembles
the packet.
