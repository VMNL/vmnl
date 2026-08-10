# Platform Support

## Validation Matrix

| Platform | CI validation | Local Justfile | Status |
|----------|---------------|----------------|--------|
| Ubuntu Linux | Configured on `ubuntu-24.04`; first matrix run pending. | Yes | Supported development path. |
| Other Linux distributions | No | Best effort. | Package installation paths are provided. |
| Windows | Configured on `windows-2022`; first matrix run pending. | Not validated. | CI invokes Cargo directly. |
| macOS | Configured on `macos-14`; first matrix run pending. | Not validated. | CI invokes Cargo directly. |

`just bootstrap` invokes `./deps`, which requires `/etc/os-release` and only contains Linux
package-manager paths. CI invokes Cargo directly and never invokes the bootstrap recipe.

## Runtime Constraints

- Visual examples and GPU tests require a Vulkan-capable GPU, a Vulkan loader, GLFW, and a display server.
- Headless verification uses `just test`; it excludes GPU/display tests.
- Compile GPU tests without a display with `just test-gpu-compile`.

Platform badges in the root README describe project targets, not a guarantee of CI validation.
