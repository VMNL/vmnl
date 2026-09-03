# Bind raw uniforms

1. Define a shader-compatible `#[repr(C)]` `Pod + Zeroable` data type.
2. Build `Uniform` from the same `Context` as the window/pipeline.
3. Start `Resources::builder(&pipeline)`.
4. Bind every reflected uniform block at its exact set/binding once.
5. Build resources from that same context.
6. Optionally call `Uniform::write` before the submit that should observe the new data.
7. Submit with `draw_raw_with`.

`Uniform::write` updates the same buffer referenced by already-built resources. It does not rebuild descriptor sets, submit GPU work, or wait for in-flight GPU access; it returns an error if the write conflicts with active CPU or GPU access. Descriptor arrays, non-uniform resources, and push constants are unsupported.

Use the complete [`examples/raw/uniform`](../../../examples/raw/uniform/src/main.rs). See [`UniformBuilder`](../reference/raw/uniforms/uniform_builder.md) and [`ResourcesBuilder`](../reference/raw/resources/resources_builder.md).
