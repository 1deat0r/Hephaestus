"""Self-improvement contract metadata; no actual learning/deployment occurs."""
import copy
from dataclasses import replace
import json
import unittest

from jsonschema import Draft202012Validator, FormatChecker
from reference.qualification import record_digest
from reference.semantic_validator import validate_bundle
from qualification_fixtures import ROOT, qualified_fixture

class SelfImprovementContracts(unittest.TestCase):
    def setUp(self):
        self.bundle, self.context = qualified_fixture()
        get = self.record
        improvement = get('improvement_candidate')
        plan, result, mission = [get(k) for k in ('experiment_plan', 'experiment_result', 'mission')]
        improvement.update(state='deployed', incumbent_sha256=plan['analysis']['implementation_sha256'],
                           candidate_sha256=plan['intervention_artifact_sha256'],
                           evaluation_manifest_sha256=plan['evidence_snapshot_sha256'],
                           evaluation_artifact_sha256=result['analysis_artifact_sha256'],
                           rollback_sha256=plan['analysis']['implementation_sha256'],
                           family_ref=copy.deepcopy(plan['family_ref']), benefit_status='supported',
                           guardrail_status='pass', evaluator_id='SYNTHETIC-PROTECTED-EVALUATOR')
        grant = copy.deepcopy(get('authorization_grant'))
        grant.update(id='GRANT-IMPROVEMENT', operation_id=improvement['operation_id'],
                     capabilities=['deploy_local_improvement'], artifact_sha256=improvement['candidate_sha256'])
        self.bundle['records'].append(grant)
        improvement['grant_ref'] = {'id': grant['id'], 'version': 1}
        mission['authorization']['allowed_capabilities'].append('deploy_local_improvement')
        hashes = dict(self.context.authenticated_record_hashes)
        hashes[(grant['id'], 1)] = record_digest(grant)
        hashes[(improvement['id'], 1)] = record_digest(improvement)
        self.context = replace(self.context,
            trusted_improvements=frozenset({(improvement['id'], 1)}),
            trusted_grants=self.context.trusted_grants | {(grant['id'], 1)},
            authenticated_record_hashes=hashes,
            current_record_hashes={(r['id'], r['record_version']): record_digest(r) for r in self.bundle['records']},
            current_versions={r['id']: r['record_version'] for r in self.bundle['records']},
            champion_by_scope={(mission['id'], improvement['target']): improvement['candidate_sha256']})

    def record(self, kind):
        return next(r for r in self.bundle['records'] if r['kind'] == kind)

    def errors(self):
        return validate_bundle(self.bundle, context=self.context)

    def rejects(self, prefix):
        errors = self.errors()
        self.assertTrue(any(e.startswith(prefix) for e in errors), errors)

    def test_synthetic_qualified_deployment_contract(self):
        schema = json.loads((ROOT / 'schemas/contracts.schema.json').read_text())
        validator = Draft202012Validator(schema, format_checker=FormatChecker())
        for record in self.bundle['records']:
            validator.validate(record)
        self.assertEqual(self.errors(), [])

    def test_self_attested_improvement_cannot_deploy(self):
        self.context = replace(self.context, trusted_improvements=frozenset())
        self.rejects('IMPROVEMENT_ASSESSMENT_NOT_TRUSTED')

    def test_inconclusive_improvement_cannot_deploy(self):
        self.record('improvement_candidate')['benefit_status'] = 'inconclusive'
        self.rejects('IMPROVEMENT_NOT_QUALIFIED')

    def test_guardrail_failure_cannot_deploy(self):
        self.record('improvement_candidate')['guardrail_status'] = 'fail'
        self.rejects('IMPROVEMENT_NOT_QUALIFIED')

    def test_changed_candidate_does_not_inherit_assessment(self):
        self.record('improvement_candidate')['candidate_sha256'] = 'a' * 64
        self.rejects('IMPROVEMENT_ASSESSMENT_NOT_TRUSTED')
        self.rejects('IMPROVEMENT_EVALUATION_BINDING_MISMATCH')

    def test_code_deployment_needs_separate_capability(self):
        self.record('improvement_candidate')['target'] = 'runtime_code'
        self.rejects('GRANT_BINDING_MISMATCH')

    def test_owner_controlled_policy_is_not_a_change_target(self):
        record = copy.deepcopy(self.record('improvement_candidate'))
        record['target'] = 'permissions'
        schema = json.loads((ROOT / 'schemas/contracts.schema.json').read_text())
        self.assertTrue(list(Draft202012Validator(schema).iter_errors(record)))

    def test_revoked_family_cannot_qualify_improvement(self):
        self.record('experiment_family')['retired'] = True
        self.rejects('IMPROVEMENT_FAMILY_NOT_QUALIFIED')

    def test_verified_incumbent_is_required_for_rollback(self):
        self.record('improvement_candidate')['rollback_sha256'] = 'a' * 64
        self.rejects('IMPROVEMENT_ROLLBACK_MISMATCH')

    def test_quarantined_learning_does_not_qualify(self):
        self.record('evidence')['quarantined'] = True
        self.rejects('IMPROVEMENT_EVIDENCE_INVALID')

    def test_roll_back_metadata_preserves_reason_and_incumbent(self):
        r = self.record('improvement_candidate')
        r.update(state='rolled_back', rollback_reason='Synthetic canary regression')
        hashes = dict(self.context.authenticated_record_hashes)
        hashes[(r['id'], r['record_version'])] = record_digest(r)
        self.context = replace(self.context, authenticated_record_hashes=hashes,
                               champion_by_scope={(r['mission_ref']['id'], r['target']): r['incumbent_sha256']})
        self.assertEqual(self.errors(), [])
        r['rollback_reason'] = None
        self.rejects('IMPROVEMENT_ROLLBACK_REASON_MISSING')

    def test_mandatory_m3_scope(self):
        scopes = json.loads((ROOT / 'release-scopes.json').read_text())['scopes']
        m3 = next(s for s in scopes if s['id'] == 'M3')
        for number in (70, 71, 72, 115, 116, 117, 118, 119):
            self.assertIn(f'R-{number:03}', m3['runtime_requirements'])
            self.assertIn(f'AT-{number:03}', m3['runtime_tests'])
