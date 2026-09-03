# VMNL Examples

Example policy lives in [docs/examples.md](../docs/examples.md).

Run examples from the repository root:

```bash
just run <example>
just build <example>
```

| Example | Command | Covers |
| --- | --- | --- |
| `d2_shapes` | `just run d2_shapes` | minimal 2D rectangle and triangle rendering |
| `d2_advanced_geometry` | `just run d2_advanced_geometry` | indices, vertex colors, transforms, line caps, memory preferences, render modes |
| `window_events_input` | `just run window_events_input` | window builder/config, polling, events, monitors, input, timers, lifecycle |
| `window_custom_shaders` | `just run window_custom_shaders` | 2D custom shaders from files; set `VMNL_INLINE_SHADERS=1` for inline shader strings |
| `window_wait_events` | `just run window_wait_events` | explicit blocking event wait and event-driven redraw |
| `raw_triangle` | `just run raw_triangle` | minimal raw pipeline triangle |
| `raw_pipeline` | `just run raw_pipeline` | raw shader paths, topology variants, blend modes, indexed/non-indexed geometry |
| `raw_uniform` | `just run raw_uniform` | raw pipeline resources backed by a directly written uniform buffer |
| `raw_d2_composition` | `just run raw_d2_composition` | ordered 2D and alpha-blended raw passes in one frame |
