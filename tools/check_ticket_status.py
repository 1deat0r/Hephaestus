#!/usr/bin/env python3
"""Ticket status gates (ADR-028): stale labels are a gate problem.

Stale-open labels (work done, Status still open) are the dangerous
direction. This tool makes them detectable and therefore stoppable:

  --format (fast; part of `make ci`):
    every issue file carries a valid Status;
    every NOT-done ticket declares a `Verify:` command.

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


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def status_of(text: str):
    m = STATUS_RE.search(text)
    return m.group(1) if m else None


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
    if problems:
        return 1
    print(f"ticket-status: ok ({len(ISSUES)} issue files, statuses valid, open tickets declare Verify)")
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
