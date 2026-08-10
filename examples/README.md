# VMNL Examples

Example policy lives in [docs/examples.md](../docs/examples.md).

Run examples from the repository root:

```bash
just run <example>
just build <example>
```

| Example | Command | Covers |
| --- | --- | --- |
| `d2_shapes` | `just run d2_shapes` | 2D shapes, anchors/origin, alpha, line caps, buffer preferences, render modes |
| `window_events_input` | `just run window_events_input` | window builder/config, polling, events, monitors, input, timers, lifecycle |
| `window_custom_shaders` | `just run window_custom_shaders` | 2D custom shaders from files; set `VMNL_INLINE_SHADERS=1` for inline shader strings |
| `raw_triangle` | `just run raw_triangle` | minimal raw pipeline triangle |
| `raw_pipeline` | `just run raw_pipeline` | raw shader paths, topology variants, blend modes, indexed/non-indexed geometry |
