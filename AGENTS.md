# VMNL Repository Agent Rules

## Scope and Routing

These rules apply repository-wide. Read any closer `AGENTS.md` before editing within its scope.

For every graphics task affecting the facade, `vmnl-graphics`, related macros, Vulkan, windowing, input, rendering, GPU resources, graphics tests, examples, documentation, or tooling, use [`$develop-vmnl-graphics`](.agents/skills/develop-vmnl-graphics/SKILL.md) and follow only the references it routes. Do not apply graphics-specific requirements to unrelated subsystems.

## Establish the Contract

Before editing:

1. Inspect the worktree and preserve unrelated user changes.
2. Read `docs/README.md`, `docs/INSTRUCTIONS.md`, and only the canonical documentation relevant to the task.
3. Classify the task as `Feature`, `Fix`, `Maintenance`, or `Read-only`; a read-only request does not authorize writes.
4. Identify applicable impacts: public API, headless behavior, GPU/window/display, visual examples, tooling/build, dependencies/releases, and performance.
5. Inspect affected tests and only the implementation needed to establish current behavior.

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
- `tests/gpu` isolates Vulkan, display, and window-dependent behavior.
- `examples` contains user-facing visual workflows.

Keep stable architecture, contracts, procedures, and status in `docs/`; public contracts in Rustdoc; and local `README.md` files as navigation.

For every feature or fix, assess public Rustdoc, technical and user documentation, examples and inventories, the `Unreleased` section of `CHANGELOG.md`, and documentation navigation. Update only surfaces whose contract, behavior, workflow, capability, limitation, or navigation changed. In the final report, state why no documentation update was required when none was made.

## Protect Workspace and External State

Local inspection and editing do not authorize issues, PRs, reviews, comments, messages, pushes, tags, releases, uploads, publications, credentials, or secrets. An explicitly requested workflow authorizes only its necessary steps; creating a PR never authorizes merging it.

Before invoking repository tooling, read its prerequisites and mutation behavior. Do not install packages, invoke `sudo`, modify the host, clean build artifacts, or apply workspace-wide automatic fixes without explicit authorization. Prefer the narrowest non-mutating check that can disprove the current hypothesis.

Add or upgrade a dependency only when required. State why the current graph is insufficient, inspect API and duplicate-version impact, and validate the resulting graph.

All VMNL releases are manual. The repository must not contain automatic release or publication workflows unless this policy is explicitly changed. Never publish, create or push a release tag, create a GitHub release, or use release credentials without explicit authorization. Do not claim publishability without successful dry-runs of the current workspace graph.

## Validate with Evidence

During development, run the narrowest check that can disprove the current hypothesis. For maintained Rust source changes, execute every applicable completion check in this order:

1. `cargo build --workspace --all-targets`
2. `cargo fmt --all --check`
3. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
4. `./run -d`
5. `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`
6. `./run -ut`
7. `./run -at`
8. `./run -st`
9. For GPU-facing changes, `cargo test -p vmnl-gpu-tests --no-run`, followed by `./run -gt` when the environment supports execution.

`./run -ft` aliases `./run -at`; never execute or report both as separate suites. Documentation-only work requires only applicable document, link, structure, and consistency checks. Rustdoc or embedded Rust example changes also require the relevant doctest or documentation build.

Do not reorder applicable completion checks without a technical justification. On failure, record the exact command, the first relevant error, whether evidence indicates code or environment, and the behavior left unverified. A blocked check does not authorize silently skipping later checks; continue only when doing so is safe and meaningful.

Graphics test placement, runner preflight, GPU requirements, and mandatory operator validation are defined in the graphics skill's [`validation.md`](.agents/skills/develop-vmnl-graphics/references/validation.md).

## Report and Prepare PR Descriptions

Report the changed behavior or documentation, relevant files, commands actually executed and observed results, failed or blocked checks, unverified behavior, and remaining uncertainty. Make no claim about compilation, tests, rendering, portability, correctness, publication, or performance without current-task evidence. Report unrelated observations separately without expanding the patch.

Prepare a PR description only when explicitly requested. Use the exact structure and evidence rules in [`CONTRIBUTING.md`](CONTRIBUTING.md#pull-request-descriptions). Never invent human manual graphical validation; if required evidence is missing, leave `Validation` empty and state outside the draft that validation is incomplete.
