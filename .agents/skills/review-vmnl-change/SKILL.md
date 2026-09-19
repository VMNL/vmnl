---
name: review-vmnl-change
description: Review or audit a concrete VMNL diff, commit, branch, pull request, or current worktree for contract, correctness, safety, and validation gaps. Do not use for implementation, diagnosis, brainstorming, change summaries, or general documentation review.
---

# Review VMNL Change

## Outcome

Produce a findings-first, evidence-backed review of a scoped change. Determine whether the change
implements its requested contract without introducing correctness, safety, architecture, validation,
documentation, or scope regressions.

The review is read-only. Do not edit files, apply fixes, create or submit a GitHub review, comment on
an issue or pull request, or change external state unless the user explicitly requests that separate
action.

## Establish the Review Target

1. Inspect the worktree before choosing the comparison. Include relevant staged, unstaged, and
   untracked files when reviewing current work.
2. Use a user-specified commit, branch, tag, merge-base, pull request, or path when provided. Resolve
   the target before reviewing and report an invalid or empty comparison.
3. Identify the originating request and its contract owner. Prefer the explicit request, then the
   priority order and domain routing in [`AGENTS.md`](../../../AGENTS.md).
4. Read only the canonical documentation, tests, implementation, and upstream specifications needed
   to evaluate the changed behavior. When no specification exists, state which review axes remain
   possible instead of inventing requirements.
5. For graphics changes, also use
   [`develop-vmnl-graphics`](../develop-vmnl-graphics/SKILL.md) and the references it routes.

## Review Axes

### Intent and scope

- Check every requested behavior for missing, partial, or incorrect implementation.
- Identify behavior added without a current requirement, unrelated refactors, and accidental public
  API or compatibility changes.
- For submodules or cross-repository changes, distinguish subsystem behavior from parent integration
  and review each revision and diff independently.

### Contract and architecture

- Compare observable behavior with its canonical owner and public Rustdoc.
- For architecture, feature, public API, module, or roadmap changes, apply
  [`docs/architecture.md`](../../../docs/architecture.md), including progressive control, domain
  independence, ownership, lifecycle, threading, synchronization, cost, and error boundaries.
- Report contradictions between documentation, tests, implementation, examples, and the request;
  identify the source that owns the unresolved decision.

### Correctness and safety

- Trace changed inputs, state transitions, outputs, errors, and cleanup through the affected callers.
- Check invalid and boundary inputs, partial failure, cancellation or shutdown, and resource lifetime.
- At FFI and `unsafe` boundaries, verify preconditions before crossing the boundary and confirm that
  each safety claim states a checkable invariant.
- For concurrent or callback code, verify ownership, ordering, thread affinity, bounded work, and
  destruction or shutdown synchronization.
- Treat panic removal, error propagation, determinism, allocation, zero-copy, non-blocking, and
  performance claims as unproven until the changed path supplies relevant evidence.

### Tests and validation evidence

- Verify that each changed observable contract has a deterministic test at the lowest level that
  exercises the real behavior.
- For fixes and performance regressions, read
  [`diagnostic-protocol.md`](../../references/diagnostic-protocol.md) and assess the reproducer,
  hypothesis evidence, regression seam, and remaining uncertainty.
- Distinguish compilation, unit or API tests, headless or Null probes, native platform execution,
  GPU or hardware execution, remote CI, Codex observations, and operator-reported manual evidence.
- Do not infer unavailable runtime, backend, hardware, portability, or manual evidence from a weaker
  check. Report commands as passed only when their current-task result was observed.
- Refer to [`CONTRIBUTING.md`](../../../CONTRIBUTING.md#before-submission) for completion order rather
  than duplicating it.

### Documentation and delivery

- Assess public Rustdoc, technical and user documentation, examples and inventories, navigation, and
  release-note impact only where the change affects their owned contract or workflow.
- Do not request documentation churn for unchanged behavior or internal implementation details.
- Check dependency, build, platform, and release implications when the diff affects them.

## Finding Threshold

Report a finding only when the changed code or documentation creates a concrete failure, violated
contract, unsafe assumption, portability or performance risk, missing required behavior, or material
validation gap. Do not report formatter output, lint handled mechanically by existing tooling, vague
preferences, or speculative abstractions without a current use case.

For each finding:

- assign a severity based on user impact and likelihood;
- cite the narrowest file and line location;
- state the violated contract or invariant;
- explain the observable consequence and supporting evidence;
- give the smallest useful correction direction without implementing it.

Group manifestations that share one root cause. Mark an inference as an inference and name the check
that would confirm it.

## Report

Present findings first, ordered by severity. Then include only relevant sections for:

- open questions or assumptions;
- validation and environment gaps;
- a short change summary.

If there are no findings, say so explicitly and report remaining test or environment limitations.
Do not prepare a pull-request description unless separately requested.
