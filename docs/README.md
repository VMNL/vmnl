# VMNL Technical Documentation

This directory is the canonical technical documentation for VMNL.

Local `README.md` files are only navigation aids for their directory. Public API contracts belong in Rustdoc. Behavior must be covered by tests.

## Index

- [API](API.md): portal to the exhaustive client-facing API book.
- [Architecture](architecture.md): workspace layout, crates, public facade, internal layers.
- [Build](build.md): toolchain, system dependencies, shaderc/Vulkan discovery, Justfile usage.
- [Coding Instructions](INSTRUCTIONS.md): project coding and documentation standards.
- [Contributing](../CONTRIBUTING.md): contribution workflow and commit convention.
- [Examples](examples.md): visual example rules and command conventions.
- [Getting Started](getting_started.md): clone, build, test, and run the first visual example.
- [Platform Support](platform_support.md): validated platforms and local environment scope.
- [Rust Instructions](RUST.md): rust coding guidelines.
- [Testing](testing.md): unit, API, smoke, GPU, and doctest conventions.
- [Troubleshooting](troubleshooting.md): shaderc, Vulkan, GLFW, and display diagnostics.

## Documentation Rules

- `docs/*.md`: stable technical concepts, procedures, and invariants.
- `*/README.md`: short local orientation only.
- Rustdoc: public types, functions, and API-level contracts.
- Tests: executable behavior, not narrative documentation.
- Examples: user-facing visual programs that open a window.
