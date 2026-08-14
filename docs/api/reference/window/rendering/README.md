# Rendering

| Item | Role |
|---|---|
| [`FrameRenderer`](frame_renderer.md) | Ordered per-frame pass builder and submission |
| [`RenderMode`](render_mode.md) | Per-object/batched submission request |

`Window::render()` creates a `FrameRenderer`. 2D and raw passes are operational. 3D passes can be recorded, but submission fails because the 3D backend is scaffolded.
