"""Selected specification invariants, not a production runtime or scientific verifier.

This module checks relationships between already schema-valid records. It does
not establish whether a cited source is true, an analysis is statistically valid,
permissions are genuine, or a sandbox is secure.
"""
from __future__ import annotations
from datetime import datetime
import math
from typing import Any


def _refs(value: Any):
    if isinstance(value, dict):
        if set(value) == {"id", "version"}:
            yield value
        else:
            for item in value.values():
                yield from _refs(item)
    elif isinstance(value, list):
        for item in value:
            yield from _refs(item)


def _finite(value: Any) -> bool:
    if isinstance(value, float):
        return math.isfinite(value)
    if isinstance(value, dict):
        return all(_finite(v) for v in value.values())
    if isinstance(value, list):
        return all(_finite(v) for v in value)
    return True


def validate_bundle(bundle: dict[str, Any]) -> list[str]:
    """Return reason-coded semantic errors for a schema-valid example bundle."""
    records = bundle.get("records", [])
    errors: list[str] = []
    index: dict[tuple[str, int], dict[str, Any]] = {}
    for record in records:
        key = (record["id"], record["record_version"])
        if key in index:
            errors.append(f"DUPLICATE_VERSION: {key}")
        index[key] = record
        if not _finite(record):
            errors.append(f"NONFINITE_NUMBER: {record['id']}")
    for record in records:
        for ref in _refs(record):
            if (ref["id"], ref["version"]) not in index:
                errors.append(f"MISSING_REFERENCE: {record['id']} -> {ref}")

    def lookup(ref: dict[str, Any] | None, kind: str | None = None):
        found = index.get((ref["id"], ref["version"])) if ref else None
        if found and kind and found["kind"] != kind:
            errors.append(f"WRONG_REFERENCE_KIND: {ref} expected {kind}")
            return None
        return found

    for record in records:
        rid, kind = record["id"], record["kind"]
        mission = lookup(record.get("mission_ref"), "mission")
        if mission and "budget" in record:
            budget, limit = record["budget"], mission["budget"]
            if budget["currency"] != limit["currency"]:
                errors.append(f"CURRENCY_MISMATCH: {rid}")
            elif budget["minor_units"] > limit["minor_units"]:
                errors.append(f"TASK_CAP_EXCEEDS_MISSION: {rid}")
            # This is a per-record cap check, NOT transactional reservation logic.
        if kind == "hypothesis":
            lookup(record["opportunity_ref"], "opportunity")
            lookup(record["mechanism_ref"], "mechanism")
            pids = {p["id"] for p in record["predictions"]}
            if len(pids) != len(record["predictions"]):
                errors.append(f"DUPLICATE_PREDICTION: {rid}")
            for falsifier in record["falsifiers"]:
                if falsifier["prediction_id"] not in pids:
                    errors.append(f"UNKNOWN_FALSIFIER_TARGET: {rid}")
            if record["state"] in {"test_ready", "testing"}:
                covered = {f["prediction_id"] for f in record["falsifiers"]}
                if covered != pids or not record["alternatives"]:
                    errors.append(f"NO_CREDIBLE_DISCRIMINATOR: {rid}")
                if record["testability"]["status"] != "accessible" or record["testability"]["blockers"]:
                    errors.append(f"TESTABILITY_BLOCKED: {rid}")
        elif kind == "experiment_plan":
            hypothesis = lookup(record["hypothesis_ref"], "hypothesis")
            if hypothesis:
                pids = {p["id"] for p in hypothesis["predictions"]}
                if not set(record["prediction_ids"]).issubset(pids):
                    errors.append(f"UNKNOWN_PLAN_PREDICTION: {rid}")
                if record["mission_ref"] != hypothesis["mission_ref"]:
                    errors.append(f"MISSION_BINDING_MISMATCH: {rid}")
            a, reg = record["analysis"], record["registration"]
            if not set(a["primary_endpoints"]).issubset(record["prediction_ids"]):
                errors.append(f"UNKNOWN_PRIMARY_ENDPOINT: {rid}")
            active = record["status"] in {"ready", "running", "completed"}
            if active:
                if record["blockers"]:
                    errors.append(f"ACTIVE_PLAN_BLOCKED: {rid}")
                if a["method_qualification"] != "qualified_for_declared_assumptions":
                    errors.append(f"UNQUALIFIED_ANALYSIS: {rid}")
                if a["design"] == "fixed_sample" and a["target_n"] is None:
                    errors.append(f"SAMPLE_SIZE_UNSET: {rid}")
                if a["design"] == "sequential" and not a["anytime_valid_method"]:
                    errors.append(f"UNAPPROVED_OPTIONAL_STOPPING: {rid}")
                if a["data_partition"] == "confirmatory":
                    if reg["status"] != "frozen" or not reg["registered_at"] or not reg["payload_sha256"]:
                        errors.append(f"CONFIRMATION_NOT_REGISTERED: {rid}")
            if reg["status"] == "frozen" and (not reg["registered_at"] or not reg["payload_sha256"]):
                errors.append(f"INCOMPLETE_REGISTRATION: {rid}")
        elif kind == "experiment_result":
            plan = lookup(record["experiment_plan_ref"], "experiment_plan")
            lookup(record["hypothesis_ref"], "hypothesis")
            if plan and record["hypothesis_ref"] != plan["hypothesis_ref"]:
                errors.append(f"RESULT_HYPOTHESIS_MISMATCH: {rid}")
            if record["execution_validity"] != "valid":
                if record["scientific_conclusion"] in {"supported", "contradicted"} or record["engineering_target"] in {"met", "not_met"}:
                    errors.append(f"INVALID_EXECUTION_CONCLUSION: {rid}")
            elif not record["raw_artifact_hashes"] or not record["analysis_artifact_sha256"]:
                errors.append(f"RESULT_RECEIPTS_MISSING: {rid}")
            for finding in record["findings"]:
                if plan and finding["prediction_id"] not in plan["prediction_ids"]:
                    errors.append(f"UNKNOWN_RESULT_PREDICTION: {rid}")
                low, high = finding["lower_bound"], finding["upper_bound"]
                if low is not None and high is not None and low > high:
                    errors.append(f"REVERSED_INTERVAL: {rid}")
            if plan and record["data_opened_at"] and plan["analysis"]["data_partition"] == "confirmatory":
                registered = plan["registration"]["registered_at"]
                if not registered or datetime.fromisoformat(record["data_opened_at"]) < datetime.fromisoformat(registered):
                    errors.append(f"DATA_BEFORE_REGISTRATION: {rid}")
        elif kind == "task":
            if record["effect_class"] in {"unknown", "external_non_idempotent"} and record["retry_policy"] == "safe_retry":
                errors.append(f"UNSAFE_RETRY: {rid}")
            if record["effect_class"] == "unknown" and record["state"] in {"ready", "running"}:
                errors.append(f"UNKNOWN_EFFECT_DISPATCH: {rid}")
            for dep in record["dependency_refs"]:
                lookup(dep, "task")
            if record["state"] in {"ready", "running"} and mission:
                auth = mission["authorization"]
                if auth["state"] != "approved":
                    errors.append(f"AUTHORIZATION_NOT_APPROVED: {rid}")
                if not set(record["required_capabilities"]).issubset(auth["allowed_capabilities"]):
                    errors.append(f"CAPABILITY_NOT_GRANTED: {rid}")
        elif kind == "decision":
            lookup(record["task_ref"], "task")
            if record["selected_option"] is not None and record["selected_option"] not in record["options"]:
                errors.append(f"INVALID_DECISION_OPTION: {rid}")
            probs = record["probabilities"]
            if probs is not None and (len(probs) != len(record["options"]) or abs(sum(probs) - 1.0) > 1e-6):
                errors.append(f"INVALID_PROBABILITY_VECTOR: {rid}")
            if record["calibration_status"] == "qualified_for_declared_domain" and not record["calibration_artifact_sha256"]:
                errors.append(f"CALIBRATION_RECEIPT_MISSING: {rid}")
            if record["abstained"] and record["selected_option"] is not None:
                errors.append(f"ABSTENTION_CONFLICT: {rid}")
        elif kind == "dossier":
            result_records = [lookup(ref, "experiment_result") for ref in record["result_refs"]]
            for ref in record["hypothesis_refs"]:
                lookup(ref, "hypothesis")
            if record["status"] == "validated_candidate":
                if record["unresolved_blockers"]:
                    errors.append(f"PROMOTION_BLOCKED: {rid}")
                qualifying = [r for r in result_records if r and r["execution_validity"] == "valid" and r["scientific_conclusion"] == "supported" and r["engineering_target"] == "met"]
                if not qualifying:
                    errors.append(f"PROMOTION_EVIDENCE_MISSING: {rid}")
                if record["reproduction_status"] != "independent_pass" or not record["reproduction_artifact_sha256"]:
                    errors.append(f"REPRODUCTION_MISSING: {rid}")
                if record["novelty_report"]["status"] not in {"near_match", "no_match_within_search_scope"}:
                    errors.append(f"NOVELTY_UNRESOLVED_FOR_PROMOTION: {rid}")
            if record["reproduction_status"] == "independent_pass" and not record["reproduction_artifact_sha256"]:
                errors.append(f"REPRODUCTION_RECEIPT_MISSING: {rid}")

    # DAG validation is independent of schema validation.
    tasks = {(r["id"], r["record_version"]): r for r in records if r["kind"] == "task"}
    visiting: set[tuple[str, int]] = set()
    visited: set[tuple[str, int]] = set()
    def visit(key: tuple[str, int]):
        if key in visiting:
            errors.append(f"TASK_DAG_CYCLE: {key}")
            return
        if key in visited or key not in tasks:
            return
        visiting.add(key)
        for ref in tasks[key]["dependency_refs"]:
            visit((ref["id"], ref["version"]))
        visiting.remove(key)
        visited.add(key)
    for key in tasks:
        visit(key)
    return errors
