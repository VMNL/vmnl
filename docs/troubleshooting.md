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

If headless tests pass and GPU tests cannot create a window, the failure is environmental rather than an API contract regression.
