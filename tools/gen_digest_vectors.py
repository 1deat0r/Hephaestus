#!/usr/bin/env python3
"""Generate cross-language digest conformance vectors (T-004).

Writes crates/hephaestus/tests/fixtures/digest-vectors.json from the Python
reference profile (reference/qualification.py). Both
tests/test_digest_vectors.py and tests/digest_conformance.rs assert against
the generated file, so a drift in either language fails closed.

Run from the repository root: python3 tools/gen_digest_vectors.py
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from reference.qualification import plan_digest, record_digest, subject_digest  # noqa: E402

OUT = ROOT / "crates" / "hephaestus" / "tests" / "fixtures" / "digest-vectors.json"

FIXTURES = [
    "examples/software-mission.json",
    "examples/software-mission.v1.1.json",
    "crates/hephaestus/tests/fixtures/qualification-bundle.json",
]

SYNTHETIC = [
    {
        "_label": "unsorted keys, unicode, nested arrays, fixture-range floats",
        "record": {
            "zeta": [1, 2.5, {"b": False, "a": None}],
            "alpha": "value with é and … and \"quotes\" and \\backslash",
            "kind": "synthetic",
            "id": "SYN-UNICODE",
            "record_version": 1,
            "thresholds": [0.0, 0.2, 0.975, -0.05, 1000000],
        },
    },
    {
        "_label": "subject omissions and plan omissions",
        "record": {
            "kind": "experiment_plan",
            "id": "SYN-PLAN",
            "record_version": 3,
            "status": "ready",
            "registration": {"status": "draft", "payload_sha256": "0" * 64},
            "analysis": {"target_n": 40, "confidence_level": 0.975},
            "execution_receipt_ref": {"id": "RCPT-1", "version": 1},
            "analysis_receipt_ref": {"id": "RCPT-2", "version": 1},
            "reproduction_receipt_ref": None,
            "promotion_receipt_ref": None,
        },
    },
]


def main() -> int:
    vectors = []
    for fixture in FIXTURES:
        bundle = json.loads((ROOT / fixture).read_text())
        for record in bundle["records"]:
            entry = {
                "fixture": fixture,
                "id": record["id"],
                "record_digest": record_digest(record),
                "subject_digest": subject_digest(record),
            }
            if record["kind"] == "experiment_plan":
                entry["plan_digest"] = plan_digest(record)
            vectors.append(entry)
    for item in SYNTHETIC:
        record = item["record"]
        entry = {
            "synthetic_label": item["_label"],
            "synthetic": record,
            "record_digest": record_digest(record),
            "subject_digest": subject_digest(record),
        }
        if record["kind"] == "experiment_plan":
            entry["plan_digest"] = plan_digest(record)
        vectors.append(entry)
    OUT.write_text(json.dumps(vectors, indent=1, ensure_ascii=True) + "\n")
    print(f"wrote {OUT.relative_to(ROOT)} ({len(vectors)} vectors)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
