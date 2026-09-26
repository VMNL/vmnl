# Defaults matrix

| Surface | Default |
|---|---|
| `BufferMemoryPreference` | `Device` |
| `Rgba` | `(0, 0, 0, 0)`; named colors are opaque |
| `Vector2f`, `Vector3f`, `Vertex2D`, `Vertex3D` | Component/field zero defaults |
| `Camera` | position `(0,0,1)`, target `(0,0,0)`, up `(0,1,0)`; 3D remains scaffolded |
| `WindowBuilder` | title `VMNL Window`, `800x600`, common event delivery enabled, built-in 2D shaders, opaque black clear, no size limits, automatic present mode |
| `PresentMode` | `Auto`: `Mailbox` → `Immediate` → `FifoRelaxed` → `Fifo` among supported modes |
| `RenderMode` | `PerObject`; `Batched` currently falls back to it |
| `Input`, `KeyboardState`, `MouseState` | All states inactive |
| Window sticky keys / sticky mouse buttons / lock-key modifier reporting | Disabled |
| `CursorMode` / new window cursor | `Normal`; backend default cursor; raw mouse motion disabled |
| `CursorBuilder` | hotspot `(0, 0)`; diagnostic hotspot marker disabled |
| `Modifiers` | `NONE` |
| `Anchor` | `TopLeft` |
| `LineCap` | `Butt` |
| `LineJoin` | `Bevel` |
| Polyline | open; width `1.0`; butt cap; bevel join; miter limit `4.0`; opaque white; device memory preference |
| Rectangle | position `(0,0)`, white, rotation `0°`, top-left origin, device memory preference |
| Circle | center `(0,0)`, white, 32-triangle tessellation, device memory preference |
| Ellipse | center `(0,0)`, white, 32-triangle tessellation, device memory preference |
| Triangle | white per vertex unless created from colored vertices; device memory preference |
| Line | width `1.0`, butt cap, white, device memory preference |
| Indexed shape / mesh / raw geometry / uniform | device memory preference |
| `PipelineSpec` | shaders missing, `TriangleList`, `Opaque` |
| `GeometryBuilder` | no indices |
