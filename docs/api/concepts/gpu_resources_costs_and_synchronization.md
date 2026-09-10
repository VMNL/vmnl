# GPU resources, costs, and synchronization

`build(&Context)` on shapes, meshes, raw geometry, uniforms, and descriptor resources may allocate host and/or device resources and perform CPU-to-buffer writes. `BufferMemoryPreference` is a preference; current direct uploads require host-visible sequential-write memory.

`raw::Uniform::write` updates an existing uniform buffer directly. It does not wait for GPU work, rebuild descriptor sets, or allocate a replacement buffer, and it can fail when active CPU or GPU access conflicts with the write.

`raw::FrameUniform` initializes one uniform slot per current swapchain image. `FrameRenderer::write_frame_uniform` queues one CPU write, then `submit` allocates or reuses a fresh subbuffer for the acquired image slot before recording commands. Frame-uniform descriptor sets are allocated during command recording so they point at the current slot. It does not wait for GPU work or overwrite buffers that may still be read by queued GPU work. If the swapchain image count no longer matches the frame uniform resources, submission fails instead of rebuilding them implicitly.

Pipeline construction compiles shaders and creates Vulkan shader modules, layouts, and a graphics pipeline. Window creation builds surface/swapchain/render-pass state. Frame submission records commands, submits work, and presents through VMNL-owned synchronization.

No latency, throughput, allocation-count, batching, cache, queue-overlap, or synchronization-performance guarantee is specified. Resource destruction timing follows Rust ownership plus the underlying shared Vulkan objects; precise reclamation timing is not specified.
