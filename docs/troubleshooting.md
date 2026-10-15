# Troubleshooting

Use the smallest command that exercises the failing layer.

## shaderc Not Found

Hypothesis: `shaderc` cannot be located.

```bash
pkg-config --exists shaderc
pkg-config --modversion shaderc
pkg-config --variable=libdir shaderc
```

If discovery succeeds, retry the smallest failing Just recipe. Otherwise, run `just bootstrap` or install the shaderc development package for the active distribution. See [Build](build.md#shaderc-discovery).

## shaderc CMake Compiler Cache Mismatch

Hypothesis: a source-build cache was created with a different C or C++ compiler. The diagnostic contains `You have changed variables that require your cache to be deleted`, followed by changed `CMAKE_C_COMPILER` or `CMAKE_CXX_COMPILER` values.

Verify native discovery first:

```bash
pkg-config --exists shaderc
pkg-config --modversion shaderc
pkg-config --variable=libdir shaderc
```

If discovery succeeds, remove only the generated `shaderc-sys` artifacts and retry the failing recipe:

```bash
cargo clean -p shaderc-sys
just test-unit
```

Do not remove the workspace `target/` directory. If discovery fails, install the shaderc development package instead of treating the CMake or `gmock` error as a VMNL test failure.

## Vulkan Loader or Driver Failure

Hypothesis: the Vulkan loader or GPU driver is unavailable.

```bash
vulkaninfo
```

- If `vulkaninfo` fails, repair the loader or driver before debugging VMNL.
- If it succeeds but an example fails, retry with `just build d2_shapes` and retain the first Vulkan error.
- Keep shaderc diagnosis separate: shader compilation discovery does not establish driver availability.

## GLFW or Display Failure

Hypothesis: GLFW cannot initialize a window or the display server is unavailable.

- Run `just test` to verify headless behavior.
- Compile display-dependent tests with `just test-gpu-compile`.
- Run `just test-gpu` only in a graphical session with Vulkan available.

Passing headless tests does not identify the cause of a GPU/window failure. Retain the first
GLFW/Vulkan error and distinguish unavailable runtime prerequisites from a VMNL backend defect.

## X11 Vulkan Surface Extension Mismatch

If initialization succeeds but window creation reports `VulkanSurfaceCreationFailed`, compare
the enabled instance extensions with the raw window handle used to create the surface. GLFW can
prefer `VK_KHR_xcb_surface` on X11 while its Rust raw-window-handle implementation exposes Xlib.
Vulkano's `Surface::from_window_ref` then requires `VK_KHR_xlib_surface` too. VMNL enables it when
GLFW requests XCB; the raw surface and instance extension selection must agree.

The earlier error mapping in `window/inner/surface.rs` discards the Vulkano cause, so the VMNL error
alone cannot distinguish an extension mismatch from a driver or native-handle failure. Inspect
the surface-creation validation path before changing the driver or the display configuration.
