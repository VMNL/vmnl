# Troubleshooting

Use the smallest command that exercises the failing layer.

## shaderc Not Found

Hypothesis: `shaderc` cannot be located.

```bash
pkg-config --exists shaderc
pkg-config --modversion shaderc
pkg-config --variable=libdir shaderc
```

If discovery succeeds, set `SHADERC_LIB_DIR` to the absolute directory reported by `pkg-config`. Otherwise, install the shaderc development package for the active distribution. See [Build](build.md#shaderc-discovery).

## Vulkan Loader or Driver Failure

Hypothesis: the Vulkan loader or GPU driver is unavailable.

```bash
vulkaninfo
```

- If `vulkaninfo` fails, repair the loader or driver before debugging VMNL.
- If it succeeds but an example fails, retry with `./run -b d2_shapes` and retain the first Vulkan error.
- Keep shaderc diagnosis separate: shader compilation discovery does not establish driver availability.

## GLFW or Display Failure

Hypothesis: GLFW cannot initialize a window or the display server is unavailable.

- Run `./run -t` to verify headless behavior.
- Compile display-dependent tests with `cargo test -p vmnl-gpu-tests --no-run`.
- Run `./run -gt` only in a graphical session with Vulkan available.

If headless tests pass and GPU tests cannot create a window, the failure is environmental rather than an API contract regression.
