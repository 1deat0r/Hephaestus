#!/usr/bin/env python3
"""Ticket status gates (ADR-028): stale labels are a gate problem.

Stale-open labels (work done, Status still open) are the dangerous
direction. This tool makes them detectable and therefore stoppable:

  --format (fast; part of `make ci`):
    every issue file carries a valid Status;
    every NOT-done ticket declares a `Verify:` command and a
    `Covers:` list of spec acceptance-criteria numbers;
    four-level nesting holds for OPEN tickets (skill rule 18):
      TASK -> small tasks (cap 8) -> micro tasks (cap 6 per small)
      -> nano bullets (cap 4 per micro);
    every small task carries its own `**Status:**`, its own
    `**Verify:**`, and either `**Micro-tasks:**` or the marker
    `atomic`; every micro task carries a `**Verify:**` and either
    nano bullets or `atomic`;
    size caps hold for OPEN tickets: at most 16 scope IDs
    (AT-NNN/R-NNN) per ticket — landed history is grandfathered;
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
    Verify that RAN NOTHING (exit 0, zero tests) -> VACUOUS: it proves
    nothing in either direction, so it never counts as green and never
    reads as a landed label (ADR-031).
    A composite Verify (ADR-034) that exits 0 while one sub-suite prints
    zero counts, but whose output elsewhere proves executed tests, is
    NOT vacuous: `cargo test --workspace` always prints zero-test
    binary and doc sub-suites beside hundreds of real results. An
    empty marker alone only means some step found nothing to run.

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
SMALL_RE = re.compile(r"^\d+\. \[ \] \*\*S\d+\*\*", re.M)
MICRO_RE = re.compile(r"^\s{3,}\d+\. \[ \] \*\*M\d+\*\*", re.M)
NANO_RE = re.compile(r"^ {3,}- \[ \] ", re.M)
ATOM_RE = re.compile(r"^\s+atomic\s*$", re.M)
MAX_SMALL = 8
MAX_MICRO = 6
MAX_NANO = 4
# A Verify that executed nothing is not a green Verify: `cargo test --test
# t <filter>` exits 0 when no test matches, and unittest/pytest print an
# empty summary. Never read that as "the work landed" (ADR-031).
VACUOUS_RE = re.compile(
    r"running 0 tests|test result: ok\. 0 passed|Ran 0 tests|collected 0 items"
)
# Executed-test evidence anywhere in the same output: a count of at
# least one. `N ignored` and `N filtered out` are deliberately absent
# (ADR-031's defect printed `9 filtered out` beside `running 0 tests`).
EXECUTED_RE = re.compile(
    r"running [1-9]\d* tests?|test result: ok\. [1-9]\d* passed|"
    r"Ran [1-9]\d* tests?|collected [1-9]\d* items?"
)


def vacuous_output(output: str) -> bool:
    """True when exit-0 output proves that no test executed.

    An empty marker is not enough: a composite command prints empty
    sub-suite summaries next to real ones, and reading any one marker
    as proof of nothing ran falsely labels green composite work
    VACUOUS (ADR-034). Vacuity needs the marker AND the total absence
    of executed-test evidence in the whole output.
    """
    if VACUOUS_RE.search(output) is None:
        return False
    return EXECUTED_RE.search(output) is None


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


def check_nesting(path: Path, text: str) -> int:
    """Validate the four-level hierarchy of one OPEN ticket (rule 18)."""
    rel = path.relative_to(ROOT)
    if "**Small tasks:**" not in text:
        print(
            f"ticket-status: FAIL {rel}: open ticket without a "
            "`**Small tasks:** section — decompose the TASK (skill rule 18)"
        )
        return 1
    problems = 0
    smalls = SMALL_RE.findall(text)
    if not smalls:
        print(f"ticket-status: FAIL {rel}: `**Small tasks:**` but no `S<n>` items")
        return 1
    if len(smalls) > MAX_SMALL:
        print(
            f"ticket-status: FAIL {rel}: {len(smalls)} small tasks "
            f"(max {MAX_SMALL}) — split the TASK"
        )
        problems += 1
    if not MICRO_RE.search(text) and not ATOM_RE.search(text):
        print(
            f"ticket-status: FAIL {rel}: no micro tasks and no `atomic` "
            "marker — every small task nests micro tasks or `atomic`"
        )
        return problems + 1
    if MICRO_RE.search(text) and not NANO_RE.search(text) and not ATOM_RE.search(text):
        print(
            f"ticket-status: FAIL {rel}: micro tasks must nest nano "
            "bullets (`- [ ]`) or the marker `atomic`"
        )
        problems += 1
    # per-parent caps
    for i, block in enumerate(
        re.split(r"(?=^\d+\. \[ \] \*\*S\d+\*\*)", text, flags=re.M)[1:], 1
    ):
        micros = MICRO_RE.findall(block)
        if len(micros) > MAX_MICRO:
            print(
                f"ticket-status: FAIL {rel}: small S{i} has {len(micros)} "
                f"micro tasks (max {MAX_MICRO}) — split the small task"
            )
            problems += 1
        for j, mblock in enumerate(
            re.split(r"(?=^\s{3,}\d+\. \[ \] \*\*M\d+\*\*)", block, flags=re.M)[1:], 1
        ):
            nanos = NANO_RE.findall(mblock)
            if len(nanos) > MAX_NANO:
                print(
                    f"ticket-status: FAIL {rel}: small S{i} micro M{j} has "
                    f"{len(nanos)} nano steps (max {MAX_NANO}) — split the micro"
                )
                problems += 1
    # per-level Verify + Status lines (indented, so TASK lines stay first)
    for i, block in enumerate(
        re.split(r"(?=^\d+\. \[ \] \*\*S\d+\*\*)", text, flags=re.M)[1:], 1
    ):
        m = re.search(r"^ {3}\*\*Status:\*\*\s*([A-Za-z-]+)", block, re.M)
        if m is None:
            print(f"ticket-status: FAIL {rel}: small S{i} without its own `**Status:**`")
            problems += 1
        elif m.group(1) not in VALID:
            print(f"ticket-status: FAIL {rel}: small S{i} invalid status {m.group(1)!r}")
            problems += 1
        if not re.search(r"^ {3}\*\*Verify:\*\*\s*\S", block, re.M):
            print(f"ticket-status: FAIL {rel}: small S{i} without its own `**Verify:**`")
            problems += 1
        for j, mblock in enumerate(
            re.split(r"(?=^\s{3,}\d+\. \[ \] \*\*M\d+\*\*)", block, flags=re.M)[1:], 1
        ):
            if not re.search(r"^ {6}\*\*Verify:\*\*\s*\S", mblock, re.M):
                print(
                    f"ticket-status: FAIL {rel}: small S{i} micro M{j} "
                    "without its own `**Verify:**`"
                )
                problems += 1
    return problems


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
            problems += check_nesting(path, text)
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
        "open tickets declare Verify+Covers; four-level nesting holds "
        "(small<=8, micro<=6, nano<=4, per-level Status/Verify, atomic "
        "markers); scope-ID caps hold; open features cover every spec AC)"
    )
    return 0


def sweep_mode() -> int:
    stale = 0
    broken = 0
    vacuous = 0
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
            output = (proc.stdout or "") + (proc.stderr or "")
            ran_nothing = proc.returncode == 0 and vacuous_output(output)
            green = proc.returncode == 0 and not ran_nothing
        except subprocess.TimeoutExpired:
            green = False
            ran_nothing = False
        if ran_nothing:
            print(
                f"ticket-status: VACUOUS {rel}: Verify exited 0 with zero "
                f"tests run — it proves nothing: {cmd}"
            )
            vacuous += 1
        elif status == "done":
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
        f"{vacuous} vacuous, {open_ok} honestly open"
    )
    return 1 if (stale or broken or vacuous) else 0


def main() -> int:
    if not ISSUES:
        print("ticket-status: no issue files found")
        return 1
    if "--sweep" in sys.argv:
        return sweep_mode()
    return fmt_mode()


if __name__ == "__main__":
    sys.exit(main())
