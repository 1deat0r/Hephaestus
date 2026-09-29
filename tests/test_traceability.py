import copy
import json
from pathlib import Path
import unittest
from tools.traceability import validate_traceability

ROOT = Path(__file__).resolve().parents[1]

class TraceabilityChecks(unittest.TestCase):
    def setUp(self):
        self.reqs = json.loads((ROOT / 'requirements.json').read_text())['requirements']
        self.tests = json.loads((ROOT / 'acceptance-tests.json').read_text())['tests']
        self.scopes = json.loads((ROOT / 'release-scopes.json').read_text())

    def errors(self):
        return validate_traceability(self.reqs, self.tests, self.scopes, ROOT)

    def rejects(self, code):
        errors = self.errors()
        self.assertTrue(any(e.startswith(code) for e in errors), errors)

    def test_current_package_consistent(self):
        self.assertEqual(self.errors(), [])

    def test_reverse_requirement_mapping_required(self):
        self.tests[0]['requirement_ids'] = ['R-002']
        self.rejects('NONRECIPROCAL_ACCEPTANCE')

    def test_reverse_test_mapping_required(self):
        self.tests[0]['requirement_ids'].append('R-002')
        self.rejects('NONRECIPROCAL_REQUIREMENT')

    def test_clause_anchor_must_exist(self):
        self.reqs[0]['clause_anchor'] = 'docs/OBLIGATIONS.md#missing-clause'
        self.rejects('MISSING_CLAUSE_ANCHOR')

    def test_positive_and_negative_cases_required(self):
        for field in ('positive_case', 'negative_case'):
            with self.subTest(field=field):
                self.setUp()
                self.tests[0][field] = ''
                self.rejects('INCOMPLETE_ACCEPTANCE_CASE')

    def test_end_to_end_obligation_is_not_m0(self):
        self.assertEqual(next(r for r in self.reqs if r['id'] == 'R-080')['runtime_milestone'], 'M3')
        self.assertNotIn('AT-080', self.scopes['scopes'][0]['runtime_tests'])

    def test_release_cannot_omit_a_due_gate(self):
        self.scopes['scopes'][4]['runtime_requirements'].pop()
        self.rejects('RELEASE_SCOPE_COVERAGE_MISMATCH')

    def test_generated_text_drift_detected(self):
        self.reqs[0]['statement'] = 'Silently changed meaning.'
        self.rejects('GENERATED_DOCUMENT_DRIFT')
