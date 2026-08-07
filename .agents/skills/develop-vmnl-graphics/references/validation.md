# Graphics Validation Protocol

## Classify Evidence

Use the lowest test level that proves the behavior:

| Behavior | Evidence |
| --- | --- |
| Private or local headless behavior | Unit test next to the implementation |
| Public headless behavior | API test through `vmnl` under `tests/api` |
| Executable startup without a window | Smoke test under `tests/smoke` |
| Vulkan, device, queue, pipeline, window, or display behavior | GPU test under `tests/gpu` |
| Stable public Rustdoc contract | Doctest; use `no_run` for unavailable runtime requirements |
| User-facing visual workflow | Example plus a separate automated invariant |

Keep API tests headless and smoke tests windowless. Never use an example as the only correctness oracle. Treat `./run -ft` only as an alias for `./run -at`; do not run or report both.

For a feature, test every new or changed observable behavior with a deterministic, maintainable oracle. If that is impossible, automate the nearest invariant and justify the remaining GPU or visual check.

For a fix, reproduce with the smallest practical test or instrumentation. Prefer a test that fails before and passes after, and retain it when deterministic, focused, maintainable, and reasonably fast. If reproduction is impossible, report the attempt, blocker, evidence used, and uncertainty.

## Preflight the Runner

Before invoking `./run -ut`, `-at`, `-ft`, `-st`, `-gt`, `-t`, `-b`, or an example/default run:

1. Read `docs/build.md`.
2. Verify shaderc through `SHADERC_LIB_DIR` or non-mutating `pkg-config` checks.
3. Accept `SHADERC_LIB_DIR` only when absolute and pointing to an existing directory.

Those runner modes currently invoke system package installation through `sudo` when shaderc discovery fails. If discovery is not proven, do not run them, install packages, modify the host, or invent a path. Report the missing prerequisite and the documented resolution.

Treat these runner modes as mutating:

- `./run -l` applies workspace-wide formatting and automatic fixes with dirty/staged files allowed;
- `./run -c` and `./run -r` invoke `cargo clean`.

Use them only when explicitly intended, preserve pre-existing changes, and inspect the resulting diff. `./run -w` omits `-D warnings`; it is not a substitute for the Clippy command below.

## Validate During Development

Run the narrowest command that can disprove the current hypothesis. Do not repeat the full suite after every small edit.

For documentation-only work, run relevant structure, link, and consistency checks. Do not run the Rust suite merely to produce validation output. Run doctests or documentation builds when Rustdoc or embedded Rust examples change.

## Complete Graphics Changes

Follow the exact completion sequence in the root `AGENTS.md`. Do not substitute `cargo check` for compilation. A successful build alone is insufficient.

Add checks by impact:

| Impact | Additional evidence |
| --- | --- |
| Public API | Headless API tests through `vmnl` and relevant Rustdoc/documentation |
| Example workflow | `./run -b <example>` |
| Relevant visual diagnosis and supported environment | Codex may run `./run <example>` after defining the expected observation, but this does not count as human manual validation |
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
