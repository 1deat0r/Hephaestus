#!/usr/bin/env python3
"""AT citation report (advisory — ADR-024 tier; ADR-027 lineage).

For every acceptance test id (AT-xxx) in requirements.json that is
cited NOWHERE in implementation code, print the triage worklist:
the requirement it belongs to, the OBLIGATIONS negative case and
required outcome (what a mirroring test must show), and the places
where the parent R-id IS cited (candidate mirror sites to audit).

Advisory by design: exit 0 always — triage lands as honest CODE CITES
(at a verified mirror) or as RECORDED GOAL CANDIDATES, never as a
silent allowlist (see t066 spec). Graduation into `make ci` is a
future decision, not this report's job.

  python tools/report_at_citations.py
  make at-coverage
"""
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_requirement_citations import code_files  # noqa: E402  (shared scope)

ROOT = Path(__file__).resolve().parent.parent
OBLIGATIONS = ROOT / "docs" / "OBLIGATIONS.md"


def at_cited(files) -> set:
    ids = set()
    for path in files:
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        ids.update(re.findall(r"AT-\d{3}", text))
    return ids


def r_sites(r_id: str, files, limit: int = 3) -> list:
    sites = []
    for path in files:
        try:
            lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        except OSError:
            continue
        for n, line in enumerate(lines, 1):
            if r_id in line:
                sites.append(f"{path.relative_to(ROOT)}:{n}")
                if len(sites) >= limit:
                    return sites
    return sites


def obligations_text() -> dict:
    """R-id -> (statement, negative case, required outcome)."""
    out = {}
    if not OBLIGATIONS.exists():
        return out
    text = OBLIGATIONS.read_text(encoding="utf-8")
    for block in re.split(r"\n(?=## R-\d{3}\n)", text):
        m = re.match(r"## (R-\d{3})\n+(.*?)(?=\n## |\Z)", block, re.S)
        if not m:
            continue
        rid, body = m.group(1), m.group(2)
        statement = body.strip().splitlines()[0].strip() if body.strip() else ""
        neg = re.search(r"\*\*Negative case:\*\*\s*(.+)", body)
        req = re.search(r"\*\*Required outcome:\*\*\s*(.+)", body)
        out[rid] = (
            statement,
            neg.group(1).strip() if neg else "(none recorded)",
            req.group(1).strip() if req else "(none recorded)",
        )
    return out


def batch_mode(path: str, min_disposed=None) -> int:
    """Check one triage-batch ticket: every listed AT must be cited in
    code scope OR listed in the ticket's own `Candidate goals:` section
    (dispositioned, never silently skipped).

    `min_disposed` (optional int) turns the check into a count gate:
    exit 0 once at least that many of the batch's ATs are disposed.
    Small tasks use it for subset Verify lines without naming IDs."""
    text = Path(path).read_text(encoding="utf-8", errors="replace")
    ats_block = re.search(r"\*\*ATs:\*\*(.*?)(\n\n)", text, re.S)
    ats = re.findall(r"AT-\d{3}", ats_block.group(1)) if ats_block else []
    cands_block = re.search(r"Candidate goals:\s*(.*?)(?=\n## |\Z)", text, re.S)
    cands = set(re.findall(r"AT-\d{3}", cands_block.group(1))) if cands_block else set()
    cited = at_cited(code_files())
    untriaged = [a for a in ats if a not in cited and a not in cands]
    print(
        f"at-batch {path}: {len(ats)} ATs, {len(cands)} candidate(s) listed, "
        f"{len(ats) - len(untriaged) - len([a for a in ats if a in cands and a not in cited])} cited"
    )
    for a in untriaged:
        print(f"  UNTRIAGED {a} — cite at a verified mirror or list it under Candidate goals")
    if not ats:
        print("  FAIL: no **ATs:** list found")
        return 1
    if min_disposed is not None:
        disposed = len(ats) - len(untriaged)
        if disposed < min_disposed:
            print(f"  FAIL: {disposed} disposed, need {min_disposed}")
            return 1
        return 0
    return 1 if untriaged else 0


def main() -> int:
    reqs = json.loads((ROOT / "requirements.json").read_text())["requirements"]
    files = code_files()
    cited = at_cited(files)
    obligations = obligations_text()

    gaps = []
    for r in reqs:
        for at in r.get("acceptance_tests", []):
            if at not in cited:
                gaps.append((at, r["id"], r.get("statement", "")))
    gaps.sort()

    print(f"at-coverage: {len(gaps)} uncited AT ids in code scope (advisory report)\n")
    for at, rid, statement in gaps:
        neg, req_out = "(see OBLIGATIONS)", "(see OBLIGATIONS)"
        if rid in obligations:
            _, neg, req_out = obligations[rid]
        sites = r_sites(rid, files)
        hint = ", ".join(sites) if sites else "R cited nowhere in code scope either (see R-gate allowlist reasons)"
        print(f"{at} [{rid}] {statement}")
        print(f"    negative:   {neg}")
        print(f"    required:   {req_out}")
        print(f"    R cited at: {hint}")
        print()
    print(
        "triage rule: cite only where a test mirrors the negative case "
        "(read both sides); otherwise record a goal candidate — no allowlist."
    )
    return 0


if __name__ == "__main__":
    argv = sys.argv[1:]
    min_disposed = None
    if "--min-disposed" in argv:
        j = argv.index("--min-disposed")
        if j + 1 >= len(argv):
            print("usage: --min-disposed <n>")
            sys.exit(2)
        min_disposed = int(argv[j + 1])
    if "--batch" in argv:
        idx = argv.index("--batch")
        if idx + 1 >= len(argv):
            print("usage: --batch <ticket-file>")
            sys.exit(2)
        sys.exit(batch_mode(argv[idx + 1], min_disposed))
    sys.exit(main())
