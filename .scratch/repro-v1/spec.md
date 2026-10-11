# repro-v1 spec — committed reports reproduce from the trace

Status: ready-for-agent

## Problem Statement

Three reports ship as committed artifacts. Nothing checks they
still regenerate from the trace fixture. Drift between code and
artifacts would silently age the release evidence. This track
adds a guard test that regenerates all three reports into temp
space and compares the deterministic sections with the
committed files.

## Acceptance Criteria

1. The pilot report regenerates byte-identical from the trace.
2. The baseline deterministic sections match the committed file.
3. The dossier deterministic sections match the committed file.
4. Measured sections stay compared as labeled-varying, never as
   identical bytes.

## Notes

Same-machine reproduction only; cross-machine independence
stays an open question. The guard proves no drift, not
portability. Reproduction behavior already holds; this ticket
adds the guard, stated plainly.
