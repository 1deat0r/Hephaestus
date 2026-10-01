"""In-sandbox job runner (T-010).

Reads a job file, validates the requested tool against the job's own
declared list (defense in depth — the provider already enforced argv),
invokes ``module:function``, and writes ``/work/out/result.json``.

No third-party imports; no network; exits 0 on success, 2 for an
undeclared tool, 1 for a failing tool.
"""

from __future__ import annotations

import importlib
import json
import os
import sys
import traceback

OUT_DIR = "/work/out"
RESULT_PATH = os.path.join(OUT_DIR, "result.json")


def fail(code: int, message: str) -> None:
    sys.stderr.write(message + "\n")
    sys.stderr.flush()
    raise SystemExit(code)


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        fail(1, "usage: runner.py <job.json>")
    with open(argv[1], encoding="utf-8") as handle:
        job = json.load(handle)

    tool = job.get("tool")
    declared = job.get("declared_tools")
    if not isinstance(tool, str) or not isinstance(declared, list) or tool not in declared:
        # Undeclared at the worker layer too (defense in depth).
        fail(2, f"TOOL_DENIED: {tool!r} not in declared_tools")

    module_name, _, func_name = tool.partition(":")
    if not module_name or not func_name:
        fail(2, f"TOOL_DENIED: malformed tool {tool!r}")
    try:
        module = importlib.import_module(module_name)
        func = getattr(module, func_name)
    except Exception:
        traceback.print_exc(file=sys.stderr)
        fail(1, f"TOOL_LOAD_FAILED: {tool}")

    try:
        result = func(job.get("input"))
    except Exception:
        traceback.print_exc(file=sys.stderr)
        fail(1, f"TOOL_FAILED: {tool}")

    payload = {"status": "ok", "tool": tool, "result": result}
    os.makedirs(OUT_DIR, exist_ok=True)
    with open(RESULT_PATH, "w", encoding="utf-8") as handle:
        json.dump(payload, handle)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
