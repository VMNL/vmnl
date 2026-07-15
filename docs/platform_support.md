# Platform Support

## Validation Matrix

| Platform | CI validation | `./run` support | Status |
|----------|---------------|-----------------|--------|
| Ubuntu Linux | Yes | Yes | Supported development path. |
| Other Linux distributions | No | Package installation paths are provided. | Best effort. |
| Windows | No | No | Not validated. Use Cargo and native dependencies manually. |
| macOS | No | No | Not validated. Use Cargo and native dependencies manually. |

The `./run` script requires `/etc/os-release` for dependency installation and only contains Linux package-manager paths.

## Runtime Constraints

- Visual examples and GPU tests require a Vulkan-capable GPU, a Vulkan loader, GLFW, and a display server.
- Headless verification uses `./run -t`; it excludes GPU/display tests.
- Compile GPU tests without a display with `cargo test -p vmnl-gpu-tests --no-run`.

Platform badges in the root README describe project targets, not a guarantee of CI validation.
