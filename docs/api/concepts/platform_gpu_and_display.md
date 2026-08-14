# Platform, GPU, and display

Context creation requires a working Vulkan loader, a compatible physical device, and the extensions/features selected by VMNL. Window creation additionally requires GLFW and a usable display/session. Headless API tests do not prove window creation, presentation, or visual output.

Linux is the active development platform. Windows and macOS CI coverage is configured but does not replace runtime Vulkan/display validation. Consult [platform support](../../platform_support.md) and [troubleshooting](../../troubleshooting.md) for canonical environment guidance.

GPU/display examples use `no_run`: documentation builds verify compilation but do not create a window or submit GPU work.
