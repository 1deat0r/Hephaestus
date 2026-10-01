# 01: Mission record, profiles, and compile happy path

**What to build:** A broad authorized goal plus standing priorities and
resource profile compiles to a versioned Mission with the full §4 field set,
explicit assumptions, and one of the four autonomy profiles bound to
permitted tools/destinations — with no hypothesis anywhere in the record
(R-001, R-010).

**Blocked by:** None (can start immediately).

**Status:** done

- [x] `Mission` record carries every §4 field + version/supersedes/provenance/assumptions (red-first at `compile` seam)
- [x] Broad goal + priorities + resource profile → `Compiled::Mission`, no hypothesis field present
- [x] All four profiles map to distinct permitted tool/destination sets
- [x] Actuation or public-disclosure goal → `NeedsAuthorization`, never a Mission
- [x] Twin-run byte-identical output for representative intakes

## Comments
Done 2026-10-01: `mission/record.rs` + `mission/compiler.rs::compile`; 4/4 tests in `mission_compile.rs` green (red was unresolved-import). Money-Eq lesson: contract `Money` is PartialEq-only, mission types follow suit.
