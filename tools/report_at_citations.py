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
    sys.exit(main())
