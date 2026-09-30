#!/usr/bin/env python3
"""Runtime gate seal (ADR-024): sha256 over the files that define 'green'.

Gate files can never join the frozen MANIFEST.sha256 envelope (Q9a/ADR-017/
ADR-019), so this seal is the tamper-detection substitute for the runtime
side. The sealed set and the files on disk must match exactly (no unsealed
gate files, no stale seal entries) and every hash must verify. Fail-closed.

  python tools/gate_seal.py --write   # regenerate after an intentional gate change
  python tools/gate_seal.py --check   # verify (wired into `make ci`)
"""
import hashlib
import sys
from fnmatch import fnmatch
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SEAL = ROOT / "tools" / "gate_seal.sha256"

EXTRA_DATA_FILES = ("tools/runtime_allowlist.txt", "tools/md_links_baseline.txt")


def gate_paths() -> list:
    paths = {"Makefile"}
    for directory, pattern in (
        (ROOT / ".github" / "workflows", "*.yml"),
        (ROOT / ".githooks", "*"),
        (ROOT / "tools", "*.py"),
    ):
        if directory.exists():
            for item in sorted(directory.iterdir()):
                if item.is_file() and fnmatch(item.name, pattern):
                    paths.add(item.relative_to(ROOT).as_posix())
    paths.update(p for p in EXTRA_DATA_FILES if (ROOT / p).exists())
    return sorted(paths)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_seal() -> dict:
    entries = {}
    for line in SEAL.read_text().splitlines():
        if line.strip():
            file_hash, path = line.split(None, 1)
            entries[path] = file_hash
    return entries


def write() -> int:
    lines = [f"{digest(ROOT / path)}  {path}" for path in gate_paths()]
    SEAL.write_text("\n".join(lines) + "\n")
    print(f"gate-seal: wrote {len(lines)} entries to {SEAL.relative_to(ROOT)}")
    return 0


def check() -> int:
    if not SEAL.exists():
        print(
            "gate-seal: FAIL — tools/gate_seal.sha256 missing. "
            "Hint: run `python tools/gate_seal.py --write` and cite ADR-024.",
            file=sys.stderr,
        )
        return 1
    sealed = load_seal()
    current = set(gate_paths())
    errors = []
    for path in sorted(current - set(sealed)):
        errors.append(
            f"unsealed gate file: {path} — run "
            "`python tools/gate_seal.py --write` (cite ADR-024 in the commit)"
        )
    for path in sorted(set(sealed) - current):
        errors.append(
            f"stale seal entry (file gone): {path} — regenerate the seal"
        )
    for path in sorted(current & set(sealed)):
        if digest(ROOT / path) != sealed[path]:
            errors.append(
                f"seal mismatch: {path} — if the change is intentional, "
                "regenerate with `python tools/gate_seal.py --write` and cite ADR-024"
            )
    if errors:
        print("gate-seal: FAIL", file=sys.stderr)
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"gate-seal: ok ({len(sealed)} gate files sealed)")
    return 0


if __name__ == "__main__":
    sys.exit(write() if "--write" in sys.argv else check())
