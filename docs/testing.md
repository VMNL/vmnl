# Testing

## Test Types

```text
unit test   = close to implementation, small scope, may test private internals
API test    = black-box public behavior through the vmnl facade
smoke test  = executable startup path, no window, exit code is the oracle
GPU test    = Vulkan/display behavior, may create a window
doctest     = Rustdoc example compilation/execution
example     = user-facing visual program, not a test oracle
```

## Layout

```text
crates/*/src/** #[cfg(test)]  unit tests
tests/api/                    public headless API tests
tests/smoke/                  startup executables without windows
tests/gpu/                    Vulkan/display tests
examples/                     visual runnable examples
```

## Commands

```bash
just test-unit
just test-api
just test-smoke
just test-gpu
just test
just doctest
```

`just test` intentionally excludes GPU/display tests and doctests.

## Continuous Integration

The CI workflow invokes Cargo directly. The Justfile is a local development helper and is never
invoked in CI.

Its strict order is:

```text
format -> Clippy -> build -> headless tests -> documentation
```

Build and headless test stages run concurrently on the Linux, macOS, and Windows matrix. The
headless test stage runs unit, API, then smoke tests on each platform. Each job starts only after
every platform in the previous matrix succeeds. The final Linux documentation job builds Rustdoc,
then runs doctests.

CI sets `CARGO_INCREMENTAL=0` because GitHub-hosted jobs use fresh workspaces. This avoids
producing incremental artifacts that cannot be reused by later jobs.

GPU/display tests remain excluded from hosted CI because they require a compatible GPU, driver,
and display server.

## Invariants

- API tests must be headless.
- Smoke tests must not open a window.
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
