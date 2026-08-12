# Workflows

These pages connect public items without duplicating the complete runnable programs under `examples/`. GPU/display snippets are compile-only (`no_run`). There is intentionally no 3D rendering workflow while that backend remains scaffolded.

| Goal | Workflow | Canonical example |
|---|---|---|
| Window/input | [Context and window](create_context_and_window.md), [event loop](window_event_loop.md), [input](keyboard_and_mouse_input.md) | [`events_input`](../../../examples/window/events_input/src/main.rs) |
| 2D | [Draw in 2D](draw_2d.md), [custom shaders](configure_custom_shaders.md) | [`shapes`](../../../examples/d2/shapes/src/main.rs) |
| Raw | [Pipeline](create_raw_pipeline.md), [geometry](create_raw_geometry.md), [uniforms](bind_raw_uniforms.md), [composition](compose_2d_and_raw.md) | [`raw`](../../../examples/raw/) |
| Failure | [Errors and shutdown](error_handling_and_shutdown.md) | Existing examples plus reference contracts |

