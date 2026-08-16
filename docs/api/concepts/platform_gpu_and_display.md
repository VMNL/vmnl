# Platform, GPU, and display

Context creation requires a working Vulkan loader, a compatible physical device, and the extensions/features selected by VMNL. Window creation additionally requires GLFW and a usable display/session. Headless API tests do not prove window creation, presentation, or visual output.

Linux is the active development platform. Windows and macOS CI coverage is configured but does not replace runtime Vulkan/display validation. Consult [platform support](../../platform_support.md) and [troubleshooting](../../troubleshooting.md) for canonical environment guidance.

GLFW portability and Vulkan/GPU support are separate contracts. `tests/platform` forces a GLFW
backend and creates a hidden `ClientApi::NoApi` window; it never creates a Vulkan instance or
surface. The generated [window compatibility matrix](../reference/window/platform_compatibility.md)
covers public VMNL operations. The exhaustive
[GLFW inventory](../maintenance/glfw_platform_inventory.md) also preserves known limitations for
platform-sensitive GLFW 3.4 functions that VMNL does not yet use.

Backend requests may succeed without producing the requested desktop effect. A callback error, a
no-op, a sentinel getter value, and a best-effort window-manager request are distinct outcomes.
Validation on Weston or Xvfb does not establish a universal Wayland or X11 guarantee.

GPU/display examples use `no_run`: documentation builds verify compilation but do not create a window or submit GPU work.
