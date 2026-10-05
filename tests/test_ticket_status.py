#!/usr/bin/env python3
"""Behavioral tests for the ticket-sweep vacuity classifier.

ADR-031: a Verify that exits 0 without running anything is not green.
ADR-034: a composite Verify that exits 0 while one of its sub-suites
prints empty counts did run tests, so it is not vacuous either.

The tests drive sweep_mode() over a temporary ticket tree, so they
exercise the gate entry point, not a private helper copy of its logic.

Receipts: .scratch/ticket-sweep-honesty/receipts/
"""
import contextlib
import importlib.util
import io
import tempfile
import unittest
from pathlib import Path

_TOOL = Path(__file__).resolve().parent.parent / "tools" / "check_ticket_status.py"
_LOADER = importlib.util.spec_from_file_location("check_ticket_status", _TOOL)
gate = importlib.util.module_from_spec(_LOADER)
_LOADER.loader.exec_module(gate)

# What `make ci` prints: 26 real unit tests, then the zero-test binary
# sub-suite, then the real unittest leg. Both kinds of marker are present.
COMPOSITE = (
    "running 26 tests\n"
    "test budget::tests::boundary_limit ... ok\n"
    "test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n"
    "running 0 tests\n"
    "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n"
    "Ran 94 tests in 0.448s\n"
    "OK\n"
)
CARGO_EMPTY = (
    "running 0 tests\n"
    "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n"
)
UNITTEST_EMPTY = "Ran 0 tests in 0.000s\n\nOK\n"
PYTEST_EMPTY = "collected 0 items\n\nno tests ran in 0.01s\n"
IGNORED_ONLY = (
    "running 0 tests\n"
    "test result: ok. 0 passed; 0 failed; 3 ignored; 0 measured; 9 filtered out\n"
)
FAILING = "running 4 tests\ntest result: FAILED. 1 passed; 3 failed\n"


def shell_quote(output_text):
    """One-line Verify command that prints this output verbatim.

    The gate reads a Verify from a single line, so the fixture's new
    lines travel as backslash escapes and printf expands them.
    """
    assert "'" not in output_text
    assert "%" not in output_text
    return "printf '" + output_text.replace("\n", "\\n") + "'"


def run_sweep(status, verify):
    """Run sweep_mode() against one temporary ticket; return (rc, output)."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        issues = root / "issues"
        issues.mkdir()
        ticket = issues / "01-probe.md"
        ticket.write_text(
            "# P-01: probe\n\n"
            "**Status:** " + status + "\n"
            "**Verify:** " + verify + "\n"
            "**Covers:** 1\n",
            encoding="utf-8",
        )
        saved_root, saved_issues = gate.ROOT, gate.ISSUES
        gate.ROOT, gate.ISSUES = root, [ticket]
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = gate.sweep_mode()
            return rc, buf.getvalue()
        finally:
            gate.ROOT, gate.ISSUES = saved_root, saved_issues


class CompositeVerifyTest(unittest.TestCase):
    """The false rejection: a green composite run must not read VACUOUS."""

    def test_composite_make_ci_output_is_not_vacuous(self):
        rc, out = run_sweep("done", shell_quote(COMPOSITE))
        self.assertNotIn("VACUOUS", out)
        self.assertNotIn("BROKEN", out)
        self.assertEqual(rc, 0, out)

    def test_composite_open_ticket_reports_stale_open_not_vacuous(self):
        rc, out = run_sweep("ready-for-agent", shell_quote(COMPOSITE))
        self.assertNotIn("VACUOUS", out)
        self.assertIn("STALE-OPEN", out)
        self.assertEqual(rc, 1, out)


class EmptyRunRefusalTest(unittest.TestCase):
    """ADR-031 refusals that must survive the ADR-034 repair."""

    def assert_vacuous(self, output_text):
        rc, out = run_sweep("done", shell_quote(output_text))
        self.assertIn("VACUOUS", out)
        self.assertNotIn("BROKEN", out)
        self.assertEqual(rc, 1, out)

    def test_cargo_filter_matching_nothing_is_still_vacuous(self):
        self.assert_vacuous(CARGO_EMPTY)

    def test_empty_unittest_run_is_still_vacuous(self):
        self.assert_vacuous(UNITTEST_EMPTY)

    def test_empty_pytest_collection_is_still_vacuous(self):
        self.assert_vacuous(PYTEST_EMPTY)

    def test_ignored_and_filtered_only_is_still_vacuous(self):
        self.assert_vacuous(IGNORED_ONLY)


class FailingRunTest(unittest.TestCase):
    def test_failing_done_ticket_reads_broken_not_vacuous(self):
        rc, out = run_sweep("done", shell_quote(FAILING) + "; exit 1")
        self.assertIn("BROKEN", out)
        self.assertNotIn("VACUOUS", out)
        self.assertEqual(rc, 1, out)


class ClassifierTest(unittest.TestCase):
    def test_executed_evidence_variants_suppress_vacuity(self):
        variants = (
            "running 1 test\n",
            "running 42 tests\n",
            "test result: ok. 1 passed\n",
            "Ran 1 test in 0.001s\n",
            "Ran 94 tests in 0.448s\n",
            "collected 1 item\n",
        )
        for evidence in variants:
            with self.subTest(evidence=evidence.strip()):
                self.assertFalse(gate.vacuous_output("running 0 tests\n" + evidence))

    def test_empty_markers_alone_are_vacuous(self):
        markers = (
            "running 0 tests\n",
            "test result: ok. 0 passed\n",
            "Ran 0 tests\n",
            "collected 0 items\n",
        )
        for marker in markers:
            with self.subTest(marker=marker.strip()):
                self.assertTrue(gate.vacuous_output(marker))


if __name__ == "__main__":
    unittest.main()
