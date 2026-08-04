# Build

## Requirements

- Rust stable matching `workspace.package.rust-version`.
- Cargo.
- C and C++ toolchains.
- CMake, Git, Python 3.
- `pkg-config`.
- Vulkan loader and headers.
- GLFW.
- shaderc.

The `./run` script checks the required tools and reports shaderc/GLFW discovery.

For build, run, unit, API, smoke, GPU, and combined-test modes, failed shaderc discovery causes the runner to invoke the platform package manager through `sudo` without a confirmation prompt. Provision dependencies first; do not use those modes when host modifications are not authorized.

## Common Commands

```bash
cargo build
./run -b d2_shapes
./run d2_shapes
```

Default run target:

```bash
./run
```

Equivalent to:

```bash
cargo run -p d2_shapes
```

## Runner Flags

```text
./run -ut   unit tests
./run -at   API tests
./run -ft   compatibility alias for API tests
./run -st   smoke tests
./run -gt   GPU/display tests
./run -t    unit + API + smoke
./run -d    doctests
./run -w    checks without -D warnings
./run -l    mutating format/fix pass
```

For non-mutating validation with warnings denied, use:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

`./run -l` applies formatting and automatic fixes across the workspace. Inspect the worktree first and use it only when those modifications are intended.

## shaderc Discovery

`./run` resolves shaderc through:

1. `SHADERC_LIB_DIR`, if set.
2. `pkg-config --variable=libdir shaderc`.
3. Common system library paths.

Invariant:

```text
SHADERC_LIB_DIR must be absolute and point to an existing directory.
```

## Debug Protocol

Hypothesis: shaderc is not discoverable.

Verification:

```bash
pkg-config --exists shaderc
pkg-config --modversion shaderc
pkg-config --variable=libdir shaderc
```

Instrumentation:

```bash
./run -b d2_shapes
```

Decision:

- If `pkg-config` finds shaderc, prefer fixing `SHADERC_LIB_DIR` or library paths.
- If `pkg-config` does not find shaderc, install the system shaderc development package.
- If Vulkan fails at runtime, inspect loader/driver state separately from shaderc.
