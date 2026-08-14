# Feature status

| Capability | Status | Evidence boundary |
|---|---|---|
| Context/device initialization | Operational, experimental | GPU/display runtime required for execution |
| Native window, events, input, monitors | Operational, experimental | GLFW/display runtime required |
| 2D predefined/indexed shapes | Operational, experimental | Unit/API checks plus GPU/examples for runtime visuals |
| Custom 2D shaders | Operational, experimental | Shader compile and visual example |
| Ordered 2D/raw frame composition | Operational, experimental | GPU/example behavior |
| Raw typed pipeline/geometry | Operational, experimental | Restricted public contract |
| Raw uniform buffers/resources | Operational, experimental | Single uniform-buffer descriptors only |
| Raw textures/samplers/storage/descriptors arrays/push constants | Unsupported | Rejected by pipeline/resource validation |
| `RenderMode::Batched` optimization | Not implemented | Falls back to `PerObject` |
| 3D camera/vertex/mesh types | Scaffolded | Data/resource construction only |
| 3D frame rendering | Not implemented | `submit` returns explicit `InvalidState` |

Compilation, doctests, and headless tests do not prove Vulkan execution or correct pixels. No performance/synchronization claim beyond documented behavior is specified.
