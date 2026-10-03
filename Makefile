# Hephaestus dev commands (T-001).
# First run after clone:
#   make setup   # python3 -m venv .venv + pip install -r requirements-validation.txt
#   make hooks   # wire git hooks (git config core.hooksPath .githooks)
PY := .venv/bin/python
CARGO := cargo

.PHONY: ci ci-fast fmt fmt-check clippy test-rust test-py verify test-package gen-check conflict-staged conflict-tree manifest-check req-coverage about-check about-live about-sync scheduled-check md-links readme-fences report-unreferenced gate-seal seal hooks hooks-check setup clean doc-check

## Run every environment-independent gate (mirrors CI; the single gate registry).
ci: fmt-check clippy test-rust test-py conflict-tree manifest-check req-coverage about-check ticket-status md-links readme-fences report-unreferenced gate-seal ci-fast hooks-check

## Fast spec/doc freshness gates (.githooks/pre-commit runs exactly these).
## manifest-check joins the tier (ADR-025): a tracked-but-uncovered file
## now fails the commit locally instead of a push after CI.
ci-fast: gen-check verify test-package conflict-staged manifest-check

## Rustdoc warnings denied — the same command the remote docs job runs.
## Deliberately in NO tier: local latency budget stays untouched (ADR-024
## step 10); the remote docs job remains the automatic backstop (ADR-025).
doc-check:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

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

## Every requirement cited in code scope or reasoned in the allowlist (ADR-027).
req-coverage:
	$(PY) tools/check_requirement_citations.py

## Advisory AT-citation worklist (ADR-024 tier — NOT in `ci` yet; see ADR-027 lineage).
at-coverage:
	$(PY) tools/report_at_citations.py

## GitHub About (description/homepage/topics) vs README (ADR-029; offline).
about-check:
	$(PY) tools/check_repo_about.py

## Live GitHub About vs the canonical file (ADR-029; online, CI + periodic).
about-live:
	$(PY) tools/check_repo_about.py --live

## Push the canonical About to GitHub and stamp `verified` (ADR-029; owner auth).
about-sync:
	$(PY) tools/check_repo_about.py --sync

## Scheduled workflows still active (ADR-030; online — GitHub's 60-day rule).
scheduled-check:
	$(PY) tools/check_scheduled_workflows.py

## Ticket status structure (ADR-028): valid Status; open tickets declare Verify.
ticket-status:
	$(PY) tools/check_ticket_status.py --format

## Stale-label sweep (ADR-028): a green open ticket fails until its label flips.
ticket-status-sweep:
	$(PY) tools/check_ticket_status.py --sweep

## Offline internal markdown links/anchors (ADR-024 L3 slice; shrink-only baseline).
md-links:
	$(PY) tools/check_md_links.py

## README ```sh fences must parse against known make targets/tools (ADR-024 L3).
readme-fences:
	$(PY) tools/check_readme_fences.py

## Advisory: tracked files never named by markdown (ADR-024 — always exits 0).
report-unreferenced:
	$(PY) tools/report_unreferenced.py

## Verify the runtime gate seal (ADR-024 — gate files can't join MANIFEST).
gate-seal:
	$(PY) tools/gate_seal.py --check

## Regenerate the gate seal after an intentional gate change (cite ADR).
seal:
	$(PY) tools/gate_seal.py --write

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
