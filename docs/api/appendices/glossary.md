# Glossary

| Term | Meaning in VMNL |
|---|---|
| Facade | The client dependency boundary exposed by crate `vmnl`. |
| Context | Shared single-threaded owner of Vulkan device/queue/allocators. |
| Window | Native window plus Vulkan presentation/render-target state. |
| Logical pass | One ordered `draw2d`, `draw3d`, `draw_raw_2d`, or `draw_raw_2d_with` addition. |
| Shape | VMNL-provided GPU-backed 2D drawable. |
| Raw | Typed custom pipeline/geometry/resource layer below predefined 2D shapes. |
| Buffer contents | Type whose bytes can be copied into a Vulkano buffer. |
| Vertex layout | Mapping between Rust fields and shader vertex inputs/formats. |
| Uniform | Typed uniform-buffer value that can be updated through a direct, fallible write. |
| FrameUniform | Typed uniform-buffer state with one current slot per swapchain image for frame-varying raw shader data. |
| Resources | Descriptor resources built for one raw pipeline layout. |
| Present mode | Swapchain image presentation/pacing policy. |
| Scaffolded | Public data/API wiring exists, but successful rendering is intentionally unavailable. |
| Headless evidence | Compile/test evidence that does not create a window or validate pixels. |
