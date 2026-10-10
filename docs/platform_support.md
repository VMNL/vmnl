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
| Ubuntu Linux | Configured: build, headless tests, GLFW Null, Weston Wayland nested in Xvfb/Openbox and Xvfb/Openbox X11, with XTEST keyboard, mapped mouse buttons, pointer movement/leave/enter, and vertical/horizontal scroll. Hosted Xvfb's 10-button map reaches GLFW buttons 1–6; buttons 7–8 remain unsupported there. | `just input-test x11`, `just input-test wayland`, and opt-in GPU-backed `just input-test-vmnl x11`. | Blocking CI path for NoApi probes; public VMNL runtime still requires a qualified Vulkan/display run. |
| Other Linux distributions | No distribution matrix. | Best effort. | Backend guarantees remain environment-scoped. |
| Windows | Configured: build, headless tests and GLFW Null; visible keyboard, GLFW mouse buttons 1–5, pointer movement, leave/enter, and vertical/horizontal scroll probes use `SendInput`/`SetCursorPos` and are experimental. GLFW buttons 6–8 have no Win32 mapping. | `just input-test win32` on an accessible interactive desktop. | Current workflow run required; native results are non-blocking until qualified. |
| macOS | Configured: build, headless tests and GLFW Null; visible keyboard, all eight GLFW mouse buttons, pointer movement, leave/enter, and vertical/horizontal scroll probes use `CGEventPost` and are experimental. | `just input-test cocoa` with event-posting access. | Current workflow run required; native results are non-blocking until qualified. |

`just bootstrap` invokes `./deps`, which requires `/etc/os-release` and only contains Linux
package-manager paths. CI invokes Cargo directly and never invokes the bootstrap recipe.

## Runtime Constraints

- Visual examples and GPU tests require a Vulkan-capable GPU, a Vulkan loader, GLFW, and a display server.
- Headless verification uses `just test`; it excludes GPU/display tests.
- GLFW portability probes use `ClientApi::NoApi`; they create no Vulkan instance, surface, or GPU resource.
- Native keyboard probes require a focused visible window. Mouse probes require a hovered window
  before injection. X11 and nested Weston inject every GLFW button mapping within the active X11
  pointer map, plus movement, leave/enter, and vertical/horizontal scroll. The hosted 10-button
  Xvfb map cannot exercise GLFW buttons 7–8. Win32 injects buttons 1–5; Cocoa injects all eight.
  Both also probe movement, leave/enter, and vertical/horizontal scroll; these Windows/macOS results
  still need CI evidence.
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
