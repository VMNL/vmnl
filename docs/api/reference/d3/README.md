# 3D API — scaffolded

All types in this section exist, and meshes can allocate GPU buffers, but 3D rendering is not operational. `FrameRenderer::draw3d` records a pass; `submit` then returns `InvalidState("3D rendering is not implemented yet")` before acquiring a swapchain image.

| Item | Role |
|---|---|
| [`Camera`](camera.md) | Stored camera parameters; no matrices are produced |
| [`Drawable3D`](drawable_3d.md) | Scaffolded backend conversion contract |
| [`RenderItem3D`](render_item_3d.md) | Opaque scaffolded draw descriptor |
| [`Vector3f`](vector_3f.md) | 3D scalar tuple |
| [`Vertex3D`](vertex_3d.md) | Position/color vertex |
| [Meshes](mesh/README.md) | GPU-backed indexed mesh scaffolding |

This status is part of every 3D item contract below; no operational 3D workflow is provided.
