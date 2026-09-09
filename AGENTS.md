# VMNL Repository Agent Rules

## Scope and Routing

These rules apply repository-wide. Read any closer `AGENTS.md` before editing within its scope.

For every graphics task affecting the facade, `vmnl-graphics`, related macros, Vulkan, windowing, input, rendering, GPU resources, graphics tests, examples, documentation, or tooling, use [`$develop-vmnl-graphics`](.agents/skills/develop-vmnl-graphics/SKILL.md) and follow only the references it routes. Do not apply graphics-specific requirements to unrelated subsystems.

## Establish the Contract

Before editing:

1. Inspect the worktree and preserve unrelated user changes.
2. Read `docs/README.md`, `docs/INSTRUCTIONS.md`, and only the canonical documentation relevant to the task.
3. For a feature, public API, architecture, module, or roadmap decision, read `docs/architecture.md` and preserve its project direction, control model, domain scope, and non-goals.
4. Classify the task as `Feature`, `Fix`, `Maintenance`, or `Read-only`; a read-only request does not authorize writes.
5. Identify applicable impacts: public API, headless behavior, GPU/window/display, visual examples, threading, tooling/build, dependencies/releases, FFI, domain independence, and performance.
6. Inspect affected tests and only the implementation needed to establish current behavior.

Resolve disagreements using this priority:

```text
explicit task
closest applicable AGENTS.md
canonical docs/
public Rustdoc
tests
implementation
examples
```

Tests and implementation prove current behavior, not necessarily intended behavior. Examples demonstrate usage but are not specifications or sole correctness oracles. Report a contradiction and its contract owner instead of silently choosing an interpretation.

## Bound the Change

Make the smallest coherent patch. Avoid unrelated refactors, renames, formatting, dependency upgrades, module moves, cleanup, and documentation rewrites. Reuse existing abstractions, ownership models, errors, and conventions when sufficient.

- A feature must prove each new or changed observable contract with the lowest practical deterministic test.
- A fix must reproduce the defect with a focused regression test or instrumentation when practical. If it cannot, report the attempted reproduction, blocker, evidence used, and remaining uncertainty.
- After an inconclusive result, inspect the current diff, restate the invariant, and obtain a new targeted measurement before broadening or repeating the change.
- Do not edit generated or third-party files manually. Change `Cargo.lock` only when dependency resolution legitimately changes.
- Preserve repository SPDX and `// SAFETY:` conventions. Validate unsafe, FFI, and security-sensitive changes against an explicit invariant; compilation alone is not evidence of correctness.

## Preserve Repository Boundaries

- `crates/vmnl` is the public facade for normal consumers.
- `crates/vmnl_graphics` owns graphics, windowing, input, and GPU-resource behavior.
- `crates/vmnl_macros` owns internal procedural macros used by VMNL crates.
- `tests/api` validates headless public behavior through the facade.
- `tests/smoke` validates executable startup without a window.
- `tests/platform` isolates GLFW backend behavior without Vulkan.
- `tests/gpu` isolates Vulkan, surface, presentation, GPU, and display-dependent behavior.
- `examples` contains user-facing visual workflows.

New audio, network, scene, or interoperability surfaces must preserve the modular target in `docs/architecture.md`. Headless audio and network clients must not acquire graphics dependencies through the facade.

Keep stable architecture, contracts, procedures, and status in `docs/`; public contracts in Rustdoc; and local `README.md` files as navigation.

For every feature or fix, assess public Rustdoc, technical and user documentation, examples and inventories, documentation navigation, and user-visible release-note impact under `docs/deployment.md`. Before the first public release, do not create per-change changelog entries. After it, record only user-visible additions, changes, fixes, and deprecations. Update only surfaces whose contract, behavior, workflow, capability, limitation, or navigation changed. In the final report, state why no documentation update was required when none was made.

## Protect Workspace and External State

Local inspection and editing do not authorize issues, PRs, reviews, comments, messages, pushes, tags, releases, uploads, publications, credentials, or secrets. An explicitly requested workflow authorizes only its necessary steps; creating a PR never authorizes merging it.

Before invoking repository tooling, read its prerequisites and mutation behavior. Do not install packages, invoke `sudo`, modify the host, clean build artifacts, or apply workspace-wide automatic fixes without explicit authorization. Prefer the narrowest non-mutating check that can disprove the current hypothesis.

Add or upgrade a dependency only when required. State why the current graph is insufficient, inspect API and duplicate-version impact, and validate the resulting graph.

## Release Policy

All VMNL releases are performed manually. The repository MUST NOT contain an automatic release or publication workflow unless the policy is explicitly changed.

`docs/deployment.md` is authoritative for release preconditions and current publication blockers. Manual release policy does not imply that the current crates.io dependency graph is publishable.

Never run a non-dry-run publication, create or push a release tag, create a GitHub release, or use release credentials without explicit authorization. Do not claim publishability until the documented blockers are resolved and the required dry-runs succeed.

## Validate With Evidence

For maintained Rust, test, example, tooling, or dependency changes, execute the applicable completion checks in the exact order defined by [`CONTRIBUTING.md`](CONTRIBUTING.md#before-submission). That section is the single command and ordering contract; do not duplicate it in agent references. `just test` combines unit, API, and smoke tests and is not a separate suite.

Do not change the order of applicable completion checks without an explicit technical justification. A blocked check does not authorize silently skipping later checks; report the blocker and continue only when doing so is safe and meaningful. Documentation-only work requires only applicable document, generated-file, snippet, link, structure, and consistency checks. Run doctests or documentation builds only when Rustdoc or embedded Rust examples change.

## Report and Prepare PR Descriptions

Report the changed behavior or documentation, relevant files, commands actually executed and observed results, failed or blocked checks, unverified behavior, and remaining uncertainty. Make no claim about compilation, tests, rendering, portability, correctness, publication, or performance without current-task evidence. Report unrelated observations separately without expanding the patch.

Prepare a PR description only when explicitly requested. Use the exact structure and evidence rules in [`CONTRIBUTING.md`](CONTRIBUTING.md#pull-request-descriptions). Never invent human manual graphical validation; if required evidence is missing, leave `Validation` empty and state outside the draft that validation is incomplete.
