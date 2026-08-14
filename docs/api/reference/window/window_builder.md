# `WindowBuilder`

## Public path and maturity

Import path: `vmnl::WindowBuilder`; normally created by `Window::builder()`. Status: experimental.

## Purpose and use cases

Configures a native window, default 2D shaders, clear color, event polling, size limits, and swapchain presentation before allocation.

## Public API

| Method | Effect |
|---|---|
| `title(&str)` | Set UTF-8 native title. |
| `size(width, height)` | Set initial logical pixel size. |
| `unset_configure_window_polling()` | Disable default event-source configuration. |
| `size_limit(min_w, min_h, max_w, max_h)` | Validate and set optional per-axis limits. |
| `vertex_shader(source)` / `fragment_shader(source)` | Replace the built-in 2D shader stage. |
| `set_clear_color(color)` | Set normalized framebuffer clear color from `Into<Rgba>`. |
| `present_mode(mode)` | Require an explicit supported mode, or automatic selection. |
| `preferred_present_mode(mode)` | Prefer a mode and fall back to automatic selection. |
| `build(&Context)` | Validate and allocate a `Window`. |

The type implements `Default`.

## Construction, defaults, and validation

Defaults: title `VMNL Window`, size `800x600`, automatic polling enabled, no size limits, built-in 2D shaders, opaque black clear color, and `PresentMode::Auto`. Both initial dimensions must be at least 64. Minimum limits cannot exceed corresponding maximum limits.

## Units, coordinates, and valid ranges

Sizes/limits are logical screen pixels. Colors use `Rgba` channels. Titles are UTF-8 strings passed to the platform backend.

## Ownership, lifecycle, and threading

Setters consume/return the builder. Owned strings and shader sources move into window creation; `build` consumes the builder. Use on the window-creation thread.

## Errors, panics, and failure conditions

`size_limit` returns `InvalidWindowSize` for inverted limits. `build` can return invalid-size, GLFW, shader, surface, swapchain, render-pass, allocation, or unsupported-presentation errors.

## Allocation, transfers, synchronization, and GPU cost

Most setters only mutate owned CPU configuration. `title` allocates. `build` performs all native/Vulkan creation and shader compilation; exact cost is not specified.

## Platform, Vulkan, and display constraints

Supported present modes, surface formats, sizes, and shaders depend on the selected Vulkan device, window surface, driver, and display server.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, PresentMode, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let window = Window::builder()
        .title("VMNL")
        .size(1280, 720)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(&context)?;
    drop(window);
    Ok(())
}
```

Related: [`Window`](window.md), [`PresentMode`](present_mode.md), and [`ShaderSource`](../common/shader_source.md).
