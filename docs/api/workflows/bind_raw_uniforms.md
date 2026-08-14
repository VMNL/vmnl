# Bind raw uniforms

1. Define a shader-compatible `#[repr(C)]` `Pod + Zeroable` data type.
2. Build `Uniform` from the same `Context` as the window/pipeline.
3. Start `Resources::builder(&pipeline)`.
4. Bind every reflected uniform block at its exact set/binding once.
5. Build resources from that same context and submit with `draw_raw_with`.

The current API uploads only an initial uniform value; it has no public in-place update method. Descriptor arrays, non-uniform resources, and push constants are unsupported.

Use the complete [`examples/raw/uniform`](../../../examples/raw/uniform/src/main.rs). See [`UniformBuilder`](../reference/raw/uniforms/uniform_builder.md) and [`ResourcesBuilder`](../reference/raw/resources/resources_builder.md).
