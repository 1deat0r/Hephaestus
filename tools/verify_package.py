#!/usr/bin/env python3
"""Validate package traceability, schemas, and synthetic examples offline."""
from __future__ import annotations
import json
from pathlib import Path
import re
import sys
from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from reference.semantic_validator import validate_bundle
from tools.traceability import validate_traceability


def main() -> int:
    errors = []
    schema = json.loads((ROOT / 'schemas/contracts.schema.json').read_text())
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(schema, format_checker=FormatChecker())
    bundle = json.loads((ROOT / 'examples/software-mission.json').read_text())
    for record in bundle['records']:
        for error in validator.iter_errors(record):
            errors.append(f"SCHEMA: {record.get('id')}: {error.message}")
    if not errors:
        errors.extend(validate_bundle(bundle))
    requirements = json.loads((ROOT / 'requirements.json').read_text())['requirements']
    tests = json.loads((ROOT / 'acceptance-tests.json').read_text())['tests']
    scopes = json.loads((ROOT / 'release-scopes.json').read_text())
    errors.extend(validate_traceability(requirements, tests, scopes, ROOT))
    req_ids, test_ids = {r['id'] for r in requirements}, {t['id'] for t in tests}
    if len(req_ids) != len(requirements) or len(test_ids) != len(tests):
        errors.append('DUPLICATE_TRACEABILITY_ID')
    sections = {int(s) for s in re.findall(r'^# (\d+)\.', (ROOT/'MASTER_SPEC.md').read_text(), re.M)}
    for req in requirements:
        if not req['owner'] or not req['milestone'] or req['spec_section'] not in sections:
            errors.append(f"INCOMPLETE_REQUIREMENT: {req['id']}")
        if not req['acceptance_tests'] or not set(req['acceptance_tests']).issubset(test_ids):
            errors.append(f"MISSING_ACCEPTANCE: {req['id']}")
    for test in tests:
        if not set(test['requirement_ids']).issubset(req_ids):
            errors.append(f"ORPHAN_ACCEPTANCE: {test['id']}")
        if test['execution_status'] != 'NOT_RUN_RUNTIME_NOT_INCLUDED':
            errors.append(f"UNSUPPORTED_RUNTIME_TEST_STATUS: {test['id']}")
    master=(ROOT/'MASTER_SPEC.md').read_text()
    cited=set(re.findall(r'\[S(\d+)\]',master))
    defined=set(re.findall(r'\*\*\[S(\d+)\]',master))
    if cited-defined:
        errors.append(f'UNDEFINED_SOURCE_IDS: {sorted(cited-defined)}')
    # Fixture source hashes must match actual local bytes.
    import hashlib
    for record in bundle['records']:
        if record['kind']=='source' and record['access_scope']=='synthetic':
            source=ROOT/record['locator']
            if not source.is_file() or hashlib.sha256(source.read_bytes()).hexdigest()!=record['content_sha256']:
                errors.append(f"FIXTURE_HASH_MISMATCH: {record['id']}")
    summary={'status':'PASS' if not errors else 'FAIL','master_sections':len(sections),'requirements':len(requirements),'runtime_acceptance_specs':len(tests),'principal_record_schemas':len(schema['oneOf']),'schema_valid_example_records':len(bundle['records']) if not any(e.startswith('SCHEMA:') for e in errors) else 0,'semantic_validation_errors':len(errors),'errors':errors,'limitations':'Package/reference checks only. No invention runtime, security boundary, scientific study, or runtime acceptance test was executed.'}
    print(json.dumps(summary,indent=2))
    return bool(errors)

if __name__=='__main__':
    raise SystemExit(main())
