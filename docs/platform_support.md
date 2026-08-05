# Platform Support

## Validation Matrix

| Platform | CI validation | `./run` support | Status |
|----------|---------------|-----------------|--------|
| Ubuntu Linux | Configured on `ubuntu-24.04`; first matrix run pending. | Yes | Supported development path. |
| Other Linux distributions | No | Package installation paths are provided. | Best effort. |
| Windows | Configured on `windows-2022`; first matrix run pending. | No | Cargo is invoked directly by CI. |
| macOS | Configured on `macos-14`; first matrix run pending. | No | Cargo is invoked directly by CI. |

The `./run` script requires `/etc/os-release` for dependency installation and only contains Linux package-manager paths. It is a local development helper and is never invoked by CI.

## Runtime Constraints

- Visual examples and GPU tests require a Vulkan-capable GPU, a Vulkan loader, GLFW, and a display server.
- Headless verification uses `./run -t`; it excludes GPU/display tests.
- Compile GPU tests without a display with `cargo test -p vmnl-gpu-tests --no-run`.

Platform badges in the root README describe project targets, not a guarantee of CI validation.
