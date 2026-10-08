# Create a raw pipeline

1. Define a `#[repr(C)]` vertex type and derive `Pod`, `Zeroable`, and `Vertex`.
2. Annotate every vertex field with its Vulkan `#[format(...)]` and optional shader `#[name(...)]`.
3. Build a window first because the pipeline is bound to its device/render pass.
4. Supply both shader stages and optionally topology, blending, culling, front-face winding, polygon mode, and line width. Require `FillModeNonSolid` / `WideLines` through `DeviceConfig` before context creation when needed; inspect `Context::line_width_limits()` to choose a valid width.
5. Build and retain the pipeline for compatible geometry submissions.

The canonical implementation is [`examples/raw/pipeline`](../../../examples/raw/pipeline/src/main.rs); do not copy its full shaders into documentation. Review [`PipelineSpec`](../reference/raw/pipeline/pipeline_spec.md) and [shader/layout safety](../concepts/shaders_vertex_layouts_and_safety.md).

Current layout limits: only descriptor-count-one uniform buffers, no descriptor arrays, no other descriptor types, and no push constants.

## Inspect triangle culling

Run `just run raw_pipeline`. The square on the right contains two triangles with opposite windings. Pipelines for all demonstrated culling/winding/polygon-mode/width combinations are built at startup and reused; key presses select an existing pipeline.

- `C` cycles `None`, `Front`, `Back`, and `FrontAndBack`.
- `F` switches `CounterClockwise` and `Clockwise`; the title shows the current selection.
- With `None`, both square halves remain visible. With `Front` or `Back`, only one half remains; switching the winding swaps the visible half. With `FrontAndBack`, the square disappears.
- `P` toggles `Fill` / `Line` for the square. In wireframe, both triangles' edges appear, including the shared diagonal; culling still removes the corresponding triangle. This does not extract a shape's outer contour.
- `W` toggles line widths chosen from the GPU's range. It affects the line list, line strip and square edges in wireframe; filled triangle surfaces and point size do not change. The title and startup output show the requested widths, not measured thickness. A GPU with only unit-width support has a single choice.
- Points and the triangle strip retain their separate default pipelines throughout.
- Resize the window, then repeat `P` / `W` / `C` / `F`; the selected state must persist. All combinations are prebuilt at startup, so key presses do not create pipelines. The example strictly requests `LargePoints`, `FillModeNonSolid` and `WideLines`; initialization fails if no compatible GPU provides all three. Driver rounding and coverage rules may make widths appear differently across GPUs.

An operator should check these observations and run the existing `raw_triangle`, `raw_uniform`, and `raw_d2_composition` examples for visible regressions. GPU submission tests do not verify the displayed pixels.
