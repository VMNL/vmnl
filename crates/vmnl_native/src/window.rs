extern crate glfw;
use crate::vmnl_instance::{VMNLInstance};
use crate::{
    Graphics, VMNLContext, VMNLError, VMNLResult, VMNLVertex
};
use glfw::{
    Action,
    Key
};
use vulkano::instance::{Instance};
use vulkano::device::Device;
use std::sync::Arc;
use vulkano::Validated;
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::VulkanError;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder,
    CommandBufferUsage,
    PrimaryAutoCommandBuffer,
    RenderPassBeginInfo,
    SubpassBeginInfo,
    SubpassContents,
    SubpassEndInfo,
};
use vulkano::swapchain::{PresentMode, Surface, Swapchain, SwapchainCreateInfo};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::pipeline::graphics::vertex_input::{VertexDefinition};
use vulkano::pipeline::graphics::{
    color_blend::{ColorBlendAttachmentState, ColorBlendState},
    input_assembly::InputAssemblyState,
    multisample::MultisampleState,
    rasterization::RasterizationState,
    viewport::{Viewport, ViewportState},
    GraphicsPipelineCreateInfo,
};
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    GraphicsPipeline,
    PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{
    Framebuffer,
    FramebufferCreateInfo,
    RenderPass,
    Subpass,
};
use vulkano::sync::{self, GpuFuture};
use vulkano::swapchain::{self, SwapchainPresentInfo};

/**
 * * Vertex shader module definition using `vulkano_shaders::shader!`.
 *
 * This macro compiles the embedded GLSL source into SPIR-V at build time
 * and generates strongly-typed Rust bindings to interface with the shader
 * (entry points, descriptor layouts, etc.).
 * ? Invariants / Requirements:
 * - Vertex buffer layout in Rust must match:
 *     - `location 0 → vec2` (e.g. `[f32; 2]`)
 *     - `location 1 → vec3` (e.g. `[f32; 3]`)
 * - Pipeline vertex input state must reflect this exact layout.
 *
 * ? Notes:
 * - Coordinates are expected in NDC space [-1, 1].
 * - No transformation (MVP) is applied here; vertices are used as-is.
 *
 * ? Generated API (by macro):
 * - `vs::load(device)` → loads the compiled shader module.
 * - `vs::entry_point("main")` → retrieves the shader entry point.
 *
 * ! Failure Modes:
 * - Mismatch between shader inputs and vertex buffer → undefined behavior
 *   or validation errors.
 *
 * ? Sources:
 * - Vulkan Spec (Shader Interfaces):
 *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap14.html
 * - Vulkano shader macro:
 *   https://docs.rs/vulkano-shaders/latest/vulkano_shaders/
 */
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(push_constant) uniform PushConstants {
                vec2 window_size;
            } pc;

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec3 color;

            layout(location = 0) out vec3 out_color;

            void main() {
                vec2 ndc = vec2(
                    (2.0 * position.x / pc.window_size.x) - 1.0,
                    1.0 - (2.0 * position.y / pc.window_size.y)
                );

                gl_Position = vec4(ndc, 0.0, 1.0);
                out_color = color;
            }
        ",
    }
}

/**
 * * Fragment shader module definition using `vulkano_shaders::shader!`.
 *
 * This macro compiles the embedded GLSL source into SPIR-V at build time
 * and generates Rust bindings to interface with the shader (entry points,
 * descriptor layouts, etc.).
 *
 * ? Shader Behavior:
 * - Inputs:
 *     - `location = 0`: `vec3 in_color`
 *         Interpolated color received from the vertex shader.
 *
 * - Outputs:
 *     - `location = 0`: `vec4 f_color`
 *         Final color written to the framebuffer attachment.
 *
 * - Main Operation:
 *     - Converts the incoming RGB color into RGBA by adding a constant
 *       alpha value of 1.0 (fully opaque):
 *         `f_color = vec4(in_color, 1.0)`
 *
 * ? Invariants / Requirements:
 * - Input interface must match the vertex shader output:
 *     - `location 0 → vec3`
 * - The render pass color attachment format must be compatible with
 *   a `vec4` output (e.g., `R8G8B8A8_UNORM`, `B8G8R8A8_SRGB`, etc.).
 *
 * ? Notes:
 * - No lighting, blending, or gamma correction is applied here.
 * - Alpha is hardcoded to 1.0 → no transparency unless blending is enabled
 *   in the pipeline.
 *
 * ? Generated API (by macro):
 * - `fs::load(device)` → loads the compiled shader module.
 * - `fs::entry_point("main")` → retrieves the shader entry point.
 *
 * ! Failure Modes:
 * - Mismatch between vertex output and fragment input locations/types
 *   → validation errors or undefined rendering.
 * - Incompatible framebuffer format → pipeline creation failure.
 *
 * ? Sources:
 * - Vulkan Spec (Fragment Shader Stage):
 *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap14.html
 * - Vulkano shader macro:
 *   https://docs.rs/vulkano-shaders/latest/vulkano_shaders/
 */
mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) in vec3 in_color;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(in_color, 1.0);
            }
        ",
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants {
    window_size: [f32; 2],
}

/**
 * * Encapsulates low-level resources required to manage a window and its
 * * associated rendering state.
 *
 * This structure groups together both GLFW windowing objects and Vulkan
 * rendering resources tied to that window. It acts as the bridge between
 * platform-specific window handling and GPU-side rendering execution.
 *
 * ? Invariants / Requirements:
 * - `framebuffers.len()` must match the number of swapchain images.
 * - `graphics_pipeline` must be created with a render pass compatible
 *   with the framebuffers.
 * - `previous_frame_end` must be properly flushed and updated each frame
 *   to maintain correct GPU synchronization.
 *
 * ! Lifecycle Notes:
 * - All Vulkan resources (framebuffers, pipeline, futures) must be
 *   recreated when the swapchain is rebuilt (e.g., window resize).
 * - GLFW resources (`instance`, `context`, `events`) must remain valid
 *   for the entire lifetime of the window.
 *
 * ? Sources:
 * - Vulkan synchronization:
 *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap7.html
 * - Vulkano futures:
 *   https://docs.rs/vulkano/latest/vulkano/sync/
 * - GLFW windowing:
 *   https://www.glfw.org/docs/latest/window_guide.html
 * - glfw-rs:
 *   https://github.com/PistonDevelopers/glfw-rs
 */
struct WindowHandle
{
    vmnl_instance:        Arc<VMNLInstance>,
    /// * List of framebuffers associated with the swapchain images.
    framebuffers:         Vec<Arc<Framebuffer>>,
    /// * Preconfigured Vulkan graphics pipeline used to render into the framebuffer
    graphics_pipeline:    Arc<GraphicsPipeline>,
    /// * Synchronization primitive representing the completion of the previous frame.
    previous_frame_end:   Option<Box<dyn GpuFuture>>,
    swapchain:        Arc<Swapchain>,
    /// * GLFW context responsible for managing windowing and event polling.
    instance:             glfw::Glfw,
    /// * Handle to the actual OS window (GLFW window).
    context:              glfw::PWindow,
    /// * Event receiver channel used to retrieve window events.
    events:               glfw::GlfwReceiver<(
                            f64,
                            glfw::WindowEvent
                          )>
}

/**
 * * Represents the runtime state of a window.
 *
 * This structure stores transient flags describing the current lifecycle
 * and availability of the window.
 *
 * ? Invariants:
 * - `is_ready == true` implies that all required resources (context,
 *   surface, swapchain, etc.) are initialized.
 * - `is_open == false` implies that no further rendering or event polling
 *   should be performed.
 */
struct WindowState
{
    /// * Indicates whether the window is fully initialized and ready for use.
    is_ready:             bool,
    /// * Indicates whether the window is currently open.
    is_open:              bool
}

/**
 * * Represents the parameter configuration of the window instance.
 *
 * This structure have all information that describe the window instance.
 *
 * ? Invariants:
 * - `is_close_with_espace` is set as true by default and can be
 *   set with the `should_close_with_escape_pressed(closed: bool)` function.
 * - `width` can't be set below 64 pixels.
 * - `height` can't be set below 64 pixels.
 */
