# Pi development protocol

## Start

1. Launch Pi from the repository root. Confirm `pi --version` returns `1.0.0`.
2. Review project settings and grant Pi project trust through its normal prompt.
3. Confirm Pi selects `xiaomi/mimo-v2.6-flash`. Run `/reload` when an existing session needs the new template.
4. Run `/hephaestus-task` after the user starts implementation. The template selects one small task.

The preparation request creates instructions and tooling. Runtime implementation starts only after the user requests it.
The project settings load only after trust. An existing session can retain its earlier model and thinking level.
Use `pi --model xiaomi/mimo-v2.6-flash --thinking low` for explicit startup selection.
The project template preserves global system instructions. No project SYSTEM.md or APPEND_SYSTEM.md replaces them.

## Read only the active task

1. Run `python3 .scratch/architecture-efficiency/workflow.py next`.
2. Read the selected ticket, its small task, and the shared specification's execution rules.
3. Read the cited requirement sections and relevant acceptance definitions before writing tests.
4. Read the named source files and reference tests. Use `rg` to resolve missing interfaces.

MiMo performs all development reasoning and reviews. Deterministic tools perform selection, calculations, execution, and checks.
Keep runtime provider contracts independent from the development model choice. AE-06 still concerns runtime providers.
Keep independent scientific qualification separate. A second MiMo review does not establish independent scientific confirmation.
Use one development session and one active small task by default. Parallelize independent reads and checks with bounded tool calls.
Use isolated artifacts and deterministic integration when a task actually requires concurrent development workers.

## Implement and verify

1. Run the selected small-task Verify. Add failing behavior tests when its planned target is absent.
2. Require a behavior failure before implementation. A missing target alone does not demonstrate the required behavior.
3. Implement the specified behavior. Treat named files as starting points; include necessary exports and callers in the same small task.
4. Run `python3 .scratch/architecture-efficiency/workflow.py verify AE-02 S1`, replacing the IDs with the selected values.
5. Read the receipt path. The runner rejects successful commands that execute zero tests.
6. Run relevant existing regression tests. Run `make ci` and `make doc-check` before landing the small task.
7. Review the diff against requirements. Preserve source provenance, protected evaluators, authorization, and scientific interpretation.
8. Follow repository commit authorization and cadence. Record status and checked children in the same implementation commit.

The runner preserves complete logs outside model context. Read its compact failure summary first.
Inspect additional lines only when needed.
Copy required receipts into the repository's evidence location before temporary logs expire. Keep secrets out of receipts.
Register new tracked paths in the runtime allowlist when preparing a commit. Apply the existing gate-seal decision procedure.
A green test filter must execute matching tests. The runner does not replace CI, hooks, specification review, or qualification.

## Control token use

Use codemode to batch independent tools. Discover tool declarations when needed; keep dependent edits and verification sequential.
Keep MiMo at low thinking for routine work. Increase thinking on the same model when invariants remain unresolved.
Record the reason for increased thinking. Never substitute another model automatically.
Keep evidence in files. Send paths, changed symbols, and compact errors into model context.
Read source ranges around affected symbols. Expand context when correctness requires it.
Keep authorization, requirement constraints, counterevidence, and unresolved failures visible.

The installed catalog declares a 1,048,576-token context window for MiMo.
The project reserves 983,040 tokens, giving a nominal 65,536-token compaction threshold.
Pi retains 20,000 recent tokens. These values are provisional development settings, not measured optimal values.
Compaction checks occur after tool results; the threshold is not a hard memory or input limit.
Record input, output, cache, and compaction tokens when Pi exposes them. Include retries and review calls.
Measure small-task elapsed time, failed checks, and peak process memory when instrumentation supports them.
Change these settings only from comparable observations that preserve correctness. Keep unavailable measurements explicit.

## Resume

Before compaction or session replacement, record the active TASK, small task, exact Verify, changed paths, and unresolved failures.
Include decision references, receipt paths, and the next nano step. Store the handoff with the task's comments or evidence.
After resume, compare the handoff with `git status --short`, ticket status, and current source versions.
Treat handoff text as a navigation aid. Current artifacts and verification establish completion.

## Verified scope

Local probes confirm Pi `1.0.0` and the `xiaomi/mimo-v2.6-flash` catalog entry.
Installed Pi 1.0.0 documentation defines project settings, prompt templates, model selection, codemode, and compaction overrides.
No paid model call, autonomous implementation, provider qualification, or measured efficiency result forms part of preparation.
The user must verify credential availability and actual tool execution when starting development.

Reference: [Pi configuration documentation](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/configuration.md).
Installed version-specific documentation remains authoritative for these settings.

## Preparation checks

`make ticket-status` passes. The workflow check confirms seven TASKS and 21 small tasks.
Dependency checks select AE-02/S1 first and AE-01 after AE-02 completes. Dependency cycles fail validation.
Verification checks reject zero tests, empty output, and failing tests. A nonempty successful test result passes.
Plan links and current source pointers resolve. Pi model listing succeeds with project trust enabled for that process.
The real AE-02/S1 Verify fails because its future test target does not exist. That failure preserves honest pending status.
Full application CI and live model execution remain outside these preparation checks.
