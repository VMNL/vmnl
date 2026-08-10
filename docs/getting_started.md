# Getting Started

## Prerequisites

Follow the system requirements in [Build](build.md). The validated development path is Linux; see [Platform Support](platform_support.md).

## Clone and Build

```bash
git clone https://github.com/VMNL/vmnl.git
cd vmnl
just build-workspace
```

## Verify the Workspace

Run the headless suite before using a display-dependent example:

```bash
just test
```

The command runs unit, API, and smoke tests. It must exit with status `0`.

## Run a First Example

```bash
just run d2_shapes
```

Expected result: a window opens and renders the 2D shapes example. A Vulkan-capable GPU and a display server are required.

Use [Examples](examples.md) to choose another visual program and [Troubleshooting](troubleshooting.md) when the build or window initialization fails.
