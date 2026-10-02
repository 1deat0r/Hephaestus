#!/usr/bin/env python3
"""Requirement citation coverage (ADR-027): every R-id in
requirements.json must be cited somewhere in IMPLEMENTATION code, or
carry a reasoned entry in tools/requirement_citations.txt.

Scope discipline (the whole point — see spec): scan CODE only
(crates/**.rs, python/**.py, tests/**, tools/*.py, validation/*.py,
.githooks/*). Requirements/validation DATA is never scanned — the
requirements would cite themselves (tautology), and skipping code
dirs manufactured phantom gaps in ad-hoc scans.

Fail-closed both directions:
  - an uncited id absent from the allowlist  -> FAIL (triage: cite or
    add an entry with a reason);
  - an allowlisted id that IS cited           -> FAIL (stale entry,
    same discipline as manifest-check's stale allowlist detection).

  python tools/check_requirement_citations.py
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALLOWLIST = ROOT / "tools" / "requirement_citations.txt"
REQUIREMENTS = ROOT / "requirements.json"

CODE_GLOBS = (
    "crates/**/*.rs",
    "python/**/*.py",
    "tests/**/*.py",
    "tests/**/*.rs",
    "tools/*.py",
    "validation/*.py",
    ".githooks/*",
)


def code_files():
    seen = []
    for pattern in CODE_GLOBS:
        for path in sorted(ROOT.glob(pattern)):
            if path.is_file() and path.name != Path(__file__).name:
                seen.append(path)
    return seen


def cited_ids(files):
    ids = set()
    for path in files:
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        ids.update(re.findall(r"R-\d{3}", text))
    return ids


def allowlisted_ids():
    entries = {}
    for line in ALLOWLIST.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        match = re.match(r"(R-\d{3})\s+#\s+(.+)", line)
        if not match:
            print(f"requirement-citations: malformed line: {line!r}")
            sys.exit(2)
        entries[match.group(1)] = match.group(2)
    return entries


def main() -> int:
    reqs = json.loads(REQUIREMENTS.read_text())["requirements"]
    all_ids = {r["id"] for r in reqs}
    cited = cited_ids(code_files()) & all_ids
    allow = allowlisted_ids()

    uncited = sorted(all_ids - cited)
    untriaged = [i for i in uncited if i not in allow]
    stale = sorted(i for i in allow if i in cited)
    unknown = sorted(i for i in allow if i not in all_ids)

    problems = False
    for rid in untriaged:
        print(
            f"requirement-citations: FAIL {rid} is cited nowhere in code "
            "scope — add the honest citation or an allowlist entry with a reason."
        )
        problems = True
    for rid in stale:
        print(
            f"requirement-citations: stale allowlist entry {rid} — now cited; "
            "remove it from tools/requirement_citations.txt."
        )
        problems = True
    for rid in unknown:
        print(f"requirement-citations: allowlist entry {rid} is not a requirements id.")
        problems = True
    if problems:
        return 1
    print(
        f"requirement-citations: ok ({len(all_ids)} requirements, "
        f"{len(cited)} cited in code scope, {len(allow)} reasoned allowlist entries)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
