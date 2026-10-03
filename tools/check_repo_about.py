#!/usr/bin/env python3
"""GitHub About gate (ADR-029): the live About cannot drift from the repo.

The About (description, website, topics) lives on GitHub, outside every
file gate, so it went stale with nothing to notice. ADR-029 moves it into
the repository as `.github/repo-about.json` and checks it from both ends:

  python tools/check_repo_about.py           # offline (in `make ci`)
  python tools/check_repo_about.py --live    # online (push CI + periodic)
  python tools/check_repo_about.py --sync    # push canonical -> GitHub, then stamp

Offline leg, fail-closed:
  - the canonical file parses and carries every required key;
  - the description is 1..350 characters (GitHub's limit) and starts with
    the project name from README.md's H1, so a renamed repository cannot
    keep an old About;
  - the description carries `v<Version>` from README.md's version line,
    so a spec bump cannot leave the About behind;
  - topics are 1..20 unique `[a-z0-9-]` tokens (GitHub's rules);
  - the homepage is empty or an https URL;
  - `verified` is a real date, not in the future, and no older than
    `max_age_days`, so an unconfirmed About expires instead of aging out
    of sight.

Online leg: GET the repository and compare the description and homepage
byte-for-byte and the topics as a set (GitHub returns them sorted).
Network or API failure retries three times and then FAILS: an unreachable
check must never read as a passing one.

`--sync` is local-only and needs owner auth (`gh`): it PATCHes the live
About from the canonical file, re-verifies, and stamps `verified`.
"""
import datetime as _dt
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CANONICAL = ROOT / ".github" / "repo-about.json"
README = ROOT / "README.md"

REQUIRED_KEYS = ("description", "homepage", "topics", "source", "verified", "max_age_days")
MAX_DESCRIPTION = 350  # GitHub's repository-description limit.
MAX_TOPICS = 20  # GitHub's repository-topic limit.
TOPIC_RE = re.compile(r"^[a-z0-9-]{1,50}$")
DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")
API_RETRIES = 3


def fail(message: str) -> int:
    print(f"about: FAIL {message}")
    return 1


