#!/usr/bin/env python3
"""README shell-fence assertion (ADR-024, L3 slice).

Every command in README.md ```sh fences must parse against a small grammar:
a known `make` target, an existing `tools/*.py` script, or `python -m ...`
module invocations. Unparseable = fail (ADR-024; illustrative commands are
statically checked, not executed).
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "README.md"
MAKEFILE = ROOT / "Makefile"

FENCE_RE = re.compile(r"^ {0,3}(`{3,}|~{3,})\s*(\S*)\s*$")
TARGET_RE = re.compile(r"^([A-Za-z][A-Za-z0-9_-]*):(?:\s|$)")
ASSIGN_PREFIX_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=\S*\s+")
SHELL_LANGS = {"sh", "bash", "shell", "console"}


def make_targets() -> set:
    targets = set()
    for line in MAKEFILE.read_text().splitlines():
        match = TARGET_RE.match(line)
        if match and match.group(1) != ".PHONY":
            targets.add(match.group(1))
    return targets


def commands_of_fence(lines: list) -> list:
    """Join backslash continuations, drop comments/blank lines."""
    joined = []
    buffer = ""
    for line in lines:
        buffer = (buffer + " " + line.strip()) if buffer else line.strip()
        if buffer.endswith("\\"):
            buffer = buffer[:-1].rstrip()
            continue
        joined.append(buffer)
        buffer = ""
    if buffer:
        joined.append(buffer)
    commands = []
    for line in joined:
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        stripped = ASSIGN_PREFIX_RE.sub("", stripped)
        if stripped:
            commands.append(stripped)
    return commands


def validate(command: str, targets: set) -> str | None:
    """Return an error message, or None if the command is acceptable."""
    parts = command.split()
    head = parts[0] if parts else ""
    if head == "make":
        args = [p for p in parts[1:] if not p.startswith("-")]
        if not args:
            return "make with no target"
        known = ", ".join(sorted(targets))
        if args[0] not in targets:
            return f"matches no target/tool (unknown make target '{args[0]}'). Known: make {{{known}}}"
        return None
    if head in {"python", "python3"}:
        rest = parts[1:]
        if "-m" in rest:
            return None  # module invocations (unittest, pip, ...) are allowed
        if rest and (ROOT / rest[0]).exists() and rest[0].endswith(".py"):
            return None
        if rest and rest[0].endswith(".py"):
            return f"matches no target/tool (script '{rest[0]}' does not exist)"
        return "matches no target/tool (expected `python tools/*.py` or `python -m ...`)"
    return f"matches no target/tool (unsupported command '{head}')"


def main() -> int:
    targets = make_targets()
    errors = []
    in_fence = False
    fence_marker = ""
    lang = ""
    fence_start = 0
    body: list = []
    for lineno, line in enumerate(README.read_text().splitlines(), 1):
        fence = FENCE_RE.match(line)
        if fence and not in_fence:
            in_fence, fence_marker = True, fence.group(1)[0] * 3
            lang, fence_start, body = fence.group(2).lower(), lineno, []
            continue
        if in_fence:
            if fence and line.startswith(fence_marker):
                if lang in SHELL_LANGS:
                    for command in commands_of_fence(body):
                        problem = validate(command, targets)
                        if problem:
                            errors.append(
                                f"README.md:{fence_start + 1}: '{command}' {problem}"
                            )
                in_fence = False
                continue
            body.append(line)
    if in_fence:
        errors.append(
            f"README.md:{fence_start}: unterminated code fence (fail-closed)"
        )
    if errors:
        print("readme-fences: FAIL", file=sys.stderr)
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print("readme-fences: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
