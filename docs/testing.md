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

## Test Design

Start from the observable contract and its owner: explicit requirement, canonical documentation,
public Rustdoc, upstream specification, or an approved measured baseline. Select the lowest
practical deterministic seam that exercises that contract rather than an implementation detail
that merely correlates with it.

- Public facade behavior belongs in an API test when it can remain headless.
- A private invariant or algorithm may be covered by a focused unit test.
- Backend, platform, GPU, FFI, and hardware behavior requires evidence that crosses the real
  boundary; a mock, Null backend, stub, or Rust-only seam proves only its modeled behavior.
- Keep lower-level tests that prove distinct safety, ABI, platform, or failure-handling invariants;
  coverage through a higher interface does not make them redundant.

Expected results must come from an independent oracle: a specification, known worked case, fixed
literal, qualified reference implementation, or directly observable outcome. A test that derives
its expectation with the same algorithm as the implementation is tautological. Tests should fail
when the required behavior is absent and should survive unrelated internal refactors.

For a deterministic feature or fix, prefer one vertical behavior slice at a time:

```text
one observable contract
  -> one focused failing test
    -> the smallest coherent implementation
      -> repeat for the next contract
```

Observing the focused test fail before the change proves that it can detect the missing or defective
behavior. Do not fabricate that evidence or force this cycle when the real boundary is unavailable,
intermittent, visual, platform-specific, or hardware-dependent. In those cases, record the attempted
reproduction and missing environment, add the narrowest deterministic coverage that remains valid,
and keep the required native, GPU, FFI, remote-system, or operator evidence explicitly outstanding.

Compilation, deterministic tests, simulated or Null backends, native platform runs, GPU or hardware
runs, remote CI, and operator observations are separate evidence classes. Report only the class
actually exercised; success at a weaker boundary does not establish a stronger one.

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
backend under Weston with Pixman nested on Xvfb with Openbox, then tests the GLFW X11 backend
under Xvfb with Openbox. Both paths run a visible keyboard probe: a separate test process
waits for the window to be focused, injects an `A` press/release pair through XTEST, and
requires the exact native GLFW event sequence before timeout. The Wayland path injects through
the parent Xvfb server into Weston's X11 backend; it does not qualify a standalone Wayland
compositor seat.
The Wayland NoApi probe attaches a zero-filled shm buffer so its surface can be mapped without
Vulkan. It signals MAPPED after attaching the buffer; the parent clicks the center of the focused
Weston X11 window, identified by WM_CLASS, until GLFW confirms keyboard focus and signals READY.
MAPPED alone never authorizes keyboard injection.

Win32 uses `SendInput` and Cocoa uses `CGEventPost` for the same scenario. Their native probes
remain visible but non-blocking until ten consecutive successful runs use the same runner image,
GLFW revision, injector, and probe schema; any of those changes resets the count.

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
- Native-input probes must acquire focus before signaling readiness, receive the injected press and
  release in order, and fail non-zero on focus failure, wrong input, timeout, crash, or abort.
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
