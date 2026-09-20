# VMNL Diagnostic Protocol

## Scope

Use this shared operational protocol when an applicable domain skill routes a bug, regression,
incorrect result, crash, hang, or performance diagnosis here. Domain skills remain responsible for
their own contracts, environment requirements, and evidence.

Follow [`AGENTS.md`](../../AGENTS.md), the relevant canonical documentation, and the test
classification in [`docs/testing.md`](../../docs/testing.md). Do not copy the completion sequence
from [`CONTRIBUTING.md`](../../CONTRIBUTING.md); link to it when a later fix requires validation.

A diagnosis request authorizes investigation, not a fix. Prefer read-only observations. Add temporary
instrumentation or a diagnostic harness only when the user requested it or explicitly approved the
repository change. Preserve unrelated work and remove only instrumentation introduced by the task.

## Diagnostic Loop

### 1. Define the invariant and symptom

- State the expected behavior and its owner: explicit request, canonical documentation, public
  Rustdoc, upstream specification, test, or measured baseline.
- State the observed behavior without interpreting its cause.
- Record the smallest relevant environment: revision, configuration, platform, backend, hardware,
  input, and timing conditions.
- Identify the evidence level required to observe the actual defect. Do not substitute a cheaper
  level that cannot exercise the failing boundary.

If sources disagree, report the contradiction and its contract owner before choosing an oracle.

### 2. Build the narrowest useful feedback signal

Prefer the lowest practical signal that exercises the real failure path:

1. existing focused test or probe;
2. minimized invocation with a fixed input;
3. isolated subsystem harness;
4. native backend or hardware scenario;
5. structured operator procedure when automation cannot observe the symptom.

The signal should distinguish the reported defect from nearby failures. Make it deterministic when
the system permits. For intermittent failures, record the workload, trial count, failure count, and
rate instead of reporting a single pass or failure.

When reproduction is unavailable, document what was attempted and which environment or artifact is
missing. Static reasoning may continue, but label its conclusions as hypotheses rather than observed
root causes.

### 3. Rank falsifiable hypotheses

Keep the list small enough to test. For each hypothesis, record:

```text
claim -> predicted observation -> falsifying observation -> narrowest check
```

Rank hypotheses using current evidence, boundary crossings, recent changes, and the number of
assumptions required. Do not treat the first plausible explanation as the default cause.

### 4. Verify, then instrument

- Run the cheapest check that can distinguish the leading hypotheses.
- Change one variable at a time and retain the same input and environment when comparing results.
- Prefer existing debuggers, traces, validation layers, counters, and focused logs before editing
  source code.
- Place instrumentation at boundaries that separate hypotheses, not everywhere along the path.
- Give temporary probes a unique marker so their removal can be verified.
- Redact credentials, tokens, personal data, addresses, and unrelated payload contents from commands,
  logs, traces, and reports.

For performance regressions, define the workload, baseline, metric, sampling method, and acceptable
variance before attributing a cause. Use instrumentation whose overhead is bounded and relevant to
the measured path.

At FFI, unsafe, concurrent, or callback boundaries, verify inputs, outputs, error channels, ownership,
lifetime, thread, ordering, and shutdown assumptions independently when they could distinguish the
hypotheses.

### 5. Decide from discriminating evidence

Classify each tested hypothesis as supported, contradicted, or unresolved and cite the observation.
Separate:

- root cause;
- contributing condition;
- visible symptom;
- unrelated defect discovered during the investigation.

Do not call correlation or a disappearing symptom proof of causation. After an inconclusive result,
restate the invariant, inspect the current diff, and choose a new targeted measurement before
broadening or repeating the investigation.

### 6. Define the fix boundary

For a diagnosis-only request, describe the smallest coherent fix and the regression seam without
implementing either. For an authorized fix, require a focused regression test or explain why the real
defect cannot be reproduced at a deterministic test seam. Validate the eventual change through the
applicable domain skill and [`CONTRIBUTING.md`](../../CONTRIBUTING.md#before-submission).

### 7. Clean up and report

- Remove temporary instrumentation, harnesses, logs, and artifacts created by the task unless the user
  explicitly asked to retain them.
- Do not remove or overwrite pre-existing user changes.
- Report the reproducer, commands executed, observed results, supported cause, remaining uncertainty,
  proposed fix boundary, and unverified environments.

## Evidence Boundaries

| Evidence | Establishes | Does not establish |
| --- | --- | --- |
| Static or compile | Code shape, types, or buildability | Runtime or backend correctness |
| Unit or API | Behavior covered by that seam | Unexercised native, device, or display behavior |
| Headless, Null, or simulated | Modeled software or backend behavior | Native platform or hardware behavior |
| Native platform | Named OS and backend behavior | Other platforms, GPU, or manual perception |
| GPU, device, or hardware | Exercised runtime configuration | Untested configurations or portability |
| Codex-observed example or screenshot | What the agent observed in that run | Human manual validation |
| Operator report | What the operator explicitly observed | Unreported automated or cross-platform coverage |
| Remote CI | Behavior on the reported runner matrix | Local hardware, unavailable backends, or manual behavior |

Report unavailable evidence as unavailable. Never promote compilation, a simulated backend, or a
planned manual procedure into stronger validation.

## Cross-Repository and Submodule Diagnoses

Treat the owning repository and its parent integration as separate diagnostic boundaries:

- establish the revision and worktree state of each repository;
- diagnose subsystem behavior in the owning repository;
- diagnose facade, feature, version-pin, and integration behavior in the parent repository;
- report commands and evidence per repository;
- do not infer that passing one boundary validates the other.

Keep each repository independently usable; do not rely on parent-relative diagnostic files from a
submodule checkout.
