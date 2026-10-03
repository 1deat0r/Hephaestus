#!/usr/bin/env python3
"""Manifest-coverage classifier (ADR-024, layer 4; hashes since ADR-032).

Every tracked file must be covered by MANIFEST.sha256 or
tools/runtime_allowlist.txt; every MANIFEST entry must be tracked; the
allowlist must stay exact (no stale, no redundant entries); and every
recorded hash must match the bytes on disk. Fail-closed.

The hash leg exists because the file is called an integrity manifest
and nothing used to read its hashes: three entries had silently rotted
(ADR-032). Coverage alone proved the file was listed, never that it
still matched.
"""
import hashlib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "MANIFEST.sha256"
ALLOWLIST = ROOT / "tools" / "runtime_allowlist.txt"


def load_manifest() -> set[str]:
    paths: set[str] = set()
    for line in MANIFEST.read_text().splitlines():
        if not line.strip():
            continue
        _digest, path = line.split(None, 1)
        paths.add(path)
    return paths


def load_allowlist() -> set[str]:
    if not ALLOWLIST.exists():
        return set()
    entries: set[str] = set()
    for line in ALLOWLIST.read_text().splitlines():
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            entries.add(stripped)
    return entries


def verify_hashes() -> list[str]:
    """Every recorded digest must match the file on disk (ADR-032)."""
    errors: list[str] = []
    for line in MANIFEST.read_text().splitlines():
        if not line.strip():
            continue
        digest, path = line.split(None, 1)
        target = ROOT / path.strip()
        if target.is_symlink() or not target.is_file():
            errors.append(f"MANIFEST entry unreadable (missing or symlink): {path.strip()}")
            continue
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        if actual != digest:
            errors.append(
                f"MANIFEST hash mismatch: {path.strip()} — recorded {digest[:12]}..., "
                f"actual {actual[:12]}... (reseal that one line; ADR-032)"
            )
    return errors


def git_ls_files() -> set[str]:
    result = subprocess.run(
        ["git", "ls-files"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return set(result.stdout.splitlines())


def main() -> int:
    manifest = load_manifest()
    allow = load_allowlist()
    tracked = git_ls_files()
    errors: list[str] = []
    errors.extend(verify_hashes())

    for path in sorted(tracked - manifest - allow):
        errors.append(
            f"uncovered: {path} (not in MANIFEST, not allowlisted). "
            "Hint: append to tools/runtime_allowlist.txt (runtime file) — "
            "spec-package files belong to a versioned MANIFEST reseal, never a casual edit."
        )
    for path in sorted(manifest - tracked):
        errors.append(
            f"MANIFEST entry absent from git: {path} — restore the file or run the spec workflow."
        )
    for path in sorted(allow - tracked):
        errors.append(
            f"stale allowlist entry (not tracked): {path} — remove from tools/runtime_allowlist.txt."
        )
    for path in sorted(allow & manifest):
        errors.append(
            f"redundant allowlist entry (already in MANIFEST): {path} — remove from tools/runtime_allowlist.txt."
        )

    if errors:
        print("manifest-check: FAIL", file=sys.stderr)
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(
        f"manifest-check: ok (tracked={len(tracked)}, "
        f"manifest={len(manifest)}, allowlisted={len(allow)}, hashes verify)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
