# Bind raw uniforms

Use `Uniform<T>` for stable data or one-off direct writes. Use `FrameUniform<T>` for data that changes every frame.

Direct write workflow:

1. Define a shader-compatible `#[repr(C)]` `Pod + Zeroable` data type.
2. Build `Uniform` from the same `Context` as the window/pipeline.
3. Bind it with `Resources::builder(&pipeline).uniform(set, binding, &uniform)`.
4. Optionally call `Uniform::write` before the submit that should observe the new data.
5. Submit with `draw_raw_with`.

Frame-varying workflow:

1. Build `FrameUniform` with `FrameUniform::builder(initial).build(&window)`.
2. Bind it once with `Resources::builder(&pipeline).frame_uniform(set, binding, &uniform)`.
3. In each frame, call `write_frame_uniform(&mut uniform, data)` before `submit`.
4. Submit with `draw_raw_with` using the same `Resources`.

`Uniform::write` updates one existing buffer directly and can fail if it conflicts with active CPU or GPU access. `FrameUniform` keeps one current slot per swapchain image; `write_frame_uniform` allocates or reuses a fresh subbuffer for the acquired image slot during submit. All queued frame-uniform writes happen before command recording, not between draw passes. Frame-uniform descriptor sets are allocated during command recording so they point at the current slot. It does not wait for GPU work or overwrite a buffer that may still be read by queued GPU work. Descriptor arrays, non-uniform resources, storage buffers, textures/samplers, and push constants are unsupported.

Use the complete [`examples/raw/uniform`](../../../examples/raw/uniform/src/main.rs). See [`Uniform`](../reference/raw/uniforms/uniform.md), [`FrameUniform`](../reference/raw/uniforms/frame_uniform.md), and [`ResourcesBuilder`](../reference/raw/resources/resources_builder.md).
