# Platform Support

## Validation Matrix

| Platform | CI validation | Local Justfile | Status |
|----------|---------------|----------------|--------|
| Ubuntu Linux | Configured: build, headless tests, GLFW Null, Weston/Wayland and Xvfb/Openbox X11. | Yes | Blocking CI path; current workflow run required for evidence. |
| Other Linux distributions | No distribution matrix. | Best effort. | Backend guarantees remain environment-scoped. |
| Windows | Configured: build, headless tests and GLFW Null; native Win32 probe is experimental. | Compile and Null only unless run locally. | Current workflow run required; native results are non-blocking until qualified. |
| macOS | Configured: build, headless tests and GLFW Null; native Cocoa probe is experimental. | Compile and Null only unless run locally. | Current workflow run required; Cocoa executes from `main` and remains non-blocking. |

`just bootstrap` invokes `./deps`, which requires `/etc/os-release` and only contains Linux
package-manager paths. CI invokes Cargo directly and never invokes the bootstrap recipe.

## Runtime Constraints

- Visual examples and GPU tests require a Vulkan-capable GPU, a Vulkan loader, GLFW, and a display server.
- Headless verification uses `just test`; it excludes GPU/display tests.
- GLFW portability probes use `ClientApi::NoApi`; they create no Vulkan instance, surface, or GPU resource.
- Compile GPU tests without a display with `just test-gpu-compile`.

The generated [window compatibility matrix](api/reference/window/platform_compatibility.md) is
canonical for public VMNL operations. The exhaustive
[GLFW inventory](api/maintenance/glfw_platform_inventory.md) also records platform-sensitive GLFW
3.4 functions not currently used by VMNL. A successful Weston, Xvfb, Win32, or Cocoa probe only
qualifies the exact backend and recorded runner environment.

Platform badges in the root README describe project targets, not a guarantee of CI validation.
