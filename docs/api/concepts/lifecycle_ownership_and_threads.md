# Lifecycle, ownership, and threads

`Context` owns the Vulkan instance, selected device, queue, and allocators. Windows and GPU resources retain shared internal Vulkan ownership, but logical compatibility is still checked where public APIs combine them.

`InputRuntime` can initialize GLFW without creating Vulkan resources. `InputRuntime`, `Context`,
windows, and native cursors share one internal GLFW owner. The first owner selects the joystick
hat-buttons initialization hint; later owners adopt it, and GLFW terminates after the last owner
drops. Acquire and drop these handles on GLFW's platform thread.

`Window` is mutable for event processing, state changes, and rendering. `Window::render()` borrows the window for the frame-builder lifetime; `FrameRenderer::submit()` consumes the builder and completes that frame submission attempt.

Input states belong to the window and are updated by `Window::poll_events`. One call is one batch:
held state survives across batches while press/release flags are cleared before pending events are
applied. Keyboard/mouse delivery filters do not disable their state tracking. Joystick state
sampling is a separate opt-in per window, independent from connection-event delivery; one window's
poll does not advance another window's snapshot or a standalone `InputRuntime` queue.

`Cursor` clones share one native handle through single-threaded `Rc` ownership. Each window retains
its assigned cursor until replacement, removal, or window destruction. The last owner destroys the
native resource while retaining the shared GLFW runtime; there is no explicit destroy method.

VMNL does not publish a general cross-thread guarantee for window, event-loop, renderer, or GPU-resource types. Treat them as confined to the creating thread unless Rust's auto-trait system and the platform backend permit a narrower use. Platform window systems may impose main-thread requirements.
