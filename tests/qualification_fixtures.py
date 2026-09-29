"""Synthetic trusted-context fixtures, never runtime receipts or approvals."""
import copy
from dataclasses import replace
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path

from reference.qualification import ValidationContext, plan_digest, record_digest, subject_digest

ROOT = Path(__file__).resolve().parents[1]

def qualified_fixture(design='fixed_sample', *, known_solution=False):
    bundle = json.loads((ROOT / 'examples/software-mission.json').read_text())
    get = lambda k: next(r for r in bundle['records'] if r['kind'] == k)
    artifacts = {}
    def artifact(label):
        data = ('SYNTHETIC TEST BYTES, NOT A RESULT: ' + label).encode()
        digest = hashlib.sha256(data).hexdigest()
        artifacts[digest] = data
        return digest
    def ref(r):
        return {'id': r['id'], 'version': r['record_version']}
    def new(kind, identifier, **fields):
        base = get('mission')
        common = {k: copy.deepcopy(base[k]) for k in
                  ('schema_version', 'record_version', 'created_at', 'data_origin', 'provenance')}
        record = dict(common, kind=kind, id=identifier, **copy.deepcopy(fields))
        bundle['records'].append(record)
        return record
    mission, hyp, plan, result, dossier = [get(k) for k in
        ('mission', 'hypothesis', 'experiment_plan', 'experiment_result', 'dossier')]
    mission['authorization']['state'] = 'approved'
    mission['authorization']['allowed_capabilities'] = list(plan['required_capabilities'])
    plan.update(status='completed', blockers=[], intervention_artifact_sha256=artifact('candidate'),
                protected_evaluator_sha256=artifact('evaluator'), evidence_snapshot_sha256=artifact('snapshot'))
    plan['analysis'].update(design=design, method_id='SYNTHETIC-QUALIFIED-METHOD',
                            method_qualification='qualified_for_declared_assumptions',
                            confidence_level=0.975 if design in {'fixed_sample', 'sequential'} else None,
                            target_n=40 if design == 'fixed_sample' else None,
                            data_partition='confirmatory', implementation_sha256=artifact('analysis implementation'),
                            assumptions=['independent repositories'],
                            anytime_valid_method='SYNTHETIC-SEQUENTIAL' if design == 'sequential' else None)
    method = new('method_qualification', 'QUAL-TEST', method_id=plan['analysis']['method_id'],
                 designs=[design], assumptions=plan['analysis']['assumptions'],
                 implementation_sha256=plan['analysis']['implementation_sha256'],
                 qualification_artifact_sha256=artifact('method qualification'), domain='software systems',
                 state='qualified', issuer_id='SYNTHETIC-METHOD-REGISTRY')
    grant = new('authorization_grant', 'GRANT-TEST', mission_ref=ref(mission),
                operation_id=plan['operation_id'], capabilities=plan['required_capabilities'],
                destination=None, artifact_sha256=plan['intervention_artifact_sha256'],
                policy_version=mission['authorization']['policy_version'],
                issued_at='2026-09-29T00:00:00+00:00', expires_at='2026-10-01T00:00:00+00:00',
                state='approved', max_cost=copy.deepcopy(mission['budget']), issuer_id='SYNTHETIC-OWNER')
    family = new('experiment_family', 'FAMILY-TEST', mission_ref=ref(mission), plan_refs=[ref(plan)],
                 selection_history_refs=[ref(hyp)], data_partition_sha256=artifact('fresh workloads'),
                 strategy='bonferroni_fixed_family' if design in {'fixed_sample', 'sequential'} else 'nonstatistical',
                 alpha=0.05 if design in {'fixed_sample', 'sequential'} else None,
                 max_confirmatory_tests=len(plan['analysis']['primary_endpoints']), retired=False)
    plan.update(qualification_ref=ref(method), grant_ref=ref(grant), family_ref=ref(family))
    plan['registration'].update(status='frozen', registered_at='2026-09-29T16:00:00+00:00')
    plan['registration']['payload_sha256'] = plan_digest(plan)
    access = new('holdout_access', 'ACCESS-TEST', family_ref=ref(family), plan_ref=ref(plan),
                 opened_at='2026-09-29T17:00:00+00:00', query_index=1,
                 feedback='sealed_until_campaign_end', issuer_id='SYNTHETIC-DATA-BROKER')
    result.update(execution_validity='valid', scientific_conclusion='supported', engineering_target='met',
                  data_opened_at=access['opened_at'], raw_artifact_hashes=[artifact('raw measurements')],
                  analysis_artifact_sha256=artifact('derived analysis'),
                  findings=[dict(prediction_id=p['id'], estimate=0.3, unit=p['unit'],
                                 lower_bound=0.25, upper_bound=0.35,
                                 interpretation='Synthetic metadata only') for p in hyp['predictions']],
                  control_results=[{'id': c, 'status': 'pass'} for c in plan['controls']],
                  guardrail_results=[{'id': g, 'status': 'pass'} for g in plan['guardrail_ids']])
    bindings = dict(mission_ref=ref(mission), hypothesis_ref=ref(hyp), experiment_plan_ref=ref(plan),
                    candidate_sha256=plan['intervention_artifact_sha256'],
                    evaluator_sha256=plan['protected_evaluator_sha256'],
                    analysis_implementation_sha256=plan['analysis']['implementation_sha256'],
                    evidence_snapshot_sha256=plan['evidence_snapshot_sha256'],
                    policy_version=mission['authorization']['policy_version'],
                    registration_sha256=plan['registration']['payload_sha256'])
    def receipt(purpose, hashes, issuer, independence=None):
        subject = result if purpose in {'execution', 'analysis'} else dossier
        return new('verification_receipt', 'RECEIPT-' + purpose.upper(), purpose=purpose,
                   bindings=copy.deepcopy(bindings), endpoint_ids=plan['analysis']['primary_endpoints'],
                   guardrail_ids=plan['guardrail_ids'], artifact_hashes=hashes, outcome='pass',
                   independence_basis=independence, issuer_id=issuer,
                   subject_ref=ref(subject), subject_payload_sha256=subject_digest(subject))
    execution = receipt('execution', result['raw_artifact_hashes'], 'SYNTHETIC-RUNNER')
    analysis = receipt('analysis', [result['analysis_artifact_sha256']], 'SYNTHETIC-ANALYZER')
    result.update(execution_receipt_ref=ref(execution), analysis_receipt_ref=ref(analysis))
    dossier.update(status='validated_candidate', unresolved_blockers=[], reproduction_status='independent_pass',
                   reproduction_artifact_sha256=artifact('reproduction'), artifact_hashes=list(artifacts))
    dossier['novelty_report'].update(status='near_match', differences='Synthetic assessed difference',
                                   claim_chart_sha256=artifact('claim chart'))
    if known_solution:
        dossier['status'] = 'validated_solution'
        dossier['novelty_report']['status'] = 'known'
    reproduction = receipt('reproduction', [dossier['reproduction_artifact_sha256']],
                           'SYNTHETIC-REPRODUCER', 'Synthetic independent pipeline; not real evidence')
    promotion = receipt('promotion', dossier['artifact_hashes'], 'SYNTHETIC-PROMOTER')
    dossier.update(reproduction_receipt_ref=ref(reproduction), promotion_receipt_ref=ref(promotion))
    context = ValidationContext(
        trusted_receipts=frozenset((r['id'], 1) for r in [execution, analysis, reproduction, promotion]),
        trusted_qualifications=frozenset({(method['id'], 1)}), trusted_grants=frozenset({(grant['id'], 1)}),
        trusted_holdout_accesses=frozenset({(access['id'], 1)}), verified_artifacts=frozenset(artifacts),
        trusted_families=frozenset({(family['id'], 1)}),
        authenticated_record_hashes={(r['id'], r['record_version']): record_digest(r)
                                    for r in bundle['records'] if r['kind'] in {
                                        'verification_receipt', 'method_qualification',
                                        'authorization_grant', 'holdout_access', 'experiment_family'}},
        current_versions={r['id']: r['record_version'] for r in bundle['records']},
        current_record_hashes={(r['id'], r['record_version']): record_digest(r) for r in bundle['records']},
        current_policies={mission['id']: mission['authorization']['policy_version']},
        evaluated_at=datetime(2026, 9, 30, tzinfo=timezone.utc), synthetic_fixture_mode=True,
    )
    return bundle, context

def resign_plan(bundle):
    plan = next(r for r in bundle['records'] if r['kind'] == 'experiment_plan')
    plan['registration']['payload_sha256'] = plan_digest(plan)
