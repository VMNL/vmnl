# Platform Support

## Support policy

- Linux is the tier-1 target. Ubuntu is the current blocking CI environment; other distributions
  require environment-specific evidence.
- Windows and macOS are tier-2 targets until their native probes and release artifacts are
  qualified.
- Mobile and Web are outside the VMNL scope.
- Platform support applies only to the domains and backends explicitly exercised on that platform.

## Validation Matrix

| Platform | CI validation | Local Justfile | Status |
|----------|---------------|----------------|--------|
| Ubuntu Linux | Configured: build, headless tests, GLFW Null, Weston Wayland nested in Xvfb/Openbox and Xvfb/Openbox X11, including XTEST keyboard, all eight GLFW mouse buttons, and vertical/horizontal scroll. | `just input-test x11`, `just input-test wayland`, and opt-in GPU-backed `just input-test-vmnl x11`. | Blocking CI path for NoApi probes; public VMNL runtime still requires a qualified Vulkan/display run. |
| Other Linux distributions | No distribution matrix. | Best effort. | Backend guarantees remain environment-scoped. |
| Windows | Configured: build, headless tests and GLFW Null; visible keyboard and left-button probes with `SendInput` are experimental. | `just input-test win32` on an accessible interactive desktop. | Current workflow run required; native results are non-blocking until qualified. |
| macOS | Configured: build, headless tests and GLFW Null; visible keyboard and left-button probes with `CGEventPost` are experimental. | `just input-test cocoa` with event-posting access. | Current workflow run required; native results are non-blocking until qualified. |

`just bootstrap` invokes `./deps`, which requires `/etc/os-release` and only contains Linux
package-manager paths. CI invokes Cargo directly and never invokes the bootstrap recipe.

## Runtime Constraints

- Visual examples and GPU tests require a Vulkan-capable GPU, a Vulkan loader, GLFW, and a display server.
- Headless verification uses `just test`; it excludes GPU/display tests.
- GLFW portability probes use `ClientApi::NoApi`; they create no Vulkan instance, surface, or GPU resource.
- Native keyboard probes require a focused visible window. Mouse probes also require pointer hover
  over that window. Linux X11 and nested Weston synthesize all eight GLFW mouse buttons through
  XTEST server buttons 1–3 and 8–12, plus vertical and horizontal scroll through buttons 4–7.
  Windows and macOS mouse probes currently synthesize only the left button.
  X11 requires XTEST and an EWMH window manager, Windows requires an accessible desktop at the
  same integrity level, and macOS requires event-posting access.
- The nested Weston probe covers XTEST → Xvfb → Weston X11 backend → Wayland client input, not a
  native Wayland `libei`, portal, or `uinput` seat.
- Compile GPU tests without a display with `just test-gpu-compile`.

The generated [window compatibility matrix](api/reference/window/platform_compatibility.md) is
canonical for public VMNL operations. The exhaustive
[GLFW inventory](api/maintenance/glfw_platform_inventory.md) also records platform-sensitive GLFW
3.4 functions not currently used by VMNL. A successful Weston, Xvfb, Win32, or Cocoa probe only
qualifies the exact backend and recorded runner environment.

Platform badges in the root README describe project targets, not a guarantee of CI validation.
