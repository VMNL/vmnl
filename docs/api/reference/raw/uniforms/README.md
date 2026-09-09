# Raw uniforms

[`Uniform<TData>`](uniform.md) owns one typed uniform buffer for direct, explicit writes.
[`FrameUniform<TData>`](frame_uniform.md) owns one current uniform slot per swapchain image for
frame-varying data such as animation, tint, offset, time, or camera data.
