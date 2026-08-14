# Lifecycle, ownership, and threads

`Context` owns the Vulkan instance, selected device, queue, and allocators. Windows and GPU resources retain shared internal Vulkan ownership, but logical compatibility is still checked where public APIs combine them.

`Window` is mutable for event processing, state changes, and rendering. `Window::render()` borrows the window for the frame-builder lifetime; `FrameRenderer::submit()` consumes the builder and completes that frame submission attempt.

Input states belong to the window and are updated by event processing. Their pressed/released transitions are frame-like snapshots and are reset by the event-processing path.

VMNL does not publish a general cross-thread guarantee for window, event-loop, renderer, or GPU-resource types. Treat them as confined to the creating thread unless Rust's auto-trait system and the platform backend permit a narrower use. Platform window systems may impose main-thread requirements.
