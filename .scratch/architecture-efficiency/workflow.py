#!/usr/bin/env python3
"""Select architecture work and preserve compact verification receipts."""

import argparse
import json
from pathlib import Path
import re
import shlex
import subprocess
import tempfile
import time

FEATURE = Path(__file__).resolve().parent
ROOT = FEATURE.parent.parent
STATUS = re.compile(r"^\*\*Status:\*\*\s*(\S+)", re.M)
VERIFY = re.compile(r"^\*\*Verify:\*\*\s*(.+)", re.M)
SMALL = re.compile(r"^\d+\. \[([ x])\] \*\*(S\d+)\*\* (.+)", re.M)
ZERO = re.compile(r"running 0 tests|test result: ok\. 0 passed|Ran 0 tests|collected 0 items")


def tickets():
    result = {}
    for path in sorted((FEATURE / "issues").glob("*.md")):
        text = path.read_text()
        key = "AE-" + path.name[:2]
        starts = list(SMALL.finditer(text))
        smalls = []
        for index, match in enumerate(starts):
            end = starts[index + 1].start() if index + 1 < len(starts) else len(text)
            block = text[match.end():end]
            status = re.search(r"^   \*\*Status:\*\* (\S+)", block, re.M).group(1)
            verify = re.search(r"^   \*\*Verify:\*\* (.+)", block, re.M).group(1)
            smalls.append({"id": match[2], "title": match[3], "done": match[1] == "x",
                           "status": status, "verify": verify})
        deps = re.search(r"^\*\*Blocked by:\*\* (.+)", text, re.M).group(1)
        result[key] = {"path": str(path.relative_to(ROOT)), "status": STATUS.search(text)[1],
                       "verify": VERIFY.search(text)[1], "smalls": smalls,
                       "dependencies": ["AE-" + number for number in re.findall(r"\b\d{2}\b", deps)]}
    return result


def check(items):
    assert len(items) == 7, "Expected seven TASKS"
    settings = json.loads((ROOT / ".pi/settings.json").read_text())
    assert settings["defaultProvider"] == "xiaomi"
    assert settings["defaultModel"] == "mimo-v2.6-flash"
    assert settings["enabledModels"] == ["xiaomi/mimo-v2.6-flash"]
    visited, active = set(), set()

    def visit(key):
        assert key in items, f"Unknown dependency: {key}"
        assert key not in active, f"Dependency cycle: {key}"
        if key in visited:
            return
        active.add(key)
        for dependency in items[key]["dependencies"]:
            visit(dependency)
        active.remove(key)
        visited.add(key)

    for key, item in items.items():
        visit(key)
        assert len(item["smalls"]) == 3, f"Expected three small tasks: {key}"
        for small in item["smalls"]:
            assert small["done"] == (small["status"] == "done"), f"Checkbox/status mismatch: {key}/{small['id']}"
            command = shlex.split(small["verify"])
            assert command[:5] == ["cargo", "test", "-p", "hephaestus", "--test"]
            assert len(command) == 7 and command[5].startswith("architecture_")
            assert command[6].startswith(small["id"].lower() + "_")
        if item["status"] == "done":
            assert all(small["done"] for small in item["smalls"]), f"Incomplete TASK: {key}"
    return {"status": "PASS", "tasks": len(items), "small_tasks": sum(len(i["smalls"]) for i in items.values())}


def next_task(items):
    for key, item in items.items():
        if item["status"] == "done" or item["status"] not in {"ready-for-agent", "blocked", "pending"}:
            continue
        if any(items[dep]["status"] != "done" for dep in item["dependencies"]):
            continue
        for small in item["smalls"]:
            if not small["done"]:
                return {"task": key, "ticket": item["path"], "small_task": small["id"],
                        "title": small["title"], "verify": small["verify"],
                        "status_update_needed": item["status"] == "blocked"}
    return {"task": None, "reason": "All tasks complete or dependencies block remaining work"}


def verify(items, task_id, small_id):
    """Run one small task's suite, or every small-task suite of a TASK.

    A TASK-level Verify runs each small task's declared command and is
    green only when every prefix suite is green and non-empty (extends
    ADR-031: a Verify that skips a small task's suite proves nothing
    about that suite). Per-command zero-test protection is unchanged.
    """
    item = items[task_id]
    if small_id:
        command = next(s["verify"] for s in item["smalls"] if s["id"] == small_id)
        commands = [(small_id, command)]
    else:
        commands = [(s["id"], s["verify"]) for s in item["smalls"]]
    with tempfile.NamedTemporaryFile(mode="w+", prefix=f"hephaestus-{task_id}-{small_id or 'TASK'}-",
                                     suffix=".log", delete=False) as log:
        log.write(json.dumps({"commands": [{"small": sid, "command": cmd} for sid, cmd in commands],
                              "cwd": str(ROOT)}) + "\n")
        log.flush()
        start_time = time.monotonic()
        results = []
        for sid, cmd in commands:
            log.write(f"### {sid}: {cmd}\n")
            log.flush()
            result = subprocess.run(shlex.split(cmd), cwd=ROOT, stdout=subprocess.PIPE,
                                    stderr=subprocess.STDOUT, text=True)
            output = result.stdout or ""
            log.write(output + "\n")
            log.flush()
            summaries = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed", output)
            nonempty = any(int(passed) > 0 and int(failed) == 0 for passed, failed in summaries)
            green = result.returncode == 0 and nonempty and not ZERO.search(output)
            entry = {"small": sid, "command": cmd, "exit_code": result.returncode,
                     "nonempty_tests": nonempty, "green": green}
            if not green:
                errors = [line for line in output.splitlines() if line.startswith("error:")]
                entry["failure_summary"] = "\n".join(errors)[:1200] if errors else output[-1200:]
            results.append(entry)
        elapsed = round(time.monotonic() - start_time, 3)
        green_all = all(entry["green"] for entry in results)
        receipt = {"status": "PASS" if green_all else "FAIL",
                   "exit_code": 0 if green_all else 1,
                   "nonempty_tests": all(entry["nonempty_tests"] for entry in results),
                   "elapsed_seconds": elapsed, "log": log.name, "smalls": results}
        if not green_all:
            receipt["failure_summary"] = "\n".join(
                f"{entry['small']}: {entry.get('failure_summary', '')}"
                for entry in results if not entry["green"])[:1200]
        return receipt, 0 if green_all else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["check", "next", "verify"])
    parser.add_argument("task", nargs="?")
    parser.add_argument("small", nargs="?")
    args = parser.parse_args()
    try:
        items = tickets()
        validation = check(items)
        code = 0
        if args.action == "check":
            result = validation
        elif args.action == "next":
            result = next_task(items)
        else:
            result, code = verify(items, args.task, args.small)
        print(json.dumps(result, separators=(",", ":")))
        return code
    except (AssertionError, AttributeError, KeyError, StopIteration, ValueError, OSError) as error:
        print(json.dumps({"status": "FAIL", "error": str(error)}, separators=(",", ":")))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
