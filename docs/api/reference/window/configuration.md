# `Window` configuration methods

## Public path and maturity

Methods on `vmnl::Window`. Status: experimental, operational.

## Purpose and use cases

Reads and updates native window properties and the framebuffer clear color.

## Public API

| Methods | Contract |
|---|---|
| `get_title`, `set_title` | Read/set title. |
| `set_size`, `get_size`, `width`, `height` | Set/read logical size. |
| `get_framebuffer_size`, `get_content_scale` | Read renderable pixel size and scale. |
| `set_size_limits` | Set/remove optional per-axis constraints. |
| `set_aspect_ratio` | Set `(numerator, denominator)` or remove with `None`. |
| `set_position`, `get_position` | Set/read virtual-screen position. |
| `opacity`, `get_opacity` | Set/read native window opacity. |
| `monitor` | Borrow the creation-time `Monitors` snapshot. |
| `set_clear_color` | Set the color used by subsequent frame clears. |

## Construction, defaults, and validation

Logical dimensions below 64 or outside GLFW's integer range are rejected by `set_size`. A minimum limit cannot exceed its maximum. Aspect terms must be non-zero and fit GLFW's integer representation; `None` maps to GLFW's unconstrained sentinel. The public opacity setter forwards the value; valid portable input is `0.0..=1.0`.

## Units, coordinates, and valid ranges

Sizes/framebuffers are pixels. Position uses GLFW screen coordinates and is not necessarily in
physical pixels. Content scale is dimensionless, and opacity is `0.0..=1.0`. Framebuffer size may
differ from logical size under DPI scaling.

## Ownership, lifecycle, and threading

Mutators require `&mut Window`; getters borrow cached or native state. `monitor()` borrows a snapshot owned by the window.

## Errors, panics, and failure conditions

Fallible setters return `InvalidWindowSize` or `InvalidState`. Infallible platform setters expose
no typed result. Unsupported operations reach the configured callback as
`GlfwUnsupportedPlatform` and have no effect. A position getter may return `(0, 0)` and an opacity
getter may return `1.0` when the backend cannot provide the property.

## Allocation, transfers, synchronization, and GPU cost

Title changes may allocate. Size changes mark swapchain-dependent state for recreation on submission. Clear-color changes update CPU state; exact synchronization cost is not specified.

## Platform, Vulkan, and display constraints

Opacity, position, limits, content scale, and window-manager behavior are platform dependent.
Wayland does not expose global window positions and GLFW 3.4 does not provide whole-window opacity
there. X11 placement depends on the active window manager. A zero-sized framebuffer cannot be
submitted until renderable again. See the generated
[platform compatibility matrix](platform_compatibility.md).

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    window.set_size(1024, 768)?;
    window.set_aspect_ratio(Some((4, 3)))?;
    window.set_clear_color([20, 24, 32]);
    Ok(())
}
```

Related: [`Rgba`](../common/rgba.md) and [monitors](monitors/README.md).
