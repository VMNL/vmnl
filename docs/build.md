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
just docs-api-check        non-mutating API book, inventory, snippet, and link checks
just docs-api-update       regenerate reviewed API snapshot and indexes (mutating)
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

## API Documentation Tooling

The public API book requires these exact tools in `PATH`:

- precompiled `mdbook 0.5.4` (keeps VMNL source validation on Rust 1.87);
- `cargo-public-api 0.52.0` plus `nightly-2026-03-12`;
- `lychee 0.24.2`.

Install mdBook from its versioned release archive, and install the nightly with the minimal
rustup profile. `docs-api-check` runs lychee offline with fragment validation: it checks local
paths/anchors without making network requests. Public surface extraction omits blanket
implementations only.

The earlier `nightly-2025-08-02` candidate emits rustdoc JSON format 55 without
`external_crates.*.path`; the currently published `cargo-public-api 0.52.0` parser expects that
field and fails before producing an inventory. The pinned 2026-03-12 nightly emits the compatible
format 57 while stable VMNL compilation remains on Rust 1.87.

`just docs-api-check` is non-mutating. `just docs-api-update` rewrites only
`public_api_snapshot.md`, `public_symbol_index.md`, and `method_index.md`; review their diff.
See the [API change protocol](api/maintenance/api_change_protocol.md).
