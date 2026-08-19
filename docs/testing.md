# Testing

## Test Types

```text
unit test   = close to implementation, small scope, may test private internals
API test    = black-box public behavior through the vmnl facade
smoke test  = executable startup path, no window, exit code is the oracle
platform test = GLFW NoApi window behavior, no Vulkan or GPU surface
GPU test    = Vulkan/display behavior, may create a window
doctest     = Rustdoc example compilation/execution
example     = user-facing visual program, not a test oracle
```

## Layout

```text
crates/*/src/** #[cfg(test)]  unit tests
tests/api/                    public headless API tests
tests/smoke/                  startup executables without windows
tests/platform/               isolated GLFW backend probes without Vulkan
tests/gpu/                    Vulkan/display tests
examples/                     visual runnable examples
```

## Commands

```bash
just test-unit
just test-api
just test-smoke
just test-platform
just test-platform-compile
just test-platform-null
just test-platform-wayland
just test-platform-x11
just test-gpu
just test
just doctest
```

`just test` intentionally excludes platform, GPU/display tests, and doctests. `just validate` adds
the portable error-conversion and GLFW Null-backend suite after smoke tests.

## Continuous Integration

The CI workflow invokes Cargo directly. The Justfile is a local development helper and is never
invoked in CI.

Its strict job order is:

```text
Quality (format -> Clippy)
  -> Validation per OS (build -> unit -> API -> smoke -> platform)
  -> Documentation
```

Each OS validation job reuses one Cargo target directory for compilation and every test stage; no
target directory is cached or transferred between runners. Linux then forces the GLFW Wayland
backend under Weston with Pixman nested on Xvfb and tests the GLFW X11 backend under Xvfb with
Openbox. Win32 and Cocoa hidden-window probes remain visible but non-blocking until ten consecutive
successful runs use the same runner image, GLFW revision, and probe schema; any of those changes
resets the count.

The documentation job runs only after all OS validation jobs. Its pinned API tools are cached by
platform, architecture, and installer-script hash, and the installer still verifies every restored
tool version before use.

CI sets `CARGO_INCREMENTAL=0` because GitHub-hosted jobs use fresh workspaces. This avoids
producing incremental artifacts that cannot be reused by later jobs.

GPU/display tests remain excluded from hosted CI because they require a compatible GPU, driver,
and display server.

## Invariants

- API tests must be headless.
- Smoke tests must not open a window.
- Platform probes must use `ClientApi::NoApi`, run one operation per subprocess, and emit one
  versioned JSON record. Missing output, unexpected backend, non-zero status, signal, or abort is
  a failure.
- GPU tests must be isolated under `tests/gpu`.
- Visual examples must live under `examples`.
- Tests must assert behavior or fail through a non-zero exit code.
- `just test` combines the unit, API, and smoke suites.

## Classification

Use this decision order:

```text
tests private/local implementation detail -> unit test
tests vmnl as an external user would      -> API test
initializes and exits without window      -> smoke test
needs Vulkan/display/window               -> GPU test
needs only a native GLFW window            -> platform test
demonstrates visual user workflow         -> example
```

## GPU Tests

GPU tests are separated because they depend on the machine:

- Vulkan loader.
- GPU driver.
- Display server.
- GLFW window creation.

Compile without running them:

```bash
just test-gpu-compile
```

Run them explicitly:

```bash
just test-gpu
```
