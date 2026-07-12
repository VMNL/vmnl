# VMNL Technical Documentation

This directory is the canonical technical documentation for VMNL.

Local `README.md` files are only navigation aids for their directory. Public API contracts belong in Rustdoc. Behavior must be covered by tests.

## Index

- [Architecture](architecture.md): workspace layout, crates, public facade, internal layers.
- [Build](build.md): toolchain, system dependencies, shaderc/Vulkan discovery, runner usage.
- [Testing](testing.md): unit, API, smoke, GPU, and doctest conventions.
- [Examples](examples.md): visual example rules and command conventions.
- [Deployment](deployment.md): release checks and current publishing constraints.

## Documentation Rules

- `docs/*.md`: stable technical concepts, procedures, and invariants.
- `*/README.md`: short local orientation only.
- Rustdoc: public types, functions, and API-level contracts.
- Tests: executable behavior, not narrative documentation.
- Examples: user-facing visual programs that open a window.

