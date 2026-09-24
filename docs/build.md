# Build

## Requirements

- Rust stable matching `workspace.package.rust-version`.
- Cargo.
- Just.
- C and C++ toolchains.
- CMake, Git, Python 3.
- `pkg-config`.
- Vulkan loader and headers.
- X11 and Wayland development headers used to build the bundled GLFW C sources.
- shaderc.

Install Just on the development machine and verify it with `just --version`. The Justfile runs
Cargo directly; system dependencies remain a prerequisite.

## Common Commands

```bash
just --list
just build
just build raw_pipeline
just build-workspace
just hooks-install
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
just test-platform         portable error conversion + GLFW Null backend
just test-platform-compile compile every platform probe without running it
just test-platform-null    isolated GLFW Null-backend probe
just test-platform-wayland Wayland contract; caller supplies Weston/compositor
just test-platform-x11     X11 contract; caller supplies X server/window manager
just test-gpu-compile      compile GPU tests without running them
just test-gpu              GPU/display tests
just doctest               Rustdoc examples
just check                 formatting and strict Clippy checks
just lint                  mutating format/fix pass
just docs                  Rustdoc build with warnings denied
just docs-api-check        non-mutating API book, inventory, snippet, and link checks
just docs-api-tools        install pinned API documentation tools under target/
just docs-api-update       regenerate reviewed API snapshot and indexes (mutating)
just hooks-install         enable repository-owned Git hooks for this clone
just bootstrap             install Linux system dependencies
```

`just lint` applies formatting and automatic fixes across the workspace. Inspect the worktree
first and use it only when those modifications are intended.

`just hooks-install` sets this clone's `core.hooksPath` to `.githooks`. The `commit-msg` hook
validates the subject, allowed type, optional scope, line lengths, and body separation before Git
creates a commit. It does not modify commit messages.

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

The public API book uses these exact tools:

- `just 1.57.0` for workflow orchestration;
- precompiled `mdbook 0.5.4` (keeps VMNL source validation on Rust 1.87);
- `cargo-public-api 0.52.0` plus `nightly-2026-03-12`;
- `lychee 0.24.2`.

Install the nightly with `rustup toolchain install nightly-2026-03-12 --profile minimal`.
Run `./tools/api_docs_tools.sh` when Just is unavailable, or `just docs-api-tools` otherwise.
The script installs the pinned tools under `target/api-tools`; precompiled archives are SHA-256
verified. `docs-api-check`, `docs-api-update`, and `validate` invoke the recipe automatically.
Existing exact versions in `PATH` are reused. The check runs lychee offline with fragment
validation: it checks local paths/anchors without making network requests. Public surface
extraction omits blanket implementations only.

The earlier `nightly-2025-08-02` candidate emits rustdoc JSON format 55 without
`external_crates.*.path`; the currently published `cargo-public-api 0.52.0` parser expects that
field and fails before producing an inventory. The pinned 2026-03-12 nightly emits the compatible
format 57 while stable VMNL compilation remains on Rust 1.87.

`just docs-api-check` does not change tracked files, but may populate `target/api-tools`.
`just docs-api-update` rewrites only
`public_api_snapshot.md`, `public_symbol_index.md`, `method_index.md`,
`glfw_platform_inventory.md`, and `platform_compatibility.md`; review their diff.
See the [API change protocol](api/maintenance/api_change_protocol.md).

## GLFW platform prerequisites

Platform recipes never install system packages. `test-platform-null` requires no display server.
`test-platform-wayland` requires an already running Wayland compositor and a matching
`WAYLAND_DISPLAY`; CI uses Weston with Pixman, nested under a dedicated Xvfb server whose X11
input seat satisfies the `wl_seat` requirement in GLFW 3.4.
`test-platform-x11` requires `DISPLAY` and
an EWMH-capable window manager; CI uses Xvfb with Openbox. These probes do not initialize Vulkan.

VMNL enables `glfw/src-build` so Linux does not silently substitute an arbitrary system GLFW 3.4
library for the audited bundled GLFW C 3.4.0 sources. CMake and the X11/Wayland development headers
therefore remain build prerequisites; installing `libglfw3-dev` does not change the selected C
implementation.

When adding or changing a GLFW call, follow the
[GLFW portability protocol](api/maintenance/glfw_portability_protocol.md) before running
`docs-api-update` and `docs-api-check`.
