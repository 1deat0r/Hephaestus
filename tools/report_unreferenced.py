#!/usr/bin/env python3
"""Unreferenced-file report (ADR-024 — advisory only, never a gate).

Lists tracked non-markdown files whose basename is not mentioned by any
tracked markdown document. Path-based "referenced nowhere" checks false-
positive on convention- and code-referenced files, so this report is
informational: it always exits 0 (Chair ruling, ADR-024 tier table).
"""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=True
    ).stdout


def main() -> int:
    tracked = [p for p in git("ls-files").splitlines() if p]
    md_files = [p for p in tracked if p.endswith(".md")]
    corpus = "".join((ROOT / p).read_text(errors="replace") for p in md_files)
    candidates = [
        p
        for p in tracked
        if not p.endswith(".md") and Path(p).name not in corpus
    ]
    if candidates:
        print(
            f"report-unreferenced: {len(candidates)} tracked file(s) not named "
            "by any markdown document (advisory — never fails):"
        )
        for path in candidates:
            print(f"  {path}")
    else:
        print("report-unreferenced: every tracked file is named by some markdown document")
    return 0


if __name__ == "__main__":
    sys.exit(main())
