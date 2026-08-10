# VMNL Repository Agent Rules

## Scope

These rules apply repository-wide. `MUST` and `MUST NOT` are unconditional; deviating from `SHOULD` requires a stated reason.

For every graphics task affecting the facade, `vmnl-graphics`, related macros, Vulkan, windowing, input, rendering, GPU resources, graphics tests, examples, documentation, or tooling, use [`$develop-vmnl-graphics`](.agents/skills/develop-vmnl-graphics/SKILL.md) and follow the references it routes.

Do not apply graphics-specific requirements to unrelated future audio, networking, or other subsystem work. Read any closer nested `AGENTS.md` before editing within its scope.

## Establish Context

Before editing:

1. Read `docs/README.md`, `docs/INSTRUCTIONS.md`, and only the canonical documentation relevant to the task.
2. Inspect the worktree and preserve unrelated user changes.
3. Inspect affected tests and only the implementation required to establish current behavior.
4. Identify the authoritative contract before resolving any disagreement.

Use this priority:

```text
explicit task
closest applicable AGENTS.md
canonical docs/
public Rustdoc
tests
implementation
examples
```

Tests and implementation prove current behavior, not necessarily intended behavior. Examples demonstrate usage but are neither specifications nor sole correctness oracles. If sources disagree, report the contradiction and contract owner; do not silently choose an interpretation or change unrelated behavior to manufacture consistency.

## Classify and Bound the Task

Classify the task before editing:

- `Feature`: introduces behavior or intentionally changes a contract.
- `Fix`: restores behavior that is already intended.
- `Maintenance`: changes documentation, tests, tooling, dependencies, or internals without intentionally changing observable library behavior.
- `Read-only`: review, explanation, diagnosis, or status; it does not authorize writes.

Also identify applicable impacts: public API, headless behavior, GPU/window/display, visual examples, tooling/build, dependencies/releases, and performance.

Make the smallest coherent patch. Avoid unrelated refactors, renames, formatting, dependency upgrades, module moves, cleanup, and documentation rewrites. Reuse existing abstractions, ownership models, errors, and conventions when sufficient. Refactor only when required for correctness, a documented invariant, or a maintainable implementation of the request.

Do not edit generated or third-party files manually. Change `Cargo.lock` only when dependency resolution legitimately changes. Preserve repository SPDX and `// SAFETY:` conventions.

## Preserve Repository Boundaries

- `crates/vmnl` is the public facade for normal consumers.
- `crates/vmnl_graphics` owns graphics, windowing, input, and GPU-resource behavior.
- `crates/vmnl_macros` owns internal procedural macros used by VMNL crates.
- `tests/api` validates headless public behavior through the facade.
- `tests/smoke` validates executable startup without a window.
- `tests/gpu` isolates Vulkan, display, and window-dependent behavior.
- `examples` contains user-facing visual workflows.

Keep stable architecture, contracts, procedures, and status in `docs/`; keep public contracts in Rustdoc; keep local `README.md` files as navigation. Update the authoritative source when intentionally changing behavior.

For a public change, determine the facade exposure, Rustdoc, tests, and documentation impact.

## Documentation Decisions

For every feature or fix, explicitly determine whether documentation must be created or updated. Review every applicable surface:

- public API Rustdoc;
- technical documentation under `docs/`;
- user-facing documentation;
- examples and their documented inventories;
- the `Unreleased` section of `CHANGELOG.md`;
- documentation navigation files.

Update the authoritative surface when a public contract, documented behavior, supported workflow, user-visible capability, limitation, or navigation entry changes. Update `CHANGELOG.md` only for user-visible features, fixes, and breaking changes.

Do not create an artificial documentation change when no contract, documented behavior, or public usage changed. In that case, the final response MUST state why no documentation update was required.

## Protect the Workspace and External State

Local in-scope inspection and editing do not authorize external actions. Issues, PRs, reviews, comments, messages, branch pushes, tags, releases, uploads, publications, credentials, and secrets require an explicit request. A requested workflow authorizes only its necessary steps; creating a PR never authorizes merging it.

Before invoking repository tooling, read its documented prerequisites and mutation behavior. Do not install packages, invoke `sudo`, modify the host, clean build artifacts, or apply workspace-wide automatic fixes without explicit authorization. Prefer the narrowest non-mutating check that can disprove the current hypothesis.

