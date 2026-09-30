# Hephaestus dev commands (T-001).
# First run after clone:
#   make setup   # python3 -m venv .venv + pip install -r requirements-validation.txt
#   make hooks   # wire git hooks (git config core.hooksPath .githooks)
PY := .venv/bin/python
CARGO := cargo

.PHONY: ci ci-fast fmt fmt-check clippy test-rust test-py verify test-package gen-check conflict-staged conflict-tree manifest-check md-links readme-fences report-unreferenced hooks hooks-check setup clean

## Run every environment-independent gate (mirrors CI; the single gate registry).
ci: fmt-check clippy test-rust test-py conflict-tree manifest-check md-links readme-fences report-unreferenced ci-fast hooks-check

## Fast spec/doc freshness gates (.githooks/pre-commit runs exactly these).
ci-fast: gen-check verify test-package conflict-staged

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

## Staged conflict markers + whitespace (pre-commit index context; ADR-024).
## git grep rc: 0=markers found (fail), 1=clean, >=2=error — case explicitly.
conflict-staged:
	@git diff --cached --check
	@git grep --cached -nE '^<{7} ' >/dev/null 2>&1; rc=$$?; \
	case $$rc in \
	0) echo "conflict-staged: FAIL — conflict markers staged:" >&2; git grep --cached -nE '^<{7} ' >&2; exit 1 ;; \
	1) ;; \
	*) echo "conflict-staged: git grep error (rc=$$rc)" >&2; exit $$rc ;; \
	esac

## Committed-tree conflict markers (backs up ci-fast against --no-verify; ADR-024).
conflict-tree:
	@git grep -nE '^<{7} ' -- . >/dev/null 2>&1; rc=$$?; \
	case $$rc in \
	0) echo "conflict-tree: FAIL — conflict markers in tracked files:" >&2; git grep -nE '^<{7} ' -- . >&2; exit 1 ;; \
	1) ;; \
	*) echo "conflict-tree: git grep error (rc=$$rc)" >&2; exit $$rc ;; \
	esac

## Tracked files ⊆ MANIFEST ∪ allowlist, MANIFEST ⊆ tracked (ADR-024 L4).
manifest-check:
	$(PY) tools/check_manifest_coverage.py

## Offline internal markdown links/anchors (ADR-024 L3 slice; shrink-only baseline).
md-links:
	$(PY) tools/check_md_links.py

## README ```sh fences must parse against known make targets/tools (ADR-024 L3).
readme-fences:
	$(PY) tools/check_readme_fences.py

## Advisory: tracked files never named by markdown (ADR-024 — always exits 0).
report-unreferenced:
	$(PY) tools/report_unreferenced.py

## One-time per clone: wire the committed git hooks.
hooks:
	git config core.hooksPath .githooks

## Fail until `make hooks` has been run; self-skips under CI (no local config there).
hooks-check:
	@if [ -n "$$CI" ]; then echo "hooks-check: skipped (CI)"; \
	elif [ "$$(git config core.hooksPath)" = ".githooks" ]; then echo "hooks-check: ok"; \
	else echo "hooks-check: FAIL — run 'make hooks' to wire .githooks"; exit 1; fi

## One-time per clone: validation venv.
setup:
	python3 -m venv .venv && .venv/bin/pip install -r requirements-validation.txt

clean:
	$(CARGO) clean
	find python -name __pycache__ -type d -exec rm -rf {} + 2>/dev/null || true
