#!/usr/bin/env python3
"""Offline internal link/anchor checker for tracked markdown (ADR-024, L3 slice).

Checks relative link targets and `#anchor` fragments against ATX headings /
id attributes of .md files. External schemes are skipped by design (a
scheduled online audit covers them). Fail-closed: any parse/IO error fails.
Suppression is only via the enumerated baseline file (tools/md_links_baseline.txt),
which may shrink but never grow in CI.
"""
import re
import subprocess
import sys
import urllib.parse
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "tools" / "md_links_baseline.txt"
EXTERNAL_SCHEMES = ("http:", "https:", "mailto:", "ftp:", "tel:", "data:", "//")

FENCE_RE = re.compile(r"^ {0,3}(`{3,}|~{3,})")
REF_DEF_RE = re.compile(r"^ {0,3}\[([^\]]+)\]:\s*(\S+)")
HEADING_RE = re.compile(r"^ {0,3}#{1,6}\s+(.*?)\s*#*\s*$")
ID_ATTR_RE = re.compile(r"""\bid=["']([^"']+)["']""")
INLINE_CODE_RE = re.compile(r"`[^`]*`")


def github_slug(text: str, seen: dict) -> str:
    """GitHub-style heading slug; tracks duplicates for -1/-2 suffixes."""
    stripped = re.sub(r"[*_~]", "", text).strip().lower()
    cleaned = "".join(ch for ch in stripped if ch.isalnum() or ch in " -")
    slug = cleaned.replace(" ", "-")
    count = seen.get(slug, 0)
    seen[slug] = count + 1
    return slug if count == 0 else f"{slug}-{count}"


def anchors_of(text: str) -> set:
    anchors: set = set()
    seen: dict = {}
    in_fence = False
    fence_marker = ""
    for line in text.splitlines():
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
        heading = HEADING_RE.match(line)
        if heading:
            anchors.add(github_slug(heading.group(1), seen))
        anchors.update(ID_ATTR_RE.findall(line))
    return anchors


def strip_fences(lines: list) -> list:
    """Blank out fenced code blocks so example links are not validated."""
    out = []
    in_fence = False
    fence_marker = ""
    for line in lines:
        fence = FENCE_RE.match(line)
        if fence:
            marker = fence.group(1)[0] * 3
            if not in_fence:
                in_fence, fence_marker = True, marker
                out.append("")
                continue
            if marker == fence_marker:
                in_fence = False
            out.append("")
            continue
        out.append("" if in_fence else line)
    return out


def extract_links(line: str) -> list:
    """Return destinations from inline links (paren-depth scan) + ref defs."""
    found = []
    ref = REF_DEF_RE.match(line)
    if ref:
        found.append(ref.group(2))
    i = 0
    while True:
        start = line.find("](", i)
        if start == -1:
            break
        depth = 1
        j = start + 2
        dest = []
        while j < len(line):
            ch = line[j]
            if ch == "\\":
                if j + 1 < len(line):
                    dest.append(line[j + 1])
                    j += 2
                    continue
            elif ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
                if depth == 0:
                    break
            dest.append(ch)
            j += 1
        if depth == 0:
            raw = "".join(dest).strip()
            if raw.startswith("<") and raw.endswith(">"):
                raw = raw[1:-1]
            if raw:
                found.append(raw)
        i = start + 2
    return found


def check_file(md_path: Path, anchor_cache: dict) -> list:
    errors = []
    rel = md_path.relative_to(ROOT).as_posix()
    text = md_path.read_text(encoding="utf-8")
    lines = strip_fences(text.splitlines())
    for lineno, line in enumerate(lines, 1):
        code_stripped = INLINE_CODE_RE.sub("", line)
        for dest in extract_links(code_stripped):
            if not dest or dest.startswith(EXTERNAL_SCHEMES):
                continue
            dest = urllib.parse.unquote(dest)
            frag = ""
            if "#" in dest:
                dest, frag = dest.split("#", 1)
            if dest:
                target = (md_path.parent / dest).resolve()
            else:
                target = md_path.resolve()
            try:
                target.relative_to(ROOT.resolve())
            except ValueError:
                errors.append(
                    f"{rel}:{lineno}: link '{dest}' escapes the repository"
                )
                continue
            if not target.exists():
                errors.append(
                    f"{rel}:{lineno}: broken link '{dest}' (target missing). "
                    "Hint: fix the path or create the referenced file."
                )
                continue
            if frag and target.suffix == ".md":
                if target not in anchor_cache:
                    anchor_cache[target] = anchors_of(
                        target.read_text(encoding="utf-8")
                    )
                if frag not in anchor_cache[target]:
                    shown = dest if dest else f"#{frag}"
                    errors.append(
                        f"{rel}:{lineno}: broken link '{shown}' "
                        f"(anchor '#{frag}' missing). "
                        "Hint: correct the fragment to match a heading."
                    )
    return errors


def load_baseline() -> set:
    if not BASELINE.exists():
        return set()
    return {
        line.strip()
        for line in BASELINE.read_text().splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def main() -> int:
    listed = subprocess.run(
        ["git", "ls-files", "-z", "*.md"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split("\0")
    anchor_cache: dict = {}
    try:
        violations: set = set()
        for rel in listed:
            if rel:
                violations.update(check_file(ROOT / rel, anchor_cache))
    except Exception as exc:  # fail-closed: parse/IO errors are failures
        print(f"md-links: FAIL — checker error: {exc!r}", file=sys.stderr)
        return 1

    baseline = load_baseline()
    new = sorted(violations - baseline)
    stale = sorted(baseline - violations)
    if new or stale:
        print("md-links: FAIL", file=sys.stderr)
        for entry in new:
            print(entry, file=sys.stderr)
        for entry in stale:
            print(
                f"stale baseline entry (no longer fires): {entry} — "
                f"remove from {BASELINE.relative_to(ROOT)}",
                file=sys.stderr,
            )
        if new:
            print(
                "baseline is shrink-only: fix the link; entries may be removed "
                "but never added in CI",
                file=sys.stderr,
            )
        return 1
    print(f"md-links: ok ({sum(1 for r in listed if r)} md files, {len(baseline)} baseline entries)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
