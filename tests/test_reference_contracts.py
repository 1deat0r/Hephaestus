"""Tests of reference metadata rules, not tests of an invention runtime."""
import copy
import json
from pathlib import Path
import unittest
from jsonschema import Draft202012Validator, FormatChecker
from reference.semantic_validator import validate_bundle

ROOT=Path(__file__).resolve().parents[1]
BASE=json.loads((ROOT/'examples/software-mission.json').read_text())
SCHEMA=json.loads((ROOT/'schemas/contracts.schema.json').read_text())

class ReferenceContracts(unittest.TestCase):
    def setUp(self):
        self.bundle=copy.deepcopy(BASE)
    def record(self,kind):
        return next(r for r in self.bundle['records'] if r['kind']==kind)
    def has(self,prefix):
        self.assertTrue(any(e.startswith(prefix) for e in validate_bundle(self.bundle)),validate_bundle(self.bundle))
    def test_example_schema_and_semantics(self):
        v=Draft202012Validator(SCHEMA,format_checker=FormatChecker())
        for record in self.bundle['records']: v.validate(record)
        self.assertEqual(validate_bundle(self.bundle),[])
    def test_duplicate_version_rejected(self):
        self.bundle['records'].append(copy.deepcopy(self.record('mission')))
        self.has('DUPLICATE_VERSION')
    def test_missing_reference_rejected(self):
        self.record('hypothesis')['mission_ref']['version']=99
        self.has('MISSING_REFERENCE')
    def test_test_ready_needs_discriminator(self):
        h=self.record('hypothesis'); h['state']='test_ready'; h['falsifiers']=[]
        self.has('NO_CREDIBLE_DISCRIMINATOR')
    def test_test_ready_cannot_be_blocked(self):
        self.record('hypothesis')['state']='test_ready'
        self.has('TESTABILITY_BLOCKED')
    def test_unknown_prediction_rejected(self):
        self.record('experiment_plan')['prediction_ids'].append('PRED-MISSING')
        self.has('UNKNOWN_PLAN_PREDICTION')
    def test_confirmatory_registration_required(self):
        p=self.record('experiment_plan'); p['status']='ready'; p['analysis']['data_partition']='confirmatory'
        self.has('CONFIRMATION_NOT_REGISTERED')
    def test_fixed_sample_requires_n(self):
        self.record('experiment_plan')['status']='ready'
        self.has('SAMPLE_SIZE_UNSET')
    def test_optional_stopping_requires_method(self):
        p=self.record('experiment_plan'); p['status']='ready'; p['analysis']['design']='sequential'
        self.has('UNAPPROVED_OPTIONAL_STOPPING')
    def test_incomplete_execution_cannot_support(self):
        self.record('experiment_result')['scientific_conclusion']='supported'
        self.has('INVALID_EXECUTION_CONCLUSION')
    def test_valid_result_requires_receipts(self):
        self.record('experiment_result')['execution_validity']='valid'
        self.has('RESULT_RECEIPTS_MISSING')
    def test_data_access_cannot_precede_registration(self):
        p=self.record('experiment_plan'); p['analysis']['data_partition']='confirmatory'; p['registration']['registered_at']='2026-09-30T05:15:00+13:00'
        self.record('experiment_result')['data_opened_at']='2026-09-30T04:15:00+13:00'
        self.has('DATA_BEFORE_REGISTRATION')
    def test_non_idempotent_blind_retry_rejected(self):
        self.record('task')['effect_class']='external_non_idempotent'
        self.has('UNSAFE_RETRY')
    def test_unknown_effect_not_dispatched(self):
        t=self.record('task'); t['effect_class']='unknown'; t['state']='ready'
        self.has('UNKNOWN_EFFECT_DISPATCH')
    def test_unapproved_mission_not_dispatched(self):
        self.record('task')['state']='ready'
        self.has('AUTHORIZATION_NOT_APPROVED')
    def test_capability_not_implicitly_granted(self):
        t=self.record('task'); t['state']='ready'; t['required_capabilities']=['public_disclosure']
        self.has('CAPABILITY_NOT_GRANTED')
    def test_task_cap_respects_mission(self):
        self.record('task')['budget']['minor_units']=999999
        self.has('TASK_CAP_EXCEEDS_MISSION')
    def test_task_cycle_rejected(self):
        self.record('task')['dependency_refs']=[{'id':'TASK-PLAN','version':1}]
        self.has('TASK_DAG_CYCLE')
    def test_model_decision_cannot_be_authoritative(self):
        self.record('decision')['authoritative']=True
        v=Draft202012Validator(SCHEMA,format_checker=FormatChecker())
        self.assertTrue(list(v.iter_errors(self.record('decision'))))
    def test_probability_vector_consistency(self):
        self.record('decision')['probabilities']=[0.9,0.9]
        self.has('INVALID_PROBABILITY_VECTOR')
    def test_abstention_has_no_selected_choice(self):
        self.record('decision')['selected_option']='ready'
        self.has('ABSTENTION_CONFLICT')
    def test_model_calibration_needs_receipt(self):
        self.record('decision')['calibration_status']='qualified_for_declared_domain'
        self.has('CALIBRATION_RECEIPT_MISSING')
    def test_promotion_needs_real_qualifying_result(self):
        self.record('dossier')['status']='validated_candidate'
        self.has('PROMOTION_EVIDENCE_MISSING')
    def test_promotion_requires_reproduction(self):
        self.record('dossier')['status']='validated_candidate'
        self.has('REPRODUCTION_MISSING')
    def test_promotion_does_not_ignore_novelty_scope(self):
        self.record('dossier')['status']='validated_candidate'
        self.has('NOVELTY_UNRESOLVED_FOR_PROMOTION')
    def test_nonfinite_data_rejected(self):
        self.record('hypothesis')['predictions'][0]['threshold']=float('nan')
        self.has('NONFINITE_NUMBER')
    def test_reversed_interval_rejected(self):
        self.record('experiment_result')['findings']=[{'prediction_id':'PRED-TIME','estimate':0.2,'unit':'fraction','lower_bound':0.4,'upper_bound':0.1,'interpretation':'Synthetic invalid bounds'}]
        self.has('REVERSED_INTERVAL')
    def test_compiled_blocked_example_is_not_deleted(self):
        h=self.record('hypothesis')
        self.assertEqual(h['state'],'compiled')
        self.assertEqual(h['testability']['status'],'blocked')
        self.assertEqual(validate_bundle(self.bundle),[])

if __name__=='__main__': unittest.main()
