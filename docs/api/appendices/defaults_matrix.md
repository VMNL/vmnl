# Defaults matrix

| Surface | Default |
|---|---|
| `BufferMemoryPreference` | `Device` |
| `ContextBuilder` / `DeviceConfig` | automatic compatible GPU/graphics queue selection; no optional feature request |
| `Rgba` | `(0, 0, 0, 0)`; named colors are opaque |
| `Vector2f`, `Vector3f`, `Vertex2D`, `Vertex3D` | Component/field zero defaults |
| `Camera` | position `(0,0,1)`, target `(0,0,0)`, up `(0,1,0)`; 3D remains scaffolded |
| `WindowBuilder` | title `VMNL Window`, `800x600`, common event delivery enabled, joystick event delivery/tracking disabled, zero-dead-zone `Math2D` left/right sticks, built-in 2D shaders, opaque black clear, no size limits, automatic present mode |
| `PresentMode` | `Auto`: `Mailbox` → `Immediate` → `FifoRelaxed` → `Fifo` among supported modes |
| Resolved window attachments | one color attachment, automatic `B8G8R8A8_SRGB` + sRGB nonlinear preference then first surface format fallback; one sample per pixel; no MSAA selection |
| `RenderTargetConfig` / `WindowBuilder::render_target` | color only; no depth image; clear depth `1.0` |
| `RenderMode` | `PerObject`; `Batched` currently falls back to it |
| `Input`, `KeyboardState`, `MouseState` | Keyboard and mouse states inactive; each joystick slot is `NotTracked` |
| Window sticky keys / sticky mouse buttons / lock-key modifier reporting | Disabled |
| `CursorMode` / new window cursor | `Normal`; backend default cursor; raw mouse motion disabled |
| `CursorBuilder` | hotspot `(0, 0)`; diagnostic hotspot marker disabled |
| `Modifiers` | `NONE` |
| `Anchor` | `TopLeft` |
| `LineCap` | `Butt` |
| `LineJoin` | `Bevel` |
| Polyline | open; width `1.0`; butt cap; bevel join; miter limit `4.0`; opaque white; device memory preference |
| Polygon | ordered simple convex or concave boundary; white from points, supplied colors from vertices; device memory preference; implicit closure; convex triangle fan or deterministic concave ear clipping |
| Rectangle | position `(0,0)`, white, rotation `0°`, top-left origin, sharp corners, `8` corner segments, device memory preference |
| Circle / ellipse through `Shape::circle` | equal required radii until `.ellipse(ry)`; center `(0,0)`, filled, white, 32 segments, inner/outer offsets `0.5`, no sector or cap, device memory preference; outer offset applies even when filled |
| Triangle | white per vertex unless created from colored vertices; device memory preference |
| Line | width `1.0`, butt cap, white, device memory preference |
| Indexed shape / mesh / raw geometry / uniform | device memory preference |
| `PipelineSpec` | shaders missing, `TriangleList`, `Opaque`, `CullMode::None`, `FrontFace::CounterClockwise`, `PolygonMode::Fill`, line width `1.0`, viewport/scissor `FullFramebuffer`, depth `Disabled` |
| `ViewportPolicy` / `ScissorPolicy` | `FullFramebuffer`, resolved against the acquired image extent on each raw draw |
| `GeometryBuilder` | no indices |
