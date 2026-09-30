# Hephaestus dev commands (T-001).
# Python gates use .venv (see requirements-validation.txt); create it with:
#   python3 -m venv .venv && .venv/bin/pip install -r requirements-validation.txt
PY := .venv/bin/python
CARGO := cargo

.PHONY: ci fmt fmt-check clippy test-rust test-py verify test-package gen-check clean

## Run every environment-independent gate (mirrors CI).
ci: fmt-check clippy test-rust gen-check test-py verify test-package

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test-rust:
	$(CARGO) test --workspace

## Generated contracts must match the schema (T-002).
gen-check:
	python3 tools/generate_contracts_rs.py --check

## Worker-package unit tests (runtime tests; separate from spec-package checks).
test-py:
	PYTHONPATH=python $(PY) -m unittest discover -s python/tests -t python -v

## Spec-package contract checks (reference only — no runtime tested).
verify:
	$(PY) tools/verify_package.py

test-package:
	$(PY) -m unittest discover -s tests

clean:
	$(CARGO) clean
	find python -name __pycache__ -type d -exec rm -rf {} + 2>/dev/null || true