struct WindowConfig
{
    /// * Indicates whether the window can be closed by pressed the espace keyboard.
    is_close_with_escape: bool,
    /// * Actual window instance title.
    title:                String,
    /// * Actual window instance width (64 or above).
    width:                u32,
    /// * Actual window instance height (64 or above).
    height:               u32
}

/**
 * * Represents the public VMNL Window instance manipulated by the API
 *
 * This structure store a WindowHandle, WindowState and WindowConfig.
 */
pub struct Window
{
    /// * Encapsulates low-level resources required to manage the window instance.
    window_handle:        WindowHandle,
    /// * Represents the runtime state of the window instance.
    window_state:         WindowState,
    /// * Represents the parameter configuration of the window instance.
    window_config:        WindowConfig
}

impl Window
{
    fn create_image_views(
        images: &[Arc<Image>]
    )-> Vec<Arc<ImageView>>
    {
        return images
            .iter()
            .map(|image| {
                ImageView::new(
                    image.clone(),
                    ImageViewCreateInfo {
                        view_type: ImageViewType::Dim2d,
                        format: image.format(),
                        subresource_range: image.subresource_range(),
                        ..Default::default()
                    },
                )
                .expect("Failed to create swapchain image view")
            })
            .collect()
    }

    /// cf: https://docs.rs/vulkano/latest/vulkano/swapchain/struct.Surface.html
    fn create_surface(
        instance: &Arc<Instance>,
        window  : &glfw::PWindow
    ) -> Arc<Surface>
    {
        unsafe {
            return Surface::from_window_ref(instance.clone(), window)
            .expect("Failed to created Surface");
        }
    }

