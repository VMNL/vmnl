# `Window`

## Public path and maturity

Import path: `vmnl::Window`. Status: experimental, operational window/render target.

## Purpose and use cases

Owns a native GLFW window plus the Vulkan surface, swapchain, render pass, synchronization state, input snapshot, and monitor snapshot required by VMNL.

## Public API

Constructors are `new(&Context)` and `builder()`. The remaining methods are partitioned by responsibility: [configuration](configuration.md), [lifecycle](lifecycle.md), [polling](polling.md), [event processing and timers](events/event_processing_and_timers.md), and [rendering](rendering/README.md).

## Construction, defaults, and validation

`new` is equivalent to `Window::builder().build(context)`. See [`WindowBuilder`](window_builder.md) for defaults and validation.

## Units, coordinates, and valid ranges

Logical window and framebuffer sizes are pixels but may differ under content scaling. Positions are virtual-screen coordinates; timing methods use seconds or platform timer ticks.

## Ownership, lifecycle, and threading

`Window` owns mutable platform and frame state and is tied to its creating context/device. It is not cloneable. Keep it on the platform-compatible creating thread; cross-thread use is not specified.

## Errors, panics, and failure conditions

Creation and fallible configuration/render methods return `VMNLResult`. Platform calls without a result may report errors through GLFW's callback. No public intentional-panic contract is specified.

## Allocation, transfers, synchronization, and GPU cost

Creation allocates native and Vulkan surface/swapchain/render-target/synchronization resources. Rendering acquires, records, submits, and presents. Exact costs and scheduling are not specified.

## Platform, Vulkan, and display constraints

Requires a compatible `Context`, GLFW, Vulkan presentation support, and a usable display/session. Minimized/zero-sized framebuffers are temporarily unrenderable.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    while window.is_open() {
        window.poll_events();
        window.render().submit()?;
    }
    Ok(())
}
```

Related: [`Context`](../context.md), [`WindowBuilder`](window_builder.md), and [`FrameRenderer`](rendering/frame_renderer.md).

