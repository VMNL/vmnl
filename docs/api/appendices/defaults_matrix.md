# Defaults matrix

| Surface | Default |
|---|---|
| `BufferMemoryPreference` | `Device` |
| `Rgba` | `(0, 0, 0, 0)`; named colors are opaque |
| `Vector2f`, `Vector3f`, `Vertex2D`, `Vertex3D` | Component/field zero defaults |
| `Camera` | position `(0,0,1)`, target `(0,0,0)`, up `(0,1,0)`; 3D remains scaffolded |
| `WindowBuilder` | title `VMNL Window`, `800x600`, automatic common polling, built-in 2D shaders, opaque black clear, no size limits, automatic present mode |
| `PresentMode` | `Auto`: `Mailbox` → `Immediate` → `FifoRelaxed` → `Fifo` among supported modes |
| `RenderMode` | `PerObject`; `Batched` currently falls back to it |
| `Input`, `KeyboardState`, `MouseState` | All states inactive |
| `Anchor` | `TopLeft` |
| `LineCap` | `Butt` |
| Rectangle | position `(0,0)`, white, rotation `0°`, top-left origin, device memory preference |
| Triangle | white per vertex unless created from colored vertices; device memory preference |
| Line | width `1.0`, butt cap, white, device memory preference |
| Indexed shape / mesh / raw geometry / uniform | device memory preference |
| `PipelineSpec` | shaders missing, `TriangleList`, `Opaque` |
| `GeometryBuilder` | no indices |
