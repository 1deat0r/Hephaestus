"""Adversarial metadata probes; no real execution or scientific claims."""
import copy
import json
import sys
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
import importlib.util
spec = importlib.util.spec_from_file_location(
    'review_v1_0_validator', ROOT / 'validation/v1.0/semantic_validator.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
validate_bundle = module.validate_bundle

base = json.loads((ROOT / 'examples/software-mission.v1.0.json').read_text())
validator = Draft202012Validator(
    json.loads((ROOT / 'schemas/contracts.v1.0.schema.json').read_text()),
    format_checker=FormatChecker(),
)

def record(bundle, kind):
    return next(r for r in bundle['records'] if r['kind'] == kind)

def assess(name, bundle):
    errors = [e.message for r in bundle['records'] for e in validator.iter_errors(r)]
    return {'probe': name, 'schema_errors': errors,
            'semantic_errors': validate_bundle(bundle) if not errors else None}

bundle = copy.deepcopy(base)
record(bundle, 'experiment_result').update(
    execution_validity='valid', scientific_conclusion='supported',
    engineering_target='met', raw_artifact_hashes=['a' * 64],
    analysis_artifact_sha256='b' * 64,
)
record(bundle, 'dossier').update(
    status='validated_candidate', reproduction_status='independent_pass',
    reproduction_artifact_sha256='c' * 64, unresolved_blockers=[],
)
record(bundle, 'dossier')['novelty_report']['status'] = 'no_match_within_search_scope'
results = [assess('promotion_from_blocked_unqualified_draft_plan', bundle)]

cross_mission = copy.deepcopy(bundle)
mission = copy.deepcopy(record(cross_mission, 'mission'))
mission['id'] = 'MIS-OTHER'
cross_mission['records'].append(mission)
record(cross_mission, 'dossier')['mission_ref']['id'] = 'MIS-OTHER'
results.append(assess('dossier_attached_to_unrelated_mission', cross_mission))

known = copy.deepcopy(bundle)
record(known, 'dossier')['novelty_report']['status'] = 'known'
results.append(assess('known_candidate_taxonomy', known))
print(json.dumps({'synthetic_adversarial_probes_only': True, 'results': results}, indent=2))
