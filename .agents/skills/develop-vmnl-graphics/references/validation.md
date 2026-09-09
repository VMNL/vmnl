# Graphics Validation Protocol

## Classify Evidence

Use the lowest test level that proves the behavior:

| Behavior | Evidence |
| --- | --- |
| Private or local headless behavior | Unit test next to the implementation |
| Public headless behavior | API test through `vmnl` under `tests/api` |
| Executable startup without a window | Smoke test under `tests/smoke` |
| GLFW `NoApi` behavior without Vulkan | Platform test under `tests/platform` |
| Vulkan, device, queue, pipeline, surface, presentation, or display behavior | GPU test under `tests/gpu` |
| Stable public Rustdoc contract | Doctest; use `no_run` for unavailable runtime requirements |
| User-facing visual workflow | Example plus a separate automated invariant |

Keep API tests headless and smoke tests windowless. Platform tests must not create a Vulkan instance
or surface. Never use an example as the only correctness oracle. `just test` combines unit, API,
and smoke suites.

For a feature, test every new or changed observable behavior with a deterministic, maintainable oracle. If that is impossible, automate the nearest invariant and justify the remaining GPU or visual check.

For a fix, reproduce with the smallest practical test or instrumentation. Prefer a test that fails before and passes after, and retain it when deterministic, focused, maintainable, and reasonably fast. If reproduction is impossible, report the attempt, blocker, evidence used, and uncertainty.

## Preflight Just

Before invoking build, test, or run recipes:

1. Read `docs/build.md`.
2. Verify shaderc through non-mutating `pkg-config` checks.

`just bootstrap` is the only recipe that invokes host mutation through `sudo`. If discovery is not proven, do not run it without authorization; report the missing prerequisite and the documented resolution.

Treat these recipes as mutating:

- `just lint` applies workspace-wide formatting and automatic fixes with dirty/staged files allowed;
- `just bootstrap` installs system dependencies.

Use them only when explicitly intended, preserve pre-existing changes, and inspect the resulting diff. `just check-clippy` is the strict Clippy check.

## Validate During Development

Run the narrowest command that can disprove the current hypothesis. Do not repeat the full suite after every small edit.

For documentation-only work, run relevant structure, link, and consistency checks. Do not run the Rust suite merely to produce validation output. Run doctests or documentation builds when Rustdoc or embedded Rust examples change.

## Complete Graphics Changes

After maintained Rust, test, example, tooling, or dependency changes, follow the applicable
completion sequence in [`CONTRIBUTING.md`](../../../../CONTRIBUTING.md#before-submission). It is the
single command and ordering contract. Do not substitute `cargo check` for compilation, reorder
applicable checks without justification, or copy the sequence into this reference.

Add checks by impact:

| Impact | Additional evidence |
| --- | --- |
| Public API | Headless API tests through `vmnl`, relevant Rustdoc, API-book update, and API-book check |
| Example workflow | `just build <example>` |
| Relevant visual diagnosis and supported environment | Codex may run `just run <example>` after defining the expected observation, but this does not count as human manual validation |
| Tooling/build script | Targeted syntax or behavior check; Rust suite if behavior changed |
| Dependency | Full Rust suite, dependency-graph inspection, and `Cargo.lock` diff |
| Performance | Comparable before/after measurement defined before editing |

GPU-test compilation proves compilation only. Run GPU tests only with a compatible GPU, loader, driver, display server, and GLFW window creation. A successful example does not replace automated tests.

## Require Human Manual Graphics Validation

For every feature affecting rendering or observable graphics behavior, require explicit operator evidence for:

- a new manual graphical scenario specific to the feature;
- relevant existing graphical examples or scenarios checked for visible regressions.

For every fix affecting rendering or observable graphics behavior, require explicit operator evidence for:

- reproduction or manual checkup of the defective scenario;
- verification that the defect is no longer observable;
- relevant existing graphical examples or scenarios checked for visible regressions.

Automated checks, GPU tests, Codex-run examples, screenshots, and planned procedures do not satisfy this requirement. Record a human manual validation only when the operator explicitly states it was performed. If evidence is missing during a PR-description request or checkup, report the missing validation and do not call the change fully validated.

## Report Evidence

In addition to the root report, identify which GPU/display/platform assumptions were exercised. Distinguish GPU-test compilation, GPU execution, Codex-observed example output, and operator-reported graphical validation; none is evidence for the others.
