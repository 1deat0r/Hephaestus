#!/usr/bin/env python3
"""Online external-link audit (ADR-024, T3 scheduled workflow).

Extracts http(s) links from tracked markdown (reusing the offline checker's
link extractor) and probes them with HEAD (falling back to GET). Reachability
signals only: 403/429 count as reachable (bot-blocked), transient 5xx is
retried once. Failures fail this run so the scheduled workflow goes red and
the owner is notified — pushes are never gated by this tool.
"""
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_md_links import FENCE_RE, INLINE_CODE_RE, extract_links  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
TIMEOUT = 10
USER_AGENT = "Hephaestus-link-audit/ADR-024 (scheduled freshness check)"
REACHABLE_CLIENT_ERRORS = {403, 429}


def external_links() -> dict:
    listed = subprocess.run(
        ["git", "ls-files", "-z", "*.md"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split("\0")
    found: dict = {}
    for rel in listed:
        if not rel:
            continue
        in_fence = False
        fence_marker = ""
        for lineno, line in enumerate((ROOT / rel).read_text().splitlines(), 1):
            fence = FENCE_RE.match(line)
            if fence:
                marker = fence.group(1)[0] * 3
                if not in_fence:
                    in_fence, fence_marker = True, marker
                elif marker == fence_marker:
                    in_fence = False
                continue
            if in_fence:
                continue
            for dest in extract_links(INLINE_CODE_RE.sub("", line)):
                if dest.startswith(("http://", "https://")):
                    found.setdefault(dest, []).append(f"{rel}:{lineno}")
    return found


def probe(url: str) -> str | None:
    """Return a failure reason, or None if reachable."""
    last = ""
    for attempt in (1, 2):
        for method in ("HEAD", "GET"):
            request = urllib.request.Request(
                url, method=method, headers={"User-Agent": USER_AGENT}
            )
            try:
                with urllib.request.urlopen(request, timeout=TIMEOUT) as response:
                    if response.status < 400 or response.status in REACHABLE_CLIENT_ERRORS:
                        return None
                    last = f"HTTP {response.status}"
            except urllib.error.HTTPError as exc:
                if exc.code in REACHABLE_CLIENT_ERRORS:
                    return None
                last = f"HTTP {exc.code}"
            except Exception as exc:  # noqa: BLE001 — report, don't hide
                last = f"{type(exc).__name__}: {exc}"
                if method == "GET":
                    break
        if attempt == 1:
            time.sleep(2)
    return last


def main() -> int:
    links = external_links()
    failures = []
    for url in sorted(links):
        reason = probe(url)
        if reason:
            failures.append((url, reason, links[url][:3]))
            print(f"unreachable: {url} ({reason}) first seen at {links[url][0]}")
    if failures:
        print(f"links-online: FAIL — {len(failures)}/{len(links)} external links unreachable")
        return 1
    print(f"links-online: ok ({len(links)} external links reachable)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