Add or upgrade a dependency only when required. State why the current graph is insufficient, inspect API and duplicate-version impact, and validate the resulting graph.

## Release Policy

All VMNL releases are performed manually. The repository MUST NOT contain an automatic release or publication workflow unless the policy is explicitly changed.

`docs/deployment.md` is authoritative for release preconditions and current publication blockers. Manual release policy does not imply that the current crates.io dependency graph is publishable.

Never run a non-dry-run publication, create or push a release tag, create a GitHub release, or use release credentials without explicit authorization. Do not claim publishability until the documented blockers are resolved and the required dry-runs succeed.

## Automated Validation Order

Execute every applicable automatic check in this order:

1. compilation: `cargo build --workspace --all-targets`;
2. formatting: `cargo fmt --all --check`;
3. Clippy with warnings denied: `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
4. doctests: `just doctest`;
5. documentation build: `just docs`;
6. unit tests: `just test-unit`;
7. API tests: `just test-api`;
8. smoke tests: `just test-smoke`;
9. when the task affects GPU-facing behavior, GPU test compilation with `just test-gpu-compile`, followed by `just test-gpu` when the environment supports execution.

`just test` combines unit, API, and smoke tests; do not report it as separate suites.

Do not change the order of applicable completion checks without an explicit technical justification. A blocked check does not authorize silently skipping later checks; report the blocker and continue only when doing so is safe and meaningful. Documentation-only work requires only applicable document, link, structure, and consistency checks.

## Human Manual Graphics Validation

Automatic checks, GPU tests, and examples executed by Codex do not replace manual graphical validation performed by the human operator.

Every feature that affects rendering or observable graphics behavior MUST include:

- a new manual graphical scenario specific to the feature, executed by the operator;
- an operator check of relevant existing graphical examples or scenarios for visible regressions;
- the applicable automatic tests and checks.

Every fix that affects rendering or observable graphics behavior MUST include:

- an operator reproduction or manual checkup of the defective graphical scenario;
- operator verification that the defect is no longer observable after the fix;
- an operator check of relevant existing graphical examples or scenarios for visible regressions;
- the applicable automatic tests and checks.

Treat a manual graphical validation as executed only when the operator explicitly reports that it was performed. Never infer, invent, or report successful human validation from source inspection, an automated result, a launched example, a screenshot, or an intended test plan.

When a pull request description or checkup is requested for an observable graphics change and no operator validation is known, state that the required manual validation is still missing and that the change cannot be considered fully validated. Do not imply that it passed.

## Report Evidence

Deliver a concise factual report proportional to the task:

- summarize the behavior or documentation changed;
- list relevant files;
- report commands actually executed and their observed results;
- distinguish failed, blocked, and not-run checks;
- state which documentation surfaces changed, or why no documentation update was required for a feature or fix;
- state unverified behavior and remaining uncertainty;
- make no claim about compilation, tests, rendering, portability, correctness, publication, or performance without direct evidence from the current task.

Report unrelated observations separately. They MUST NOT expand the patch or block delivery unless they directly affect current correctness or a required invariant.

## Pull Request Descriptions

Prepare a pull request description only when explicitly requested. Always use exactly this structure:

```markdown
## Context

<Context and reason for the pull request>

## Changes

- <Relevant change>
- <Tests added or updated>
- <Documentation added or updated>

## Validation

- <Manual graphical validation performed by the operator>
```

`Context` MUST explain the problem or need, why the change is required, and the expected result. It MUST NOT be left empty.

`Changes` MUST concisely list every relevant behavioral, API, architecture, test, example, and documentation change. Do not describe unchanged behavior.

`Validation` MUST contain only manual graphical validations explicitly reported by the human operator. Combine evidence already provided in the task context with evidence the operator provides when requesting the description. Do not include compilation, formatting, Clippy, doctests, documentation builds, unit tests, API/functional tests, smoke tests, GPU tests, CI results, or other automatic checks.

Never fabricate or assume manual validation. For an observable graphics change with no known operator validation, leave `Validation` empty and state outside the draft that the description cannot be finalized as validated until the mandatory manual checks are provided. For a non-graphical change, keep the section empty when no manual graphical validation exists; do not invent a `not applicable` validation result.
