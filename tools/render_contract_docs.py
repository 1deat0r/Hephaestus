#!/usr/bin/env python3
"""Render reviewed normative mappings; no runtime status is inferred."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def render_documents(requirements, tests):
    lines = ['# Normative obligation matrix', '', 'Version 1.2 · 30 September 2026', '',
             'Each stable obligation below is normative and maps to its source section, enforcement owner and positive/negative runtime acceptance cases. Contract readiness is M0; runtime behavior is due at the stated milestone. This matrix is not evidence of implementation, scientific adequacy or exhaustive prose interpretation. Supplementary clauses refine the cited sources; unresolved conflicts block affected work.', '']
    trace = ['# Requirements traceability', '', 'Version 1.2 · 30 September 2026', '',
             'All entries are future obligations. Contract readiness and runtime acceptance are distinct. Exact clauses and cases are in `docs/OBLIGATIONS.md`.', '',
             '| Requirement | Section | Owner | Contract | Runtime | Acceptance |',
             '|---|---:|---|---|---|---|']
    bytest = {t['id']: t for t in tests}
    for r in requirements:
        lines += ['<a id="' + r['id'].lower() + '"></a>', '', '## ' + r['id'], '',
                  r['statement'], '', '**Source:** [' + r['source_anchor'] + '](../' + r['source_anchor'] + ').'
                  + (' Supplement: [' + r['supplement'] + '](../' + r['supplement'] + ').' if 'supplement' in r else ''), '',
                  '**Enforcement:** ' + r['enforcement_service'] + '. **Contract:** ' + r['contract_milestone']
                  + '. **Runtime:** ' + r['runtime_milestone'] + '. **Acceptance:** ' + ', '.join(r['acceptance_tests']) + '.', '']
        for tid in r['acceptance_tests']:
            t = bytest[tid]
            lines += ['**Positive case:** ' + t['positive_case'], '', '**Negative case:** ' + t['negative_case'], '',
                      '**Required outcome:** ' + t['expected'], '']
        trace += [f"| {r['id']} | {r['spec_section']} | {r['owner']} | {r['contract_milestone']} | {r['runtime_milestone']} | {', '.join(r['acceptance_tests'])} |"]
    acceptance = ['# Acceptance tests', '', 'Version 1.2 · 30 September 2026', '',
                  f'These {len(tests)} runtime acceptance specifications have NOT RUN: the application is not included. A pass requires code/environment/input identities, commands, raw outputs and protected verification receipts. Reference metadata checks are reported separately.', '']
    for t in tests:
        acceptance += ['## ' + t['id'] + ' · ' + ', '.join(t['requirement_ids']), '', t['title'], '',
                       '**Setup:** ' + t['setup'], '', '**Positive case:** ' + t['positive_case'], '',
                       '**Negative case:** ' + t['negative_case'], '', '**Action:** ' + t['action'], '',
                       '**Required outcome:** ' + t['expected'], '', '**Status:** NOT RUN — runtime not included.', '']
    return {'docs/OBLIGATIONS.md': '\n'.join(lines) + '\n',
            'TRACEABILITY.md': '\n'.join(trace) + '\n',
            'ACCEPTANCE_TESTS.md': '\n'.join(acceptance) + '\n'}

if __name__ == '__main__':
    requirements = json.loads((ROOT / 'requirements.json').read_text())['requirements']
    tests = json.loads((ROOT / 'acceptance-tests.json').read_text())['tests']
    for name, text in render_documents(requirements, tests).items():
        (ROOT / name).write_text(text)
