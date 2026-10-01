"""Declared demo tools for the T-010 worker protocol.

Every function takes the job's ``input`` JSON and returns a JSON-serializable
result. No network, no filesystem writes except :func:`write_out`, which may
place one file under ``/work/out`` (sanitized to a basename).
"""

from __future__ import annotations

import os


def echo(value):
    """Return the input unchanged (happy-path demo)."""
    return value


def fail(value):
    """Always raise (error-path demo)."""
    raise RuntimeError("intentional tool failure: %r" % (value,))


def write_out(value):
    """Write ``{"name": ..., "content": ...}`` into /work/out (basename only)."""
    name = os.path.basename(str(value.get("name", "out.bin")))
    if not name or name in (".", ".."):
        raise ValueError("refused output name")
    path = os.path.join("/work/out", name)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(str(value.get("content", "")))
    return {"wrote": name}


def spin(value):
    """Burn CPU forever (timeout/DoS demo — the supervisor must kill it)."""
    while True:
        pass
