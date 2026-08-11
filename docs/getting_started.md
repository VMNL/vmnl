# Getting Started

This guide prepares a development checkout. To use VMNL as a dependency, see
[Installation](../README.md#installation).

## Prerequisites

The validated development path is Linux; see [Platform Support](platform_support.md). Follow the
system requirements in [Build](build.md) before cloning. If a required Linux system dependency is
missing, `just bootstrap` installs packages through `sudo`; it modifies the host and must be run
explicitly.

## Clone

```bash
git clone https://github.com/VMNL/vmnl.git
cd vmnl
```

## Render a First Example

```bash
just run d2_shapes
```

Cargo builds `d2_shapes` when needed. Expected result: a window opens and renders the 2D shapes
example. This requires a Vulkan-capable GPU, a Vulkan loader, GLFW, and a display server.

## Verify the Checkout

Compile every workspace target:

```bash
just build-workspace
```

Then run the headless suite:

```bash
just test
```

Both commands must exit with status `0`. `just test` runs unit, API, and smoke tests; it does not
validate Vulkan, GLFW, or display-dependent behavior.

## Diagnose a Failure

- Build cannot find `shaderc`: follow [shaderc discovery](build.md#shaderc-discovery).
- `vulkaninfo` fails or rendering cannot initialize: follow [Vulkan loader or driver failure](troubleshooting.md#vulkan-loader-or-driver-failure).
- GLFW cannot create a window: follow [GLFW or display failure](troubleshooting.md#glfw-or-display-failure).

Use [Examples](examples.md) to choose another visual program, or [Testing](testing.md) for the test
classification and GPU-test workflow.
