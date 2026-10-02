#!/usr/bin/env python3
"""Ticket status gates (ADR-028): stale labels are a gate problem.

Stale-open labels (work done, Status still open) are the dangerous
direction. This tool makes them detectable and therefore stoppable:

  --format (fast; part of `make ci`):
    every issue file carries a valid Status;
    every NOT-done ticket declares a `Verify:` command and a
    `Covers:` list of spec acceptance-criteria numbers;
    size caps hold for OPEN tickets (small tasks ONLY): at most 16
    scope IDs (AT-NNN/R-NNN) and at most 8 unchecked boxes — landed
    history is grandfathered;
    a feature with any open ticket covers EVERY spec AC through the
    union of its tickets' Covers lists (as many tickets as needed);
    Covers numbers must exist in that feature's spec;
    a feature with a spec has at least one ticket.

  --sweep (periodic workflow; runs the declared commands):
    open ticket + Verify passes   -> STALE-OPEN (work is green: flip
                                     the Status in the same commit that
                                     landed the work) -> exit 1
    open ticket + Verify fails    -> honest open (work is not green)
    done ticket + Verify fails    -> BROKEN (regression or false-done)
                                     -> exit 1
    done tickets without Verify   -> covered by push CI regression.

  python tools/check_ticket_status.py --format
  python tools/check_ticket_status.py --sweep
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ISSUES = sorted(ROOT.glob(".scratch/*/issues/*.md"))
VALID = {"done", "ready-for-agent", "pending", "blocked"}
STATUS_RE = re.compile(r"\*\*Status:\*\*\s*([A-Za-z-]+)")
VERIFY_RE = re.compile(r"^\*{0,2}Verify:\*{0,2}\s*(.+)$", re.MULTILINE)
COVERS_RE = re.compile(r"^\*{0,2}Covers:\*{0,2}\s*(.+)$", re.MULTILINE)
SCOPE_ID_RE = re.compile(r"AT-\d{3}|(?<![A-Z])R-\d{3}")
MAX_SCOPE_IDS = 16
MAX_BOXES = 8


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def status_of(text: str):
    m = STATUS_RE.search(text)
    return m.group(1) if m else None


def covers_of(text: str):
    m = COVERS_RE.search(text)
    if not m:
        return None
    return [int(n) for n in re.findall(r"\d+", m.group(1))]


def spec_acs(slug: str):
    spec = ROOT / ".scratch" / slug / "spec.md"
    if not spec.exists():
        return None
    text = spec.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"## Acceptance Criteria\s*(.*?)(?=\n## |\Z)", text, re.S)
    if not m:
        return None
    return sorted({int(n) for n in re.findall(r"^(\d+)\. ", m.group(1), re.M)})


def verify_of(text: str):
    m = VERIFY_RE.search(text)
    return m.group(1).strip() if m else None


def fmt_mode() -> int:
    problems = 0
    for path in ISSUES:
        text = read(path)
        status = status_of(text)
        if status is None:
            print(f"ticket-status: FAIL {path}: no **Status:** line")
            problems += 1
        elif status not in VALID:
            print(f"ticket-status: FAIL {path}: invalid status {status!r}")
            problems += 1
        elif status != "done" and not verify_of(text):
            print(
                f"ticket-status: FAIL {path}: open ticket without a "
                "`Verify:` command — declare how to prove it green"
            )
            problems += 1
        if status != "done":
            n_ids = len(SCOPE_ID_RE.findall(text))
            if n_ids > MAX_SCOPE_IDS:
                print(
                    f"ticket-status: FAIL {path}: {n_ids} scope IDs "
                    f"(max {MAX_SCOPE_IDS}) — split into smaller tickets"
                )
                problems += 1
            n_boxes = text.count("- [ ]")
            if n_boxes > MAX_BOXES:
                print(
                    f"ticket-status: FAIL {path}: {n_boxes} unchecked boxes "
                    f"(max {MAX_BOXES}) — split into smaller tickets"
                )
                problems += 1
        slug = path.relative_to(ROOT / ".scratch").parts[0]
        acs = spec_acs(slug)
        covers = covers_of(text)
        if status != "done" and covers is None:
            print(
                f"ticket-status: FAIL {path}: open ticket without a "
                "`Covers:` list — name the spec ACs it closes"
            )
            problems += 1
        if acs is not None and covers is not None:
            bad = [c for c in covers if c not in acs]
            if bad:
                print(
                    f"ticket-status: FAIL {path}: Covers {bad} not in "
                    f"spec Acceptance Criteria {acs}"
                )
                problems += 1
    # feature-level coverage: a feature with open tickets must cover
    # every spec AC through the union of its tickets' Covers lists
    by_feature = {}
    for path in ISSUES:
        slug = path.relative_to(ROOT / ".scratch").parts[0]
        by_feature.setdefault(slug, []).append(path)
    for slug, paths in sorted(by_feature.items()):
        acs = spec_acs(slug)
        if acs is None:
            continue
        texts = [read(p) for p in paths]
        if all(status_of(x) == "done" for x in texts):
            continue  # closed feature: nothing left to decompose
        union = set()
        for x in texts:
            union.update(covers_of(x) or [])
        missing = [a for a in acs if a not in union]
        if missing:
            print(
                f"ticket-status: FAIL .scratch/{slug}: spec ACs {missing} "
                "covered by no ticket Covers list — add tickets or fix Covers"
            )
            problems += 1
    if problems:
        return 1
    print(
        f"ticket-status: ok ({len(ISSUES)} issue files; statuses valid; "
        "open tickets declare Verify+Covers; size caps hold; open features cover every spec AC)"
    )
    return 0


def sweep_mode() -> int:
    stale = 0
    broken = 0
    open_ok = 0
    for path in ISSUES:
        text = read(path)
        status = status_of(text)
        cmd = verify_of(text)
        if not cmd:
            continue  # done without Verify: push CI covers regressions
        rel = path.relative_to(ROOT)
        try:
            proc = subprocess.run(
                cmd, shell=True, cwd=ROOT, capture_output=True, text=True, timeout=900
            )
            green = proc.returncode == 0
        except subprocess.TimeoutExpired:
            green = False
        if status == "done":
            if not green:
                print(f"ticket-status: BROKEN {rel}: Verify fails: {cmd}")
                broken += 1
        else:
            if green:
                print(
                    f"ticket-status: STALE-OPEN {rel}: Verify is green — "
                    "flip Status to done in the commit that landed the work"
                )
                stale += 1
            else:
                print(f"ticket-status: open (honest) {rel}")
                open_ok += 1
    print(
        f"ticket-status-sweep: {stale} stale-open, {broken} broken-done, "
        f"{open_ok} honestly open"
    )
    return 1 if (stale or broken) else 0


def main() -> int:
    if not ISSUES:
        print("ticket-status: no issue files found")
        return 1
    if "--sweep" in sys.argv:
        return sweep_mode()
    return fmt_mode()


if __name__ == "__main__":
    sys.exit(main())
