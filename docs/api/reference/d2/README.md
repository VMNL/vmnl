# 2D API

The experimental 2D path is operational. Builders validate CPU geometry, allocate GPU buffers from a `Context`, and produce `Shape` values submitted with `FrameRenderer::draw2d`.

| Item | Role |
|---|---|
| [`Drawable2D`](drawable_2d.md) | Backend conversion contract |
| [`RenderItem2D`](render_item_2d.md) | Opaque backend draw descriptor |
| [`Vector2f`](vector_2f.md) | 2D scalar pair |
| [`Vertex2D`](vertex_2d.md) | Public position/color vertex |
| [Shapes](shapes/README.md) | GPU-backed shape and builders |