    /// cf: https://docs.rs/vulkano/latest/vulkano/swapchain/index.html
    fn create_swapchain(
        device:        &Arc<Device>,
        surface:       &Arc<Surface>,
        window_extent: [u32; 2]
    ) -> (Arc<Swapchain>, Vec<Arc<Image>>)
    {
        let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .expect("Failed to create surface capabilities");
        let (image_format, image_color_space) = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .expect("Failed to create surface format")[0];
        let mut min_image_count = surface_capabilities.min_image_count.max(2);
        if let Some(max_image_count) = surface_capabilities.max_image_count {
            min_image_count = min_image_count.min(max_image_count);
        }
        let image_extent =
        if let Some(current_extent) = surface_capabilities.current_extent {
            current_extent
        } else {
            [
                window_extent[0].clamp(
                    surface_capabilities.min_image_extent[0],
                    surface_capabilities.max_image_extent[0],
                ),
                window_extent[1].clamp(
                    surface_capabilities.min_image_extent[1],
                    surface_capabilities.max_image_extent[1],
                ),
            ]
        };

        return Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count,
                image_format,
                image_color_space,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                .supported_composite_alpha
                .into_iter()
                .next()
                .expect("Not supported surface composite alpha."),
                pre_transform: surface_capabilities.current_transform,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            }
        )
        .expect("Failed to create Swapchain");
    }

    /**
     * * Create a render pass, that describes the overall process of drawing a frame.
     * * It is subdivided into one or more subpasses.
     *
     * Render passes are typically created at initialization only
     * (for example during a loading screen) because they can be costly.
     * While framebuffers can be created and destroyed either at initialization or during the frame.
     * Consequently you can create graphics pipelines from a render pass object alone.
     *
     * ! Parameter:
     * - `vmnl_instance`: Vulkan context holding the device and the swapchain
     *   resources required to build framebuffer attachments.
     *
     * ! Return:
     * - `Arc<RenderPass>`: Shared handle to the created render pass.
     *
     * ? Requierement:
     * - Need a device selected by a physical device.
     * - Need also a swapchain created before.
     *
     * ? Sources:
     * - Vulkano render pass module:
     *   https://docs.rs/vulkano/latest/vulkano/render_pass/index.html
     */
    fn create_render_pass(
        device:        &Arc<Device>,
        swapchain:     &Arc<Swapchain>
    ) -> Arc<RenderPass>
    {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )
        .expect("Failed to create render pass")
    }

    /**
     * * Creates one framebuffer for each swapchain image.
     *
     * This function builds a set of `Framebuffer` objects compatible with the
     * provided render pass. Each framebuffer typically wraps a single swapchain
     * image view, or multiple attachments if the render pass also uses depth,
     * stencil, or additional color targets.
     *
     * ! Parameters:
     * - `vmnl_instance`: Vulkan context holding the device and the swapchain
     *   resources required to build framebuffer attachments.
     * - `render_pass`: Render pass that defines the attachment layout and
     *   subpass structure each framebuffer must match.
     *
     * ! Returns:
     * - `Vec<Arc<Framebuffer>>`: List of framebuffers, generally one per
     *   swapchain image.
     *
     * ? Invariants / Requirements:
     * - Each framebuffer must use attachments whose formats and ordering match
     *   the attachments declared in `render_pass`.
     * - The framebuffer extent must match the dimensions expected for rendering,
     *   usually the current swapchain extent.
     * - If the swapchain is recreated (for example after a resize), the
     *   framebuffers must also be recreated.
     *
     * ? Typical Usage:
     * - Create image views from swapchain images.
     * - For each image view, create a framebuffer bound to the same render pass.
     * - Reuse the resulting framebuffer list during command buffer recording.
     *
     * ! Failure Modes:
     * - Attachment count mismatch with the render pass.
     * - Attachment format incompatibility.
     * - Using outdated swapchain resources after a resize or surface change.
     *
     * ? Performance Notes:
     * - Framebuffer creation is not done every frame.
     * - Recreate only when dependent resources change, especially on swapchain
     *   rebuild.
     *
     * ? Sources:
     * - Vulkan Specification, Framebuffer chapter:
     *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap8.html
     * - Vulkano framebuffer module:
     *   https://docs.rs/vulkano/latest/vulkano/render_pass/
     */
    fn create_framebuffers(
        image_views: &Vec<Arc<ImageView>>,
        render_pass: &Arc<RenderPass>,
    ) -> Vec<Arc<Framebuffer>>
    {
        image_views
            .iter()
            .map(|image_view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image_view.clone()],
                        ..Default::default()
                    },
                )
                .expect("Failed to create framebuffer")
            })
            .collect()
    }

    /**
     * * Creates and configures a Vulkan graphics pipeline.
     *
     * This function builds a `GraphicsPipeline` object using the provided
     * VMNL instance and render pass. The pipeline encapsulates the full
     * fixed-function and programmable stages required for rendering
     * (shader stages, vertex input, rasterization, viewport, etc.).
     *
     * ! Parameters:
     * - `vmnl_instance`: Global Vulkan context containing the logical device,
     *   allocators, and queue configuration used to create pipeline resources.
     * - `render_pass`: Render pass describing the framebuffer attachments,
     *   subpasses, and dependencies the pipeline must be compatible with.
     *
     * ! Returns:
     * - `Arc<GraphicsPipeline>`: Shared handle to the created graphics pipeline.
     *
     * ? Invariants / Requirements:
     * - The pipeline layout (descriptor sets, push constants) must match
     *   the shaders used in the pipeline.
     * - The vertex input description must match the memory layout of the
     *   bound vertex buffers.
     * - The render pass must be compatible with the framebuffer used during
     *   command buffer recording (same attachments/formats).
     *
     * ? Performance Notes:
     * - Pipeline creation is expensive; it should be done once and reused.
     * - Consider pipeline caching (`VkPipelineCache`) to reduce creation cost.
     *
     * ! Failure Modes:
     * - Invalid shader modules or mismatched layouts will cause pipeline creation
     *   to fail at runtime (validation layers strongly recommended).
     *
     * ? Sources:
     * - Vulkan Specification:
     *   https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap9.html
     * - Vulkano documentation:
     *   https://docs.rs/vulkano/latest/vulkano/pipeline/graphics/
     */
    fn create_graphics_pipeline(
        device:        &Arc<Device>,
        swapchain:     &Arc<Swapchain>,
        render_pass:   &Arc<RenderPass>
    ) -> Arc<GraphicsPipeline>
    {
        let vs = vs::load(device.clone())
            .expect("Failed to load vertex shader");
        let fs = fs::load(device.clone())
            .expect("Failed to load fragment shader");
        let vs = vs.entry_point("main").expect("Missing vertex entry point");
        let fs = fs.entry_point("main").expect("Missing fragment entry point");
        let stages = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .expect("Failed to create pipeline layout info"),
        )
        .expect("Failed to create pipeline layout");
        let extent = swapchain.image_extent();
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };
        let subpass = Subpass::from(render_pass.clone(), 0)
            .expect("Failed to create subpass");
        let vertex_input_state = VMNLVertex::per_vertex()
            .definition(&vs)
            .expect("Failed to create vertex input state");

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .expect("Failed to create graphics pipeline")
    }

    /**
     * * Initializes a GLFW window along with its associated event receiver.
     *
     * This function creates a window using the provided GLFW instance and
     * configures it with the given dimensions and title. It also sets up
     * an event channel to receive window-related events (keyboard, mouse,
     * resize, etc.).
     *
     * ! Parameters:
     * - `instance`: Initialized GLFW context used to create the window.
     * - `width`: Width of the window in pixels.
     * - `height`: Height of the window in pixels.
     * - `title`: Title displayed in the window's title bar.
     *
     * ! Returns:
     * - `glfw::PWindow`: The created window handle.
     * - `glfw::GlfwReceiver<(f64, glfw::WindowEvent)>`:
     *     Event receiver used to poll and handle window events.
     *
     * ? Invariants / Notes:
     * - The GLFW instance must be properly initialized before calling this function.
     * - The returned event receiver must be polled regularly to keep the window responsive.
     * - Failure to process events may result in the window being marked as unresponsive
     *   by the operating system.
     *
     * ? Source:
     * - GLFW documentation: https://www.glfw.org/docs/latest/window_guide.html
     */
    fn init_window(
        mut instance: glfw::Glfw,
        width:        u32,
        height:       u32,
        title:        &str
    ) -> (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>)
    {
        return instance
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("VMNL Error: Failed to create VMNL window.");
    }

    /**
     * * Updates and returns the current open state of the window.
     *
     * This method queries the underlying GLFW window to determine whether a
     * close event has been requested (e.g., user clicks the close button or
     * presses a bound key). It updates the internal `is_open` flag accordingly.
     *
     * ? Behavior:
     * - If the `safe` feature is enabled, the method first checks whether the
     *   window is ready. If not, it returns `false` immediately.
     * - Otherwise, it calls `should_close()` on the GLFW window context and
     *   updates `window_state.is_open` to the inverse of that value.
     *
     * ! Returns:
     * - `true` if the window should remain open.
     * - `false` if a close event has been triggered or the window is not ready
     *   (when `safe` is enabled).
     *
     * ? Invariants:
     * - `is_open == false` implies the main loop should terminate.
     * - Must be called regularly (typically once per frame) to keep the state
     *   in sync with user actions.
     *
     * ? Typical Usage:
     * - Main loop condition:
     *     `while window.is_open() { ... }`
     *
     * ! Failure Modes:
     * - Skipping this call may cause the application to ignore close events.
     *
     */
    pub fn is_open(&mut self) -> bool
    {
        #[cfg(feature = "safe")] {
            if self.is_ready == false {
                return false;
            }
        }
        self.window_state.is_open = !self.window_handle.context.should_close();
        return self.window_state.is_open;
    }


    /**
     * * Returns whether the window is fully initialized and ready for use.
     *
     * This method exposes the internal `is_ready` flag from `WindowState`,
     * indicating that all required resources (window context, Vulkan objects,
     * swapchain, etc.) have been successfully created.
     *
     * ! Returns:
     * - `true` if the window is ready for rendering and event processing.
     * - `false` if initialization is incomplete or has failed.
     *
     * ? Invariants:
     * - When `true`, it is safe to perform rendering operations.
     * - When `false`, dependent systems should not attempt to use GPU or
     *   window-related resources.
    */
    pub fn is_ready(&self) -> bool
    {
        return self.window_state.is_ready;
    }

    /**
     * * Enables or disables closing the window when the Escape key is pressed.
     *
     * This method updates the window configuration flag controlling whether
     * an Escape key press should trigger a window close event.
     *
     * ! Parameters:
     * - `closed`:
     *     - `true`  → pressing Escape will request window closure.
     *     - `false` → Escape key will be ignored for closing behavior.
     *
     * ? Invariants:
     * - It's set at true by default.
     *
     * ? Typical Usage:
     * - Configure behavior at initialization:
     *     `window.should_close_with_escape_pressed(false);`
     */
    pub fn should_close_with_escape_pressed(
        &mut self,
        closed: bool
    ) -> ()
    {
        self.window_config.is_close_with_escape = closed;
    }

    pub fn poll_event(&mut self) -> ()
    {
        #[cfg(feature = "safe")] {
            if self.is_ready {
                return;
            }
        }
        self.window_handle.instance.poll_events();
        for (_, event) in glfw::flush_messages(&self.window_handle.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    if self.window_config.is_close_with_escape {
                        self.window_handle.context.set_should_close(true);
                    }
                }
                _ => {}
            }
        }
    }

    /**
     * * Returns the current window width in pixels.
     *
     * This method exposes the width stored in `WindowConfig`.
     * It reflects the configured logical size of the window.
     *
     * ! Returns:
     * - `u32`: Window width in pixels.
     *
     * ? Invariants:
     * - Value is set during initialization or updated on resize events.
     * - May differ from framebuffer extent used by Vulkan swapchain.
     *
     * ? Notes:
     * - For rendering, prefer querying the framebuffer size if DPI scaling
     *   is involved.
     */
    pub fn get_width(&self) -> u32
    {
        return self.window_config.width;
    }

    /**
     * * Returns the current window height in pixels.
     *
     * This method exposes the height stored in `WindowConfig`.
     * It reflects the configured logical size of the window.
     *
     * ! Returns:
     * - `u32`: Window height in pixels.
     *
     * ? Invariants:
     * - Value is set during initialization or updated on resize events.
     * - May differ from framebuffer extent used by Vulkan swapchain.
     *
     * ? Notes:
     * - For rendering, prefer querying the framebuffer size if DPI scaling
     *   is involved.
     */
    pub fn get_height(&self) -> u32
    {
        return self.window_config.height;
    }

    fn build_command_buffer(
        &self,
        image_index: u32,
        graphics: &Graphics
    ) -> Arc<PrimaryAutoCommandBuffer>
    {
        let extent = self.window_handle.swapchain.image_extent();

        let push_constants = PushConstants {
            window_size: [extent[0] as f32, extent[1] as f32],
        };

        let framebuffer =
            self.window_handle.framebuffers[image_index as usize].clone();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.window_handle
                .vmnl_instance
                .command_buffer_allocator
                .clone(),
            self.window_handle
                .vmnl_instance
                .graphics_queue
                .queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("Failed to create command buffer builder");

        unsafe {
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.0, 0.0, 0.0, 0.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .expect("Failed to begin render pass")
                .bind_pipeline_graphics(self.window_handle.graphics_pipeline.clone())
                .expect("Failed to bind graphics pipeline")
                .push_constants(
                    self.window_handle.graphics_pipeline.layout().clone(),
                    0,
                    push_constants,
                )
                .expect("Failed to push constants")
                .bind_vertex_buffers(0, graphics.vertex_buffer.clone())
                .expect("Failed to bind vertex buffer")
                .draw(graphics.vertex_buffer.len() as u32, 1, 0, 0)
                .expect("Failed to record draw command")
                .end_render_pass(SubpassEndInfo::default())
                .expect("Failed to end render pass");
        }

        builder.build()
            .expect("Failed to build command buffer")
    }

    pub fn draw(&mut self, graphics: &Graphics)
    {
        if let Some(previous_frame_end) = &mut self.window_handle.previous_frame_end {
            previous_frame_end.cleanup_finished();
        }
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.window_handle.swapchain.clone(), None) {
                Ok(result) => result,
                Err(Validated::Error(VulkanError::OutOfDate)) => {
                    eprintln!("Swapchain out of date: resize handling not implemented yet.");
                    return;
                }
                Err(error) => {
                    panic!("Failed to acquire next image: {error:?}");
                }
            };
        if suboptimal {
            eprintln!("Swapchain is suboptimal: resize handling not implemented yet.");
        }
        let command_buffer = self.build_command_buffer(image_index, graphics);
        let future = self.window_handle
            .previous_frame_end
            .take()
            .expect("previous_frame_end was None")
            .join(acquire_future)
            .then_execute(self.window_handle.vmnl_instance.graphics_queue.clone(), command_buffer)
            .expect("Failed to execute command buffer")
            .then_swapchain_present(
                self.window_handle.vmnl_instance.graphics_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.window_handle.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();
        match future {
            Ok(future) => {
                self.window_handle.previous_frame_end = Some(future.boxed());
            }
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                eprintln!("Present returned OutOfDate: resize handling not implemented yet.");
                self.window_handle.previous_frame_end =
                    Some(sync::now(self.window_handle.vmnl_instance.device.clone()).boxed());
                return;
            }
            Err(error) => {
                eprintln!("Failed to flush future: {error:?}");
                self.window_handle.previous_frame_end =
                    Some(sync::now(self.window_handle.vmnl_instance.device.clone()).boxed());
                return;
            }
        }
    }

    /**
     * * Initializes a VMNL window along with its associated event receiver.
     *
     * This function creates a window and configures it with the given dimensions and title.
     * It also sets up an event channel to receive window-related events (keyboard, mouse,
     * resize, etc.).
     *
     * ? Behavior:
     * - If the `safe` feature is enabled, the method first checks whether the
     *   window is ready. If not, it returns `false` immediately.
     *
     * ! Parameters:
     * - `width`: Width of the window in pixels.
     * - `height`: Height of the window in pixels.
     * - `title`: Title displayed in the window's title bar.
     *
     * ! Returns:
     * - `VMNLResult<Self>`: The created window handle.
     *
     * ? Invariants / Notes:
     * - The first time of calling this function while created the vmnl instance at the same time
     */
    pub fn new(
        vmnl_context:  &VMNLContext,
        width:         u32,
        height:        u32,
        title:         &str
    ) -> VMNLResult<Self>
    {
        let vmnl_instance: Arc<VMNLInstance> = vmnl_context.inner.clone();
        let mut instance = glfw::init(glfw::fail_on_errors)
            .map_err(|_| VMNLError::VMNLInitFailed)?;
        instance.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (mut window, events) =
            Window::init_window(instance.clone(), width, height, title);
        let surface: Arc<Surface> =
            Self::create_surface(&vmnl_instance.instance, &window);
        let supports_present = vmnl_instance.physical_device
            .surface_support(vmnl_instance.graphics_queue_family_index, &surface)
            .expect("failed to query surface support");
        if !supports_present {
            panic!("selected queue family does not support presentation on this surface");
        }
        let (frame_buffer_width, frame_buffer_height): (i32, i32) =
            window.get_framebuffer_size();
        let (swapchain, images): (Arc<Swapchain>, Vec<Arc<Image>>) =
            Self::create_swapchain(
                &vmnl_instance.device,
                &surface,
                [frame_buffer_width as u32, frame_buffer_height as u32]
            );
        let image_views: Vec<Arc<ImageView>> =
            Self::create_image_views(&images);
        let render_pass: Arc<RenderPass>
            = Self::create_render_pass(&vmnl_instance.device, &swapchain);
        let framebuffers: Vec<Arc<Framebuffer>>
            = Self::create_framebuffers(&image_views, &render_pass);
        let graphics_pipeline: Arc<GraphicsPipeline>
            = Self::create_graphics_pipeline(&vmnl_instance.device, &swapchain, &render_pass);
        let previous_frame_end: Option<Box<dyn GpuFuture>>
            = Some(sync::now(vmnl_instance.device.clone()).boxed());

        if window.is_visible() == false {
            window.show();
        }
        window.set_key_polling(true);
        println!("VMNL log: Window named \"{}\" with  [{}, {}] created.", title, height, width);
        Ok(Self {
            window_handle: WindowHandle {
                instance,
                vmnl_instance,
                context: window,
                events,
                framebuffers,
                graphics_pipeline,
                previous_frame_end,
                swapchain,
            },
            window_state: WindowState {
                is_ready: true,
                is_open:  false
            },
            window_config: WindowConfig {
                is_close_with_escape: true,
                title:                title.to_string(),
                width,
                height
            }
        })
    }

}

impl Drop for Window
{
    fn drop(&mut self) -> ()
    {
        eprintln!("VMNL log: Window named \"{}\" destroyed.", self.window_config.title);
    }
}
