# VMNL Technical Documentation

This directory is the canonical technical documentation for VMNL.

Local `README.md` files are only navigation aids for their directory. Public API contracts belong in Rustdoc. Behavior must be covered by tests.

## Index

- [Coding Instructions](INSTRUCTIONS.md): project coding and documentation standards.
- [API](API.md): control spectrum and high-level/low-level API composition.
- [Architecture](architecture.md): workspace layout, crates, public facade, internal layers.
- [Build](build.md): toolchain, system dependencies, shaderc/Vulkan discovery, runner usage.
- [Getting Started](getting_started.md): clone, build, test, and run the first visual example.
- [Platform Support](platform_support.md): validated platforms and local environment scope.
- [Testing](testing.md): unit, API, smoke, GPU, and doctest conventions.
- [Examples](examples.md): visual example rules and command conventions.
- [Troubleshooting](troubleshooting.md): shaderc, Vulkan, GLFW, and display diagnostics.
- [Deployment](deployment.md): manual release checks and current publishing constraints.
- [Contributing](../CONTRIBUTING.md): validation, pull request, and commit conventions.

## Documentation Rules

- `docs/*.md`: stable technical concepts, procedures, and invariants.
- `*/README.md`: short local orientation only.
- Rustdoc: public types, functions, and API-level contracts.
- Tests: executable behavior, not narrative documentation.
- Examples: user-facing visual programs that open a window.
