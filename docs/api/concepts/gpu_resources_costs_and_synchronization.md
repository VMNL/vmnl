# GPU resources, costs, and synchronization

`build(&Context)` on shapes, meshes, raw geometry, uniforms, and descriptor resources may allocate host and/or device resources and perform CPU-to-buffer writes. `BufferMemoryPreference` is a preference; current direct uploads require host-visible sequential-write memory.

`raw::Uniform::write` updates an existing uniform buffer directly. It does not wait for GPU work, rebuild descriptor sets, or allocate a replacement buffer, and it can fail when active CPU or GPU access conflicts with the write.

Pipeline construction compiles shaders and creates Vulkan shader modules, layouts, and a graphics pipeline. Window creation builds surface/swapchain/render-pass state. Frame submission records commands, submits work, and presents through VMNL-owned synchronization.

No latency, throughput, allocation-count, batching, cache, queue-overlap, or synchronization-performance guarantee is specified. Resource destruction timing follows Rust ownership plus the underlying shared Vulkan objects; precise reclamation timing is not specified.