def load_canonical() -> tuple[dict | None, list[str]]:
    if not CANONICAL.is_file():
        return None, [f"missing canonical About file: {CANONICAL.relative_to(ROOT)}"]
    try:
        data = json.loads(CANONICAL.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return None, [f"{CANONICAL.relative_to(ROOT)} does not parse: {exc}"]
    if not isinstance(data, dict):
        return None, [f"{CANONICAL.relative_to(ROOT)} must be a JSON object"]
    errors = [f"missing key: {key}" for key in REQUIRED_KEYS if key not in data]
    return (None, errors) if errors else (data, [])


def readme_facts() -> tuple[str | None, str | None, list[str]]:
    """Project name from the H1 and `Version x.y` from the version line."""
    if not README.is_file():
        return None, None, ["README.md missing"]
    text = README.read_text(encoding="utf-8", errors="replace")
    errors: list[str] = []
    name = None
    version = None
    first = text.splitlines()[0] if text.splitlines() else ""
    heading = re.match(r"^#\s+(.+)$", first)
    if not heading:
        errors.append("README.md has no H1 heading")
    else:
        name = heading.group(1).split(" — ")[0].strip().split()[0]
        if not name:
            errors.append("README.md H1 yields no project name")
    match = re.search(r"^Version\s+(\d+(?:\.\d+)+)", text, re.M)
    if not match:
        errors.append("README.md has no `Version x.y` line")
    else:
        version = match.group(1)
    return name, version, errors


def offline_check() -> int:
    data, errors = load_canonical()
    if data is None:
        for line in errors:
            print(f"about: FAIL {line}")
        return 1

    description = data.get("description")
    if not isinstance(description, str) or not description.strip():
        errors.append("description must be a non-empty string")
    elif len(description) > MAX_DESCRIPTION:
        errors.append(
            f"description is {len(description)} chars, GitHub allows {MAX_DESCRIPTION}"
        )

    name, version, readme_errors = readme_facts()
    errors.extend(readme_errors)
    if name and isinstance(description, str):
        if not description.startswith(name):
            errors.append(
                f"description must start with the README project name {name!r} "
                "(rename the About with the repository)"
            )
    if version and isinstance(description, str):
        if f"v{version}" not in description:
            errors.append(
                f"description must carry v{version} from README.md's version line "
                "(the About ages out when the spec version moves)"
            )

    topics = data.get("topics")
    if not isinstance(topics, list) or not topics:
        errors.append("topics must be a non-empty list")
    else:
        if len(topics) > MAX_TOPICS:
            errors.append(f"{len(topics)} topics, GitHub allows {MAX_TOPICS}")
        bad = [t for t in topics if not isinstance(t, str) or not TOPIC_RE.match(t)]
        if bad:
            errors.append(f"invalid topic token(s): {bad}")
        if len(set(topics)) != len(topics):
            errors.append("topics must be unique")

    homepage = data.get("homepage")
    if not isinstance(homepage, str):
        errors.append("homepage must be a string")
    elif homepage and not homepage.startswith("https://"):
        errors.append("homepage must be empty or an https:// URL")

    if not isinstance(data.get("source"), str) or data.get("source") != "README.md":
        errors.append("source must name README.md (the About is derived from it)")

    verified = data.get("verified")
    max_age = data.get("max_age_days")
    if not isinstance(verified, str) or not DATE_RE.match(verified):
        errors.append("verified must be a YYYY-MM-DD date")
    elif not isinstance(max_age, int) or max_age < 1:
        errors.append("max_age_days must be a positive integer")
    else:
        stamp = _dt.date.fromisoformat(verified)
        today = _dt.date.today()
        if stamp > today + _dt.timedelta(days=1):
            errors.append(f"verified {verified} is in the future")
        else:
            age = (today - stamp).days
            if age > max_age:
                errors.append(
                    f"verified {verified} is {age} days old (max {max_age}) — "
                    "run `make about-sync` to re-confirm the live About"
                )

    if errors:
        for line in errors:
            print(f"about: FAIL {line}")
        return 1
    print(
        f"about: ok ({len(description)} chars, {len(topics)} topics, "
        f"verified {verified}, README {name} v{version})"
    )
    return 0


def repo_slug() -> str:
    slug = os.environ.get("GITHUB_REPOSITORY", "")
    if slug.count("/") == 1:
        return slug
    try:
        url = subprocess.run(
            ["git", "remote", "get-url", "origin"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
    except subprocess.CalledProcessError:
        return ""
    match = re.search(r"github\.com[:/]([^/]+)/([^/\s]+?)(?:\.git)?$", url)
    return f"{match.group(1)}/{match.group(2)}" if match else ""


def api_get(slug: str, token: str | None) -> dict:
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "hephaestus-about-gate",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(f"https://api.github.com/repos/{slug}", headers=headers)
    with urllib.request.urlopen(request, timeout=20) as response:
        return json.loads(response.read().decode("utf-8"))


def fetch_live(slug: str) -> tuple[dict | None, str]:
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


def live_check() -> int:
    data, errors = load_canonical()
    if data is None:
        for line in errors:
            print(f"about: FAIL {line}")
        return 1
    slug = repo_slug()
    if not slug:
        return fail("cannot determine the GitHub repository (no origin remote, no GITHUB_REPOSITORY)")
    live, error = fetch_live(slug)
    if live is None:
        # Fail-closed: unreachable must never read as passing.
        return fail(
            f"cannot reach the GitHub API for {slug} after {API_RETRIES} attempts "
            f"({error}) — an unknown live About is NOT a passing About"
        )

    problems = []
    want_desc = data["description"]
    got_desc = live.get("description") or ""
    if want_desc != got_desc:
        problems.append(f"description:\n    canonical: {want_desc!r}\n    live:      {got_desc!r}")
    want_home = data["homepage"]
    got_home = live.get("homepage") or ""
    if want_home != got_home:
        problems.append(f"homepage: canonical {want_home!r}, live {got_home!r}")
    want_topics = sorted(data["topics"])
    got_topics = sorted(live.get("topics") or [])
    if want_topics != got_topics:
        # GitHub returns topics sorted; order carries no meaning for them.
        problems.append(f"topics: canonical {want_topics}, live {got_topics}")

    if problems:
        print(f"about: FAIL live GitHub About drifted from {CANONICAL.relative_to(ROOT)}")
        for line in problems:
            print(f"  {line}")
        print("  fix: run `make about-sync` (owner auth) and commit the result")
        return 1
    print(f"about: live matches canonical ({slug}): {len(data['description'])} chars, {len(want_topics)} topics")
    return 0


def gh_json(method: str, endpoint: str, payload: dict) -> None:
    """Call the GitHub API through `gh`; stdout stays out of the gate log."""
    subprocess.run(
        ["gh", "api", "-X", method, endpoint, "--input", "-"],
        input=json.dumps(payload),
        text=True,
        check=True,
        cwd=ROOT,
        capture_output=True,
    )


def sync() -> int:
    data, errors = load_canonical()
    if data is None:
        for line in errors:
            print(f"about: FAIL {line}")
        return 1
    slug = repo_slug()
    if not slug:
        return fail("cannot determine the GitHub repository")
    try:
        gh_json("PATCH", f"repos/{slug}", {
            "description": data["description"],
            "homepage": data["homepage"],
        })
        gh_json("PUT", f"repos/{slug}/topics", {"names": list(data["topics"])})
    except subprocess.CalledProcessError as exc:
        return fail(f"`gh` sync failed ({exc}) — check owner auth")
    today = _dt.date.today().isoformat()
    if data.get("verified") != today:
        data["verified"] = today
        CANONICAL.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"about: stamped verified={today} (commit {CANONICAL.relative_to(ROOT)})")
    return live_check()


def main() -> int:
    argv = sys.argv[1:]
    if "--live" in argv:
        return live_check()
    if "--sync" in argv:
        return sync()
    if argv:
        print("usage: check_repo_about.py [--live | --sync]")
        return 2
    return offline_check()


if __name__ == "__main__":
    sys.exit(main())
