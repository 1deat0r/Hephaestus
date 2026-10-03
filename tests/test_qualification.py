"""Adversarial and positive qualification contracts; no science was performed."""
import copy
from dataclasses import replace
from datetime import datetime, timezone
import json
import unittest

from jsonschema import Draft202012Validator, FormatChecker
from reference.semantic_validator import validate_bundle
from reference.qualification import ValidationContext, plan_digest, record_digest
from qualification_fixtures import ROOT, qualified_fixture, resign_plan

class QualificationContracts(unittest.TestCase):
    def setUp(self):
        self.bundle, self.context = qualified_fixture()

    def record(self, kind):
        return next(r for r in self.bundle['records'] if r['kind'] == kind)

    def errors(self):
        return validate_bundle(self.bundle, context=self.context)

    def rejects(self, code):
        errors = self.errors()
        self.assertTrue(any(e.startswith(code) for e in errors), errors)

    def authenticate_test_record(self, record):
        """Simulate a protected issuer so malformed signed metadata is tested too."""
        hashes = dict(self.context.authenticated_record_hashes)
        hashes[(record['id'], record['record_version'])] = record_digest(record)
        self.context = replace(self.context, authenticated_record_hashes=hashes)

    def test_positive_synthetic_qualification_profile(self):
        schema = json.loads((ROOT / 'schemas/contracts.schema.json').read_text())
        validator = Draft202012Validator(schema, format_checker=FormatChecker())
        for record in self.bundle['records']:
            validator.validate(record)
        self.assertEqual(self.errors(), [])

    def test_archived_schema_does_not_silently_migrate_approval(self):
        legacy = json.loads((ROOT / 'examples/software-mission.v1.0.json').read_text())
        old = Draft202012Validator(json.loads((ROOT / 'schemas/contracts.v1.0.schema.json').read_text()),
                                  format_checker=FormatChecker())
        current = Draft202012Validator(json.loads((ROOT / 'schemas/contracts.schema.json').read_text()),
                                      format_checker=FormatChecker())
        for record in legacy['records']:
            old.validate(record)
            self.assertTrue(list(current.iter_errors(record)))

    def test_no_trust_can_be_claimed_by_bundle(self):
        errors = validate_bundle(self.bundle)
        self.assertTrue(any(e.startswith('RECEIPT_NOT_TRUSTED') for e in errors))
        self.assertTrue(any(e.startswith('SYNTHETIC_PROMOTION_FORBIDDEN') for e in errors))

    def test_review_promotion_probe_now_rejected(self):
        self.record('experiment_plan').update(status='draft', blockers=['Unimplemented'])
        self.record('experiment_plan')['analysis']['method_qualification'] = 'not_qualified'
        self.record('experiment_result')['findings'] = []
        self.rejects('RESULT_PLAN_NOT_EXECUTABLE')
        self.rejects('RESULT_ENDPOINTS_MISSING_OR_DUPLICATED')

    def test_cross_mission_dossier_rejected(self):
        other = copy.deepcopy(self.record('mission'))
        other['id'] = 'MIS-OTHER'
        self.bundle['records'].append(other)
        self.record('dossier')['mission_ref'] = {'id': 'MIS-OTHER', 'version': 1}
        self.rejects('DOSSIER_LINEAGE_MISMATCH')

    def test_result_cannot_cover_unlisted_hypothesis(self):
        self.record('dossier')['hypothesis_refs'] = []
        self.rejects('DOSSIER_LINEAGE_MISMATCH')

    def test_changed_hypothesis_does_not_inherit_receipt(self):
        self.record('hypothesis')['record_version'] = 2
        self.record('experiment_plan')['hypothesis_ref']['version'] = 2
        self.record('experiment_result')['hypothesis_ref']['version'] = 2
        self.record('dossier')['hypothesis_refs'][0]['version'] = 2
        self.rejects('RECEIPT_BINDING_MISMATCH')

    def test_all_primary_endpoints_required(self):
        self.record('experiment_result')['findings'].pop()
        self.rejects('RESULT_ENDPOINTS_MISSING_OR_DUPLICATED')

    def test_duplicate_endpoint_not_evidence(self):
        self.record('experiment_result')['findings'].append(copy.deepcopy(self.record('experiment_result')['findings'][0]))
        self.rejects('RESULT_ENDPOINTS_MISSING_OR_DUPLICATED')

    def test_unit_mismatch_rejected(self):
        self.record('experiment_result')['findings'][0]['unit'] = 'seconds'
        self.rejects('RESULT_UNIT_MISMATCH')

    def test_failed_control_blocks_valid_execution(self):
        self.record('experiment_result')['control_results'][0]['status'] = 'fail'
        self.rejects('RESULT_CONTROLS_NOT_PASSED')

    def test_guardrail_failure_blocks_promotion(self):
        self.record('experiment_result')['guardrail_results'][0]['status'] = 'fail'
        self.rejects('RESULT_GUARDRAILS_NOT_PASSED')

    def test_arbitrary_digest_not_verified_artifact(self):
        self.record('experiment_result')['raw_artifact_hashes'] = ['a' * 64]
        self.rejects('ARTIFACT_NOT_VERIFIED')
        self.rejects('RECEIPT_ARTIFACT_MISMATCH')

    def test_untrusted_qualification_enum_insufficient(self):
        self.context = replace(self.context, trusted_qualifications=frozenset())
        self.rejects('METHOD_NOT_TRUSTED')

    def test_revoked_or_changed_method_rejected(self):
        self.record('method_qualification')['state'] = 'revoked'
        self.authenticate_test_record(self.record('method_qualification'))
        self.rejects('METHOD_BINDING_MISMATCH')

    def test_qualification_domain_rejected(self):
        self.record('method_qualification')['domain'] = 'physical science'
        self.authenticate_test_record(self.record('method_qualification'))
        self.rejects('METHOD_DOMAIN_MISMATCH')

    def test_unqualified_assumption_rejected(self):
        self.record('experiment_plan')['analysis']['assumptions'].append('unknown independence')
        self.rejects('METHOD_BINDING_MISMATCH')

    def test_changed_registered_payload_rejected(self):
        # R-037/AT-037: endpoints and analysis are registered immutably
        # before confirmation; a change to the frozen payload is
        # rejected by digest mismatch.
        self.record('experiment_plan')['comparator'] = 'Weakened baseline'
        self.rejects('REGISTRATION_DIGEST_MISMATCH')

    def test_registration_digest_independent_of_key_order(self):
        plan = self.record('experiment_plan')
        self.assertEqual(plan_digest(plan), plan_digest(dict(reversed(list(plan.items())))))

    def test_operation_status_does_not_change_registered_payload(self):
        plan = self.record('experiment_plan')
        old = plan_digest(plan)
        plan['status'] = 'running'
        self.assertEqual(plan_digest(plan), old)

    def test_shared_canonicalization_vectors(self):
        from reference.qualification import subject_digest
        vectors = json.loads((ROOT / 'examples/canonicalization-vectors.json').read_text())['vectors']
        for vector in vectors:
            with self.subTest(name=vector['name']):
                digest = plan_digest if vector['kind'] == 'plan' else subject_digest
                self.assertEqual(digest(vector['payload']), vector['expected_sha256'])

    def test_holdout_access_must_be_authenticated(self):
        self.context = replace(self.context, trusted_holdout_accesses=frozenset())
        self.rejects('CONFIRMATORY_ACCESS_NOT_VERIFIED')

    def test_holdout_access_cannot_precede_registration(self):
        self.record('holdout_access')['opened_at'] = '2026-09-29T15:00:00+00:00'
        self.rejects('HOLDOUT_BEFORE_REGISTRATION')

    def test_holdout_feedback_leak_rejected(self):
        self.record('holdout_access')['feedback'] = 'detailed'
        self.rejects('HOLDOUT_REUSE_FORBIDDEN')

    def test_holdout_query_limit_rejected(self):
        self.record('holdout_access')['query_index'] = 3
        self.rejects('HOLDOUT_REUSE_FORBIDDEN')

    def test_duplicate_holdout_query_rejected(self):
        other = copy.deepcopy(self.record('holdout_access'))
        other['id'] = 'ACCESS-DUPLICATE'
        self.bundle['records'].append(other)
        self.rejects('HOLDOUT_QUERY_DUPLICATED')

    def test_renamed_family_cannot_reset_same_partition(self):
        other = copy.deepcopy(self.record('experiment_family'))
        other.update(id='FAMILY-RESET', plan_refs=[])
        self.bundle['records'].append(other)
        self.rejects('HOLDOUT_FAMILY_RESET_FORBIDDEN')

    def test_family_counts_endpoint_tests(self):
        self.record('experiment_family')['max_confirmatory_tests'] = 1
        self.rejects('FAMILY_TEST_LIMIT_EXCEEDED')

    def test_family_alpha_allocation_enforced(self):
        self.record('experiment_plan')['analysis']['confidence_level'] = 0.95
        self.rejects('FAMILY_ERROR_ALLOCATION_INVALID')

    def test_retracted_and_quarantined_evidence_blocks_current_label(self):
        for field, value in [('status', 'retracted'), ('quarantined', True)]:
            with self.subTest(field=field):
                self.bundle, self.context = qualified_fixture()
                self.record('evidence')[field] = value
                self.rejects('PROMOTION_EVIDENCE_RETRACTED_OR_QUARANTINED')

    def test_changed_dependency_requires_applicability(self):
        versions = dict(self.context.current_versions)
        versions[self.record('hypothesis')['id']] = 2
        self.context = replace(self.context, current_versions=versions)
        self.rejects('PROMOTION_DEPENDENCY_STALE_OR_UNKNOWN')

    def test_same_version_cannot_hide_dependency_mutation(self):
        self.record('mission')['objective'] = 'Silently changed goal'
        self.rejects('PROMOTION_DEPENDENCY_CONTENT_CHANGED')

    def test_missing_reproduction_receipt_rejected(self):
        self.record('dossier')['reproduction_receipt_ref'] = None
        self.rejects('RECEIPT_NOT_TRUSTED')

    def test_same_reproduction_issuer_is_insufficient(self):
        receipts = {r['purpose']: r for r in self.bundle['records'] if r['kind'] == 'verification_receipt'}
        receipts['reproduction']['issuer_id'] = receipts['execution']['issuer_id']
        self.authenticate_test_record(receipts['reproduction'])
        self.rejects('REPRODUCTION_INDEPENDENCE_MISSING')

    def test_receipt_candidate_evaluator_snapshot_policy_bindings(self):
        for field in ('candidate_sha256', 'evaluator_sha256', 'evidence_snapshot_sha256', 'policy_version'):
            with self.subTest(field=field):
                self.bundle, self.context = qualified_fixture()
                self.record('verification_receipt')['bindings'][field] = 'changed' if field == 'policy_version' else 'a' * 64
                self.authenticate_test_record(self.record('verification_receipt'))
                self.rejects('RECEIPT_BINDING_MISMATCH')

    def test_grant_revocation_expiry_and_binding_changes(self):
        for field, value, code in [
            ('state', 'revoked', 'GRANT_BINDING_MISMATCH'),
            ('operation_id', 'OP-OTHER', 'GRANT_BINDING_MISMATCH'),
            ('destination', 'https://unauthorized.invalid', 'GRANT_BINDING_MISMATCH'),
            ('artifact_sha256', 'a' * 64, 'GRANT_BINDING_MISMATCH'),
            ('policy_version', 'changed', 'GRANT_BINDING_MISMATCH'),
            ('expires_at', '2026-09-29T01:00:00+00:00', 'GRANT_EXPIRED_OR_TIME_UNKNOWN'),
        ]:
            with self.subTest(field=field):
                self.bundle, self.context = qualified_fixture()
                self.record('authorization_grant')[field] = value
                self.authenticate_test_record(self.record('authorization_grant'))
                self.rejects(code)

    def test_unknown_dispatch_time_rejected(self):
        self.context = replace(self.context, evaluated_at=None)
        self.rejects('GRANT_EXPIRED_OR_TIME_UNKNOWN')

    def test_changed_current_policy_rejected(self):
        self.context = replace(self.context, current_policies={self.record('mission')['id']: 'new-policy'})
        self.rejects('GRANT_BINDING_MISMATCH')

    def test_known_solution_has_same_scientific_status(self):
        result = copy.deepcopy(self.record('experiment_result'))
        self.bundle, self.context = qualified_fixture(known_solution=True)
        self.assertEqual(self.errors(), [])
        self.assertEqual(self.record('experiment_result'), result)

    def test_receipt_identity_cannot_hide_content_tampering(self):
        self.record('verification_receipt')['outcome'] = 'pass'
        self.record('verification_receipt')['issuer_id'] = 'FORGED-ISSUER'
        self.rejects('RECEIPT_NOT_TRUSTED')

    def test_changed_result_requires_new_analysis_receipt(self):
        self.record('experiment_result')['findings'][0]['estimate'] = 0.99
        self.rejects('RECEIPT_SUBJECT_MISMATCH')

    def test_valid_negative_guardrail_outcome_is_preserved(self):
        result = self.record('experiment_result')
        result['engineering_target'] = 'not_met'
        result['guardrail_results'][0]['status'] = 'fail'
        dossier = self.record('dossier')
        dossier['status'] = 'completed_investigation'
        from reference.qualification import subject_digest
        for r in self.bundle['records']:
            if r['kind'] == 'verification_receipt' and r['purpose'] in {'execution', 'analysis'}:
                r['subject_payload_sha256'] = subject_digest(result)
                self.authenticate_test_record(r)
        self.assertEqual(self.errors(), [])

    def test_known_invention_label_rejected(self):
        self.record('dossier')['novelty_report']['status'] = 'known'
        self.rejects('INVENTION_DIFFERENCES_UNVERIFIED')

    def test_invention_requires_verified_claim_chart(self):
        self.record('dossier')['novelty_report']['claim_chart_sha256'] = None
        self.rejects('INVENTION_DIFFERENCES_UNVERIFIED')

    def test_formal_and_deterministic_do_not_require_statistical_n(self):
        for design in ('formal', 'deterministic'):
            with self.subTest(design=design):
                self.bundle, self.context = qualified_fixture(design)
                self.assertIsNone(self.record('experiment_plan')['analysis']['target_n'])
                self.assertIsNone(self.record('experiment_plan')['analysis']['confidence_level'])
                self.assertIsNone(self.record('experiment_family')['alpha'])
                self.assertEqual(self.errors(), [])

if __name__ == '__main__':
    unittest.main()
