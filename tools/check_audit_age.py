#!/usr/bin/env python3
"""Audit-artifact cadence watchdog (ADR-024, T3 scheduled workflow).

Fails when validation/*/AUDIT.md is missing or its newest commit date is
older than the cadence window. Commit dates (not file mtimes) are used
because CI checkouts reset mtimes. Existence and freshness are gated;
content quality stays with the author-led audit process (R-086).
"""
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CADENCE_DAYS = 90


def newest_audit_epoch() -> int | None:
    listed = subprocess.run(
        ["git", "ls-files", "-z", "validation/*/AUDIT.md"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split("\0")
    newest = None
    for path in listed:
        if not path:
            continue
        epoch = int(
            subprocess.run(
                ["git", "log", "-1", "--format=%ct", "--", path],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=True,
            ).stdout.strip()
        )
        newest = epoch if newest is None else max(newest, epoch)
    return newest


def main() -> int:
    epoch = newest_audit_epoch()
    if epoch is None:
        print(
            "audit-age: FAIL — no validation/<date>/AUDIT.md exists. "
            "Hint: author-led audits are the L3 remainder (ADR-024); "
            "commit one under validation/<YYYY-MM-DD>/AUDIT.md.",
            file=sys.stderr,
        )
        return 1
    age_days = int((time.time() - epoch) // 86400)
    if age_days > CADENCE_DAYS:
        print(
            f"audit-age: FAIL — newest AUDIT.md is {age_days} days old "
            f"(cadence {CADENCE_DAYS} days). Hint: run a fresh audit.",
            file=sys.stderr,
        )
        return 1
    print(f"audit-age: ok (newest AUDIT.md {age_days} days old, cadence {CADENCE_DAYS})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
