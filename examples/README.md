# VMNL Examples

Example policy lives in [docs/examples.md](../docs/examples.md).

Run examples from the repository root:

```bash
just run <example>
just build <example>
```

| Example | Command | Covers |
| --- | --- | --- |
| `d2_shapes` | `just run d2_shapes` | minimal 2D rectangle, circle, and triangle rendering |
| `d2_circles` | `just run d2_circles` | circles, ellipses, rings, filled sectors, and an outlined elliptical arc with round caps |
| `d2_advanced_geometry` | `just run d2_advanced_geometry` | convex polygons , indices, vertex colors, transforms, line caps, polyline joins/caps/color modes, memory preferences, render modes |
| `joystick_runtime` | `just run joystick_runtime` | one-shot joystick/gamepad sampling through `InputRuntime` without a `Context`, Vulkan instance, or window; GLFW still requires a usable platform backend |
| `window_events_input` | `just run window_events_input` | window builder/config, polling, keyboard names/scancodes/sticky modes, events, monitors, mouse, joystick/gamepad snapshots and stick processing, cursor resources/modes/raw motion, timers, lifecycle |
| `window_custom_shaders` | `just run window_custom_shaders` | 2D custom shaders from files; set `VMNL_INLINE_SHADERS=1` for inline shader strings |
| `window_wait_events` | `just run window_wait_events` | explicit blocking event wait and event-driven redraw |
| `raw_triangle` | `just run raw_triangle` | minimal raw pipeline triangle |
| `raw_pipeline` | `just run raw_pipeline` | explicit `LargePoints` / `FillModeNonSolid` / `WideLines` requirements and line-width limits, raw shader paths, topology/blend variants, indexed/non-indexed geometry; `C` culling, `F` winding, `P` fill/wireframe, `W` line width |
| `raw_uniform` | `just run raw_uniform` | animated raw pipeline resources backed by `FrameUniform` |
| `raw_d2_composition` | `just run raw_d2_composition` | ordered 2D/raw passes; `V` full/fixed viewport, `S` full/fixed scissor; yellow clipping outline, framebuffer-pixel diagnostics, resize and state-isolation markers |

See the [current `d2_shapes` limitation](../docs/examples.md#commands) before running it.
