"""Fail-closed qualification metadata checks, NOT authentication or science.

ValidationContext must come from a protected verifier, never from a bundle/model.
The future runtime must establish signatures, artifact integrity, and freshness.
Test contexts are synthetic and demonstrate contract behavior only.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
import hashlib
import json
from typing import Any

@dataclass(frozen=True)
class ValidationContext:
    trusted_receipts: frozenset[tuple[str, int]] = frozenset()
    trusted_qualifications: frozenset[tuple[str, int]] = frozenset()
    trusted_grants: frozenset[tuple[str, int]] = frozenset()
    trusted_holdout_accesses: frozenset[tuple[str, int]] = frozenset()
    trusted_families: frozenset[tuple[str, int]] = frozenset()
    authenticated_record_hashes: dict[tuple[str, int], str] = field(default_factory=dict)
    verified_artifacts: frozenset[str] = frozenset()
    current_versions: dict[str, int] = field(default_factory=dict)
    current_record_hashes: dict[tuple[str, int], str] = field(default_factory=dict)
    current_policies: dict[str, str] = field(default_factory=dict)
    evaluated_at: datetime | None = None
    synthetic_fixture_mode: bool = False

def plan_digest(plan: dict[str, Any]) -> str:
    """Canonical registered payload: all fields except status and registration.

    UTF-8, sorted object keys, compact separators, finite JSON numbers. Version
    1.1 uses this Python JSON profile; cross-language implementations need vectors.
    """
    payload = {k: v for k, v in plan.items() if k not in {'status', 'registration'}}
    return hashlib.sha256(json.dumps(payload, sort_keys=True, separators=(',', ':'),
                                     ensure_ascii=False, allow_nan=False).encode()).hexdigest()

def record_digest(record: dict[str, Any]) -> str:
    return hashlib.sha256(json.dumps(record, sort_keys=True, separators=(',', ':'),
                                     ensure_ascii=False, allow_nan=False).encode()).hexdigest()

def subject_digest(record: dict[str, Any]) -> str:
    """Assessment payload excludes only receipt references to avoid hash cycles."""
    omitted = {'execution_receipt_ref', 'analysis_receipt_ref',
               'reproduction_receipt_ref', 'promotion_receipt_ref'}
    return record_digest({k: v for k, v in record.items() if k not in omitted})

def validate_qualification(records: list[dict[str, Any]], ctx: ValidationContext) -> list[str]:
    errors: list[str] = []
    index = {(r['id'], r['record_version']): r for r in records}
    def key(ref):
        return (ref['id'], ref['version']) if ref else None
    def resolve(ref, kind):
        found = index.get(key(ref))
        return found if found and found['kind'] == kind else None
    def ref_of(r):
        return {'id': r['id'], 'version': r['record_version']}
    def error(code, r):
        errors.append(f"{code}: {r['id']}")
    def trusted(ref, allowed):
        found = index.get(key(ref))
        if not found or key(ref) not in allowed:
            return False
        try:
            return ctx.authenticated_record_hashes.get(key(ref)) == record_digest(found)
        except ValueError:
            return False
    def artifacts(hashes, r):
        if not set(hashes).issubset(ctx.verified_artifacts):
            error('ARTIFACT_NOT_VERIFIED', r)
    def check_grant(r, mission, *, artifact, capabilities, operation, destination):
        grant = resolve(r.get('grant_ref'), 'authorization_grant')
        if not grant or not trusted(r.get('grant_ref'), ctx.trusted_grants):
            error('GRANT_NOT_TRUSTED', r)
            return
        if (grant['mission_ref'] != ref_of(mission) or grant['state'] != 'approved'
                or grant['policy_version'] != mission['authorization']['policy_version']
                or ctx.current_policies.get(mission['id']) != grant['policy_version']
                or grant['artifact_sha256'] != artifact
                or grant['operation_id'] != operation or grant['destination'] != destination
                or not set(capabilities).issubset(grant['capabilities'])
                or not set(capabilities).issubset(mission['authorization']['allowed_capabilities'])
                or mission['authorization']['state'] != 'approved'):
            error('GRANT_BINDING_MISMATCH', r)
        if (destination is not None
                and destination not in mission['authorization']['allowed_destinations']):
            error('DESTINATION_NOT_GRANTED', r)
        if (r['budget']['currency'] != grant['max_cost']['currency']
                or r['budget']['minor_units'] > grant['max_cost']['minor_units']):
            error('GRANT_COST_EXCEEDED', r)
        now = ctx.evaluated_at
        if (now is None or now.tzinfo is None
                or not datetime.fromisoformat(grant['issued_at']) <= now
                < datetime.fromisoformat(grant['expires_at'])):
            error('GRANT_EXPIRED_OR_TIME_UNKNOWN', r)

    def receipt(ref, purpose, plan, hypothesis, owner, required_artifacts):
        found = resolve(ref, 'verification_receipt')
        if not found or not trusted(ref, ctx.trusted_receipts):
            error('RECEIPT_NOT_TRUSTED', owner)
            return None
        mission = resolve(plan['mission_ref'], 'mission')
        expected = {
            'mission_ref': plan['mission_ref'], 'hypothesis_ref': ref_of(hypothesis),
            'experiment_plan_ref': ref_of(plan),
            'candidate_sha256': plan['intervention_artifact_sha256'],
            'evaluator_sha256': plan['protected_evaluator_sha256'],
            'analysis_implementation_sha256': plan['analysis']['implementation_sha256'],
            'evidence_snapshot_sha256': plan['evidence_snapshot_sha256'],
            'policy_version': mission['authorization']['policy_version'] if mission else None,
            'registration_sha256': plan['registration']['payload_sha256'],
        }
        if found['purpose'] != purpose or found['bindings'] != expected or found['outcome'] != 'pass':
            error('RECEIPT_BINDING_MISMATCH', owner)
        if (found['subject_ref'] != ref_of(owner)
                or found['subject_payload_sha256'] != subject_digest(owner)):
            error('RECEIPT_SUBJECT_MISMATCH', owner)
        if (not set(plan['analysis']['primary_endpoints']).issubset(found['endpoint_ids'])
                or not set(plan['guardrail_ids']).issubset(found['guardrail_ids'])):
            error('RECEIPT_COVERAGE_MISSING', owner)
        if not set(required_artifacts).issubset(found['artifact_hashes']):
            error('RECEIPT_ARTIFACT_MISMATCH', owner)
        artifacts(found['artifact_hashes'], owner)
        return found

    for r in records:
        kind = r['kind']
        if kind == 'source':
            continue
        if kind == 'evidence':
            for source_ref in r['source_refs']:
                if not resolve(source_ref, 'source'):
                    error('EVIDENCE_SOURCE_KIND_MISMATCH', r)
            for span in r['source_spans']:
                if span['source_ref'] not in r['source_refs']:
                    error('SPAN_SOURCE_NOT_DECLARED', r)
        if kind == 'mechanism':
            if not resolve(r['opportunity_ref'], 'opportunity'):
                error('MECHANISM_OPPORTUNITY_MISMATCH', r)
        if kind == 'hypothesis':
            opp = resolve(r['opportunity_ref'], 'opportunity')
            mech = resolve(r['mechanism_ref'], 'mechanism')
            if ((opp and opp['mission_ref'] != r['mission_ref'])
                    or (mech and mech['opportunity_ref'] != r['opportunity_ref'])):
                error('HYPOTHESIS_LINEAGE_MISMATCH', r)
        if kind == 'task' and r['state'] in {'ready', 'running'}:
            mission = resolve(r['mission_ref'], 'mission')
            if mission:
                check_grant(r, mission, artifact=r.get('approval_artifact_sha256'),
                            capabilities=r['required_capabilities'], operation=r['operation_id'],
                            destination=r.get('destination'))
        if kind == 'authorization_grant':
            if datetime.fromisoformat(r['expires_at']) <= datetime.fromisoformat(r['issued_at']):
                error('GRANT_INVALID_INTERVAL', r)
        if kind == 'experiment_family':
            member_plans = [resolve(p, 'experiment_plan') for p in r['plan_refs']]
            test_count = sum(len(p['analysis']['primary_endpoints']) for p in member_plans if p)
            if test_count > r['max_confirmatory_tests']:
                error('FAMILY_TEST_LIMIT_EXCEEDED', r)
            if r['strategy'] not in {'exploratory_only', 'nonstatistical'} and r['alpha'] is None:
                error('FAMILY_ALPHA_MISSING', r)
            for plan_ref in r['plan_refs']:
                plan = resolve(plan_ref, 'experiment_plan')
                if not plan or plan.get('family_ref') != ref_of(r) or plan['mission_ref'] != r['mission_ref']:
                    error('FAMILY_MEMBERSHIP_MISMATCH', r)
        if kind == 'holdout_access':
            family = resolve(r['family_ref'], 'experiment_family')
            plan = resolve(r['plan_ref'], 'experiment_plan')
            if not family or not plan or r['plan_ref'] not in family['plan_refs']:
                error('HOLDOUT_MEMBERSHIP_MISMATCH', r)
            elif (family['retired'] or r['query_index'] > family['max_confirmatory_tests']
                  or r['feedback'] != 'sealed_until_campaign_end'):
                error('HOLDOUT_REUSE_FORBIDDEN', r)
            if plan:
                registered = plan['registration']['registered_at']
                if not registered or datetime.fromisoformat(r['opened_at']) < datetime.fromisoformat(registered):
                    error('HOLDOUT_BEFORE_REGISTRATION', r)
        if kind == 'experiment_plan':
            active = r['status'] in {'ready', 'running', 'completed'}
            if r['registration']['status'] == 'frozen':
                try:
                    if r['registration']['payload_sha256'] != plan_digest(r):
                        error('REGISTRATION_DIGEST_MISMATCH', r)
                except ValueError:
                    error('REGISTRATION_PAYLOAD_INVALID', r)
            if active:
                method = resolve(r.get('qualification_ref'), 'method_qualification')
                if not method or not trusted(r.get('qualification_ref'), ctx.trusted_qualifications):
                    error('METHOD_NOT_TRUSTED', r)
                else:
                    a = r['analysis']
                    if (method['state'] != 'qualified' or method['method_id'] != a['method_id']
                            or a['design'] not in method['designs']
                            or method['implementation_sha256'] != a['implementation_sha256']
                            or not set(a['assumptions']).issubset(method['assumptions'])):
                        error('METHOD_BINDING_MISMATCH', r)
                    mission = resolve(r['mission_ref'], 'mission')
                    if mission and method['domain'] not in mission['domains']:
                        error('METHOD_DOMAIN_MISMATCH', r)
                    artifacts([method['qualification_artifact_sha256'], method['implementation_sha256']], r)
                mission = resolve(r['mission_ref'], 'mission')
                if mission:
                    check_grant(r, mission, artifact=r['intervention_artifact_sha256'],
                                capabilities=r['required_capabilities'], operation=r['operation_id'],
                                destination=r['destination'])
                if r['analysis']['data_partition'] == 'confirmatory':
                    family = resolve(r.get('family_ref'), 'experiment_family')
                    if (not family or not trusted(r.get('family_ref'), ctx.trusted_families)
                            or family['retired'] or family['strategy'] == 'exploratory_only'
                            or ref_of(r) not in family['plan_refs']
                            or family['mission_ref'] != r['mission_ref']):
                        error('CONFIRMATORY_FAMILY_MISSING', r)
                    elif family['strategy'] == 'bonferroni_fixed_family':
                        confidence = r['analysis']['confidence_level']
                        if (family['alpha'] is None or confidence is None
                                or confidence < 1 - family['alpha'] / family['max_confirmatory_tests']):
                            error('FAMILY_ERROR_ALLOCATION_INVALID', r)
                    elif family['strategy'] == 'nonstatistical' and r['analysis']['design'] not in {'formal', 'deterministic'}:
                        error('NONSTATISTICAL_FAMILY_INVALID', r)
        if kind == 'experiment_result' and r['execution_validity'] == 'valid':
            plan = resolve(r['experiment_plan_ref'], 'experiment_plan')
            hyp = resolve(r['hypothesis_ref'], 'hypothesis')
            if not plan or not hyp:
                continue
            if plan['status'] not in {'ready', 'running', 'completed'} or plan['blockers']:
                error('RESULT_PLAN_NOT_EXECUTABLE', r)
            findings = [f['prediction_id'] for f in r['findings']]
            if (len(findings) != len(set(findings))
                    or not set(plan['analysis']['primary_endpoints']).issubset(findings)):
                error('RESULT_ENDPOINTS_MISSING_OR_DUPLICATED', r)
            if (len(r['control_results']) != len(plan['controls'])
                    or {c['id'] for c in r['control_results']} != set(plan['controls'])
                    or any(c['status'] != 'pass' for c in r['control_results'])):
                error('RESULT_CONTROLS_NOT_PASSED', r)
            if ({g['id'] for g in r['guardrail_results']} != set(plan['guardrail_ids'])
                    or (r['engineering_target'] == 'met'
                        and any(g['status'] != 'pass' for g in r['guardrail_results']))):
                error('RESULT_GUARDRAILS_NOT_PASSED', r)
            units = {p['id']: p['unit'] for p in hyp['predictions']}
            if any(f['unit'] != units.get(f['prediction_id']) for f in r['findings']):
                error('RESULT_UNIT_MISMATCH', r)
            artifacts(r['raw_artifact_hashes'] + ([r['analysis_artifact_sha256']] if r['analysis_artifact_sha256'] else []), r)
            receipt(r.get('execution_receipt_ref'), 'execution', plan, hyp, r, r['raw_artifact_hashes'])
            receipt(r.get('analysis_receipt_ref'), 'analysis', plan, hyp, r,
                    [r['analysis_artifact_sha256']] if r['analysis_artifact_sha256'] else [])
            if plan['analysis']['data_partition'] == 'confirmatory':
                accesses = [a for a in records if a['kind'] == 'holdout_access'
                            and a['plan_ref'] == ref_of(plan)
                            and trusted(ref_of(a), ctx.trusted_holdout_accesses)]
                if len(accesses) != 1 or accesses[0]['opened_at'] != r['data_opened_at']:
                    error('CONFIRMATORY_ACCESS_NOT_VERIFIED', r)
        if kind == 'dossier':
            hypotheses = [resolve(h, 'hypothesis') for h in r['hypothesis_refs']]
            results = [resolve(t, 'experiment_result') for t in r['result_refs']]
            if (any(h and h['mission_ref'] != r['mission_ref'] for h in hypotheses)
                    or any(t and t['hypothesis_ref'] not in r['hypothesis_refs'] for t in results)):
                error('DOSSIER_LINEAGE_MISMATCH', r)
            if r['status'] in {'validated_candidate', 'validated_solution'}:
                if r['data_origin'] != 'live' and not ctx.synthetic_fixture_mode:
                    error('SYNTHETIC_PROMOTION_FORBIDDEN', r)
                if r['unresolved_blockers']:
                    error('PROMOTION_BLOCKED', r)
                if r['novelty_report']['status'] in {'unresolved', 'conflicting'}:
                    error('NOVELTY_UNRESOLVED_FOR_PROMOTION', r)
                if r['status'] == 'validated_candidate' and (r['novelty_report']['status'] == 'known'
                        or not r['novelty_report']['claim_chart_sha256']):
                    error('INVENTION_DIFFERENCES_UNVERIFIED', r)
                if r['novelty_report']['claim_chart_sha256']:
                    artifacts([r['novelty_report']['claim_chart_sha256']], r)
                qualifying = []
                for hyp in hypotheses:
                    matches = [t for t in results if t and hyp and t['hypothesis_ref'] == ref_of(hyp)
                               and t['execution_validity'] == 'valid'
                               and t['scientific_conclusion'] == 'supported' and t['engineering_target'] == 'met']
                    if not matches:
                        error('PROMOTION_CLAIM_COVERAGE_MISSING', r)
                    qualifying.extend(matches)
                if not qualifying:
                    error('PROMOTION_EVIDENCE_MISSING', r)
                if r['reproduction_status'] != 'independent_pass' or not r['reproduction_artifact_sha256']:
                    error('REPRODUCTION_MISSING', r)
                # v1.1 reference profile deliberately qualifies one hypothesis/run.
                if len(hypotheses) != 1 or len(qualifying) != 1:
                    error('PROMOTION_PROFILE_UNSUPPORTED', r)
                else:
                    t = qualifying[0]
                    plan = resolve(t['experiment_plan_ref'], 'experiment_plan')
                    hyp = hypotheses[0]
                    if plan:
                        if plan['analysis']['data_partition'] != 'confirmatory' or plan['registration']['status'] != 'frozen':
                            error('PROMOTION_NOT_CONFIRMATORY', r)
                        required = [r['reproduction_artifact_sha256']] if r['reproduction_artifact_sha256'] else []
                        rep = receipt(r.get('reproduction_receipt_ref'), 'reproduction', plan, hyp, r, required)
                        receipt(r.get('promotion_receipt_ref'), 'promotion', plan, hyp, r, r['artifact_hashes'])
                        execution = resolve(t.get('execution_receipt_ref'), 'verification_receipt')
                        if rep and (not rep['independence_basis'] or not execution
                                    or rep['issuer_id'] == execution['issuer_id']):
                            error('REPRODUCTION_INDEPENDENCE_MISSING', r)
                        artifacts([plan['intervention_artifact_sha256'], plan['protected_evaluator_sha256'],
                                   plan['analysis']['implementation_sha256'], plan['evidence_snapshot_sha256']], r)
                # Traverse the immutable dependency graph, including provenance.
                seen = set()
                def walk(value):
                    if isinstance(value, list):
                        for item in value: walk(item)
                    elif isinstance(value, dict):
                        if set(value) == {'id', 'version'}:
                            k = key(value)
                            if k in seen: return
                            seen.add(k)
                            target = index.get(k)
                            if not target: return
                            if ctx.current_versions.get(target['id']) != target['record_version']:
                                error('PROMOTION_DEPENDENCY_STALE_OR_UNKNOWN', r)
                            try:
                                if ctx.current_record_hashes.get(k) != record_digest(target):
                                    error('PROMOTION_DEPENDENCY_CONTENT_CHANGED', r)
                            except ValueError:
                                error('PROMOTION_DEPENDENCY_CONTENT_CHANGED', r)
                            if target['kind'] == 'evidence' and (target['status'] != 'active' or target['quarantined']):
                                error('PROMOTION_EVIDENCE_RETRACTED_OR_QUARANTINED', r)
                            if target['data_origin'] != 'live' and not ctx.synthetic_fixture_mode:
                                error('SYNTHETIC_PROMOTION_FORBIDDEN', r)
                            walk(target)
                        else:
                            for item in value.values(): walk(item)
                walk(r)

    accesses = [r for r in records if r['kind'] == 'holdout_access']
    pairs = [(key(a['family_ref']), a['query_index']) for a in accesses]
    if len(pairs) != len(set(pairs)):
        errors.append('HOLDOUT_QUERY_DUPLICATED: bundle')
    families = [r for r in records if r['kind'] == 'experiment_family'
                and r['strategy'] != 'exploratory_only']
    partitions = [f['data_partition_sha256'] for f in families]
    if len(partitions) != len(set(partitions)):
        errors.append('HOLDOUT_FAMILY_RESET_FORBIDDEN: bundle')
    return errors
