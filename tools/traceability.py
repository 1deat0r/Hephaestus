"""Check explicit reviewed mappings, not completeness of scientific meaning."""
import re
from tools.render_contract_docs import render_documents

def validate_traceability(requirements, tests, scopes, root, *, check_rendered=True):
    errors = []
    reqs = {r['id']: r for r in requirements}
    cases = {t['id']: t for t in tests}
    if len(reqs) != len(requirements) or len(cases) != len(tests):
        errors.append('DUPLICATE_TRACEABILITY_ID')
    for r in requirements:
        if (not r.get('acceptance_tests') or r.get('milestone') != r.get('runtime_milestone')
                or not all(r.get(k) for k in ('owner', 'enforcement_service', 'clause_anchor', 'source_anchor'))
                or r.get('contract_milestone') != 'M0'
                or not re.fullmatch(r'M[0-6]', r.get('runtime_milestone', ''))):
            errors.append('INCOMPLETE_CLAUSE: ' + r['id'])
        for tid in r.get('acceptance_tests', []):
            if tid not in cases or r['id'] not in cases[tid]['requirement_ids']:
                errors.append('NONRECIPROCAL_ACCEPTANCE: ' + r['id'])
        for field in ('clause_anchor', 'source_anchor'):
            target = r.get(field, '')
            path, _, anchor = target.partition('#')
            file = root / path
            if not anchor or not file.is_file() or f'id="{anchor}"' not in file.read_text():
                errors.append('MISSING_CLAUSE_ANCHOR: ' + r['id'])
        if r.get('supplement') and not (root / r['supplement']).is_file():
            errors.append('MISSING_SUPPLEMENT: ' + r['id'])
    for t in tests:
        if not t.get('requirement_ids') or not all(t.get(k) for k in
                ('setup', 'action', 'expected', 'positive_case', 'negative_case')):
            errors.append('INCOMPLETE_ACCEPTANCE_CASE: ' + t['id'])
        for rid in t.get('requirement_ids', []):
            if rid not in reqs or t['id'] not in reqs[rid]['acceptance_tests']:
                errors.append('NONRECIPROCAL_REQUIREMENT: ' + t['id'])
    entries = scopes.get('scopes', [])
    if {s['id'] for s in entries} != {f'M{m}' for m in range(7)} or len(entries) != 7:
        errors.append('RELEASE_SCOPE_SET_INCOMPLETE')
    for scope in entries:
        sid = scope['id']
        if not re.fullmatch(r'M[0-6]', sid):
            errors.append('RELEASE_SCOPE_INVALID: ' + sid)
            continue
        due = {r['id'] for r in requirements if r.get('runtime_milestone', 'M9') <= sid}
        expected_tests = {tid for rid in due for tid in reqs[rid]['acceptance_tests']}
        if (set(scope['contract_requirements']) != set(reqs)
                or set(scope['runtime_requirements']) != due
                or set(scope['runtime_tests']) != expected_tests):
            errors.append('RELEASE_SCOPE_COVERAGE_MISMATCH: ' + sid)
    if check_rendered and not errors:
        for path, text in render_documents(requirements, tests).items():
            if not (root / path).is_file() or (root / path).read_text() != text:
                errors.append('GENERATED_DOCUMENT_DRIFT: ' + path)
    return errors
