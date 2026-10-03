#!/usr/bin/env python3
"""Scheduled-workflow state gate (ADR-030): a cron that silently stopped.

GitHub auto-disables scheduled workflows in a public repository after 60
days with no repository activity. A disabled scheduler is silent: no run,
no red job, no failure — only the first email. This check makes it loud
on the next push.

  python tools/check_scheduled_workflows.py    # online (push CI + local)

It reads every workflow file that declares a `- cron:` schedule, fetches
the live workflow list, and fails unless each one is `active`. A workflow
auto-disabled for inactivity reports as `disabled_inactivity` and names
the repair command. Network or API failure retries three times and then
FAILS: an unreachable check must never read as a passing one.

Repair: `gh workflow enable <file>` (or Actions -> workflow -> Enable
workflow), then push so the 60-day clock restarts.
"""
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_repo_about import repo_slug  # noqa: E402  (shared origin parsing)

ROOT = Path(__file__).resolve().parent.parent
WORKFLOWS = ROOT / ".github" / "workflows"
# A schedule block is the only place a cron list item appears.
CRON_RE = re.compile(r"^\s*-\s*cron:", re.M)
API_RETRIES = 3
FIX = "gh workflow enable <file>   (or Actions -> workflow -> Enable workflow), then push"


def scheduled_files() -> list[str]:
    if not WORKFLOWS.is_dir():
        return []
    found = []
    for path in sorted(WORKFLOWS.glob("*.yml")) + sorted(WORKFLOWS.glob("*.yaml")):
        if CRON_RE.search(path.read_text(encoding="utf-8", errors="replace")):
            found.append(path.relative_to(ROOT).as_posix())
    return found


def api_get(slug: str, token: str | None) -> dict:
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "hephaestus-scheduled-workflow-gate",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(
        f"https://api.github.com/repos/{slug}/actions/workflows?per_page=100",
        headers=headers,
    )
    with urllib.request.urlopen(request, timeout=20) as response:
        return json.loads(response.read().decode("utf-8"))


def fetch(slug: str) -> tuple[dict | None, str]:
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN") or None
    last = ""
    for attempt in range(1, API_RETRIES + 1):
        try:
            return api_get(slug, token), ""
        except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, OSError) as exc:
            last = f"{type(exc).__name__}: {exc}"
            if attempt < API_RETRIES:
                time.sleep(2 * attempt)
    return None, last


def main() -> int:
    slug = repo_slug()
    if not slug:
        print("scheduled: FAIL cannot determine the GitHub repository")
        return 1

    want = scheduled_files()
    if not want:
        print("scheduled: FAIL no workflow file declares a `- cron:` schedule")
        return 1

    live, error = fetch(slug)
    if live is None:
        # Fail-closed: unreachable must never read as passing.
        print(
            f"scheduled: FAIL cannot reach the GitHub API for {slug} after "
            f"{API_RETRIES} attempts ({error}) — an unknown workflow state is "
            "NOT a passing workflow state"
        )
        return 1

    by_path = {
        item.get("path"): item
        for item in live.get("workflows", [])
        if item.get("path")
    }

    problems = []
    for path in want:
        item = by_path.get(path)
        if item is None:
            problems.append(f"{path}: not present in the live workflow list")
            continue
        state = item.get("state", "")
        if state == "active":
            print(f"scheduled: ok {path} state=active")
            continue
        if state == "disabled_inactivity":
            problems.append(
                f"{path}: AUTO-DISABLED for 60 days without repository activity"
            )
        else:
            problems.append(f"{path}: state={state or 'unknown'}")

    if problems:
        print(f"scheduled: FAIL {len(problems)} scheduled workflow(s) not active")
        for line in problems:
            print(f"  {line}")
        print(f"  fix: {FIX}")
        return 1

    print(f"scheduled: all {len(want)} scheduled workflow(s) active on {slug}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
