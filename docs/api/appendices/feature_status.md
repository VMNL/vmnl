# Feature status

Capabilities of `vmnl-graphics 0.1.0`. Versioning does not qualify a native platform or complete a scaffolded feature.

| Capability | Status | Evidence boundary |
|---|---|---|
| Context/device initialization | Operational in 0.1.0 | GPU/display runtime required for execution |
| Native window, events, input, monitors | Operational in 0.1.0 | GLFW/display runtime required |
| 2D predefined/indexed shapes | Operational in 0.1.0 | Unit/API checks plus GPU/examples for runtime visuals |
| Custom 2D shaders | Operational in 0.1.0 | Shader compile and visual example |
| Ordered 2D/raw frame composition | Operational in 0.1.0 | GPU/example behavior |
| Raw typed pipeline/geometry | Operational in 0.1.0 | Restricted public contract |
| Raw uniform buffers/resources | Operational in 0.1.0 | Single uniform-buffer descriptors only |
| Render-target inspection / preflight | Operational in 0.1.0 | Actual color/depth formats and samples; strict originating-window identity; resize preserves validity |
| Optional window depth attachments / raw depth tests | Operational in 0.1.0 | Explicit D16/D32, per-image ownership and resize; compare/write state on raw pipelines; high-level 3D remains scaffolded |
| Raw textures/samplers/storage/descriptors arrays/push constants | Unsupported | Rejected by pipeline/resource validation |
| `RenderMode::Batched` optimization | Not implemented | Falls back to `PerObject` |
| 3D camera/vertex/mesh types | Scaffolded | Data/resource construction only |
| 3D frame rendering | Not implemented | `submit` returns explicit `InvalidState` |

Compilation, doctests, and headless tests do not prove Vulkan execution or correct pixels. No performance/synchronization claim beyond documented behavior is specified.
