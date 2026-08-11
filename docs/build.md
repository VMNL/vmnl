# Build

## Requirements

- Rust stable matching `workspace.package.rust-version`.
- Cargo.
- Just.
- C and C++ toolchains.
- CMake, Git, Python 3.
- `pkg-config`.
- Vulkan loader and headers.
- GLFW.
- shaderc.

Install Just on the development machine and verify it with `just --version`. The Justfile runs
Cargo directly; system dependencies remain a prerequisite.

## Common Commands

```bash
just --list
just build
just build raw_pipeline
just build-workspace
just run
just run d2_shapes
just test
just validate
```

`just build` and `just run` default to `d2_shapes`. `just build-workspace` compiles every
workspace target. `just validate` runs the required non-GPU build, check, Rustdoc, and headless
test sequence.

## Recipes

```text
just test                  unit + API + smoke tests
just test-unit             unit tests
just test-api              API tests
just test-smoke            smoke tests
just test-gpu-compile      compile GPU tests without running them
just test-gpu              GPU/display tests
just doctest               Rustdoc examples
just check                 formatting and strict Clippy checks
just lint                  mutating format/fix pass
just docs                  Rustdoc build with warnings denied
just bootstrap             install Linux system dependencies
```

`just lint` applies formatting and automatic fixes across the workspace. Inspect the worktree
first and use it only when those modifications are intended.

## Test Summaries

Each `just test-*` recipe streams Cargo output then prints a final suite summary. Interactive
terminals use color; `NO_COLOR=1 just test-api` disables it. The summary distinguishes executed
tests from `just test-gpu-compile`, which reports compiled executables only.

`just test` and `just validate` finish with a detailed recap of each executed suite or step,
followed by their aggregate. `just validate` remains a local non-GPU validation, not CI parity.

## Dependency Bootstrap

`just bootstrap` installs the system packages required for VMNL builds on supported Linux
distributions. It invokes the dedicated `./deps` script.

It requires `/etc/os-release` and invokes the appropriate package manager for the detected distro family.

## shaderc Discovery

### Debug Protocol

Hypothesis: shaderc is not discoverable.

Verification:

```bash
pkg-config --exists shaderc
pkg-config --modversion shaderc
pkg-config --variable=libdir shaderc
```

Instrumentation:

```bash
just build d2_shapes
```

Decision:

- If `pkg-config` finds shaderc, retry the smallest failing Just recipe.
- If `pkg-config` does not find shaderc, run `just bootstrap` or install the system shaderc development package.
- If Vulkan fails at runtime, inspect loader/driver state separately from shaderc.
