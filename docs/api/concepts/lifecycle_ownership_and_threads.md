# Lifecycle, ownership, and threads

`Context` owns the Vulkan instance, selected device, queue, and allocators. Windows and GPU resources retain shared internal Vulkan ownership, but logical compatibility is still checked where public APIs combine them.

`Window` is mutable for event processing, state changes, and rendering. `Window::render()` borrows the window for the frame-builder lifetime; `FrameRenderer::submit()` consumes the builder and completes that frame submission attempt.

Input states belong to the window and are updated by `Window::poll_events`. One call is one batch: held state survives across batches while press/release flags are cleared before pending events are applied. Public event delivery can be disabled without disabling keyboard/mouse-button tracking.

`Cursor` clones share one native handle through single-threaded `Rc` ownership. Each window retains
its assigned cursor until replacement, removal, or window destruction. The last owner destroys the
native resource while its retained GLFW token is still alive; there is no explicit destroy method.

VMNL does not publish a general cross-thread guarantee for window, event-loop, renderer, or GPU-resource types. Treat them as confined to the creating thread unless Rust's auto-trait system and the platform backend permit a narrower use. Platform window systems may impose main-thread requirements.
