// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Command buffer submodule for recording Vulkan draw commands.
//!
//! This module builds primary command buffers for rendering VMNL graphics objects
//! into the current swapchain framebuffer.

use crate::common::{BlendMode, MaterialKey, PipelineKey};
use crate::window::render::RenderPassCommand;
use crate::window::PushConstants;
use crate::{window::inner::VMNLWindow, VMNLError, VMNLErrorKind, VMNLResult};
use smallvec::smallvec;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
    },
    pipeline::Pipeline,
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline},
    render_pass::Framebuffer,
    swapchain::Swapchain,
};

impl VMNLWindow {
    #[allow(clippy::cast_precision_loss)]
    fn extent_as_f32(extent: [u32; 2]) -> [f32; 2] {
        [extent[0] as f32, extent[1] as f32]
    }

    /// Builds a Vulkan command buffer for rendering the provided graphics objects to the specified swapchain image.
    ///
    /// # Arguments
    /// - `image_index`: Index of the swapchain image to render to.
    /// - `commands`: Ordered render commands to record.
    ///
    /// # Returns
    /// An `Arc<PrimaryAutoCommandBuffer>` containing the built command buffer ready for execution.
    #[allow(clippy::too_many_lines)]
    pub(super) fn build_command_buffer(
        &self,
        image_index: u32,
        commands: &[RenderPassCommand],
    ) -> VMNLResult<Arc<PrimaryAutoCommandBuffer>> {
        let swapchain: &Arc<Swapchain> = &self.handle.swapchain;
        let allocator: Arc<StandardCommandBufferAllocator> =
            self.handle.vmnl_instance.command_buffer_allocator.clone();
        let queue_family: u32 = self
            .handle
            .vmnl_instance
            .graphics_queue
            .queue_family_index();
        let extent: [u32; 2] = swapchain.image_extent();
        let extent_f32: [f32; 2] = Self::extent_as_f32(extent);
        let framebuffer_index: usize = usize::try_from(image_index)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
        let framebuffer: Arc<Framebuffer> = self
            .handle
            .framebuffers
            .get(framebuffer_index)
            .cloned()
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
        let mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> =
            AutoCommandBufferBuilder::primary(
                allocator,
                queue_family,
                CommandBufferUsage::OneTimeSubmit,
            )
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanCommandBufferCreationFailed))?;

        // SAFETY: all resources belong to this window's Vulkan context, remain owned while the
        // command buffer exists, and Vulkano validates each command before recording it.
        unsafe {
            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some(self.state.clear_color.into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))?
                .set_viewport(
                    0,
                    smallvec![Viewport {
                        offset: [0.0, 0.0],
                        extent: extent_f32,
                        depth_range: 0.0..=1.0,
                    }],
                )
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
            let mut current_blend_mode: Option<BlendMode> = None;
            for command in commands {
                match command {
                    RenderPassCommand::D2 { items } => {
                        for render_item in items {
                            debug_assert_eq!(render_item.pipeline_key, PipelineKey::Color2D);
                            debug_assert_eq!(render_item.material_key, MaterialKey::VertexColor);
                            let pipeline: &Arc<GraphicsPipeline> = match render_item.blend_mode {
                                BlendMode::Opaque => &self.handle.pipeline_2d_opaque,
                                BlendMode::Alpha => &self.handle.pipeline_2d_alpha,
                            };
                            if current_blend_mode != Some(render_item.blend_mode) {
                                builder
                                    .bind_pipeline_graphics(pipeline.clone())
                                    .map_err(|_| {
                                        VMNLError::new(VMNLErrorKind::VulkanPipelineCreationFailed)
                                    })?;
                                current_blend_mode = Some(render_item.blend_mode);
                            }
                            let push_constants: PushConstants = PushConstants {
                                window_size: extent_f32,
                            };
                            builder
                                .push_constants(pipeline.layout().clone(), 0, push_constants)
                                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?
                                .bind_vertex_buffers(0, render_item.vertex_buffer.as_subbuffer())
                                .map_err(|_| {
                                    VMNLError::new(VMNLErrorKind::VulkanVertexBufferCreationFailed)
                                })?;
                            if let Some(index_buffer) = &render_item.index_buffer {
                                builder
                                    .bind_index_buffer(index_buffer.as_subbuffer())
                                    .map_err(|_| {
                                        VMNLError::new(
                                            VMNLErrorKind::VulkanIndexBufferCreationFailed,
                                        )
                                    })?
                                    .draw_indexed(render_item.index_count, 1, 0, 0, 0)
                                    .map_err(|_| {
                                        VMNLError::new(VMNLErrorKind::VulkanValidationFailed)
                                    })?;
                            } else {
                                builder
                                    .draw(render_item.vertex_count, 1, 0, 0)
                                    .map_err(|_| {
                                        VMNLError::new(VMNLErrorKind::VulkanValidationFailed)
                                    })?;
                            }
                        }
                    }
                    RenderPassCommand::Raw { items } => {
                        current_blend_mode = None;
                        for render_item in items {
                            if !Arc::ptr_eq(
                                &render_item.pipeline_device,
                                &self.handle.vmnl_instance.device,
                            ) || !Arc::ptr_eq(
                                &render_item.geometry_device,
                                &self.handle.vmnl_instance.device,
                            ) || !Arc::ptr_eq(
                                &render_item.pipeline_render_pass,
                                &self.handle.render_pass,
                            ) {
                                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                                    "raw pipeline and geometry must belong to this window context"
                                        .into(),
                                )));
                            }

                            builder
                                .bind_pipeline_graphics(render_item.pipeline.clone())
                                .map_err(|_| {
                                    VMNLError::new(VMNLErrorKind::VulkanPipelineCreationFailed)
                                })?
                                .bind_vertex_buffers(0, render_item.vertex_buffer.clone())
                                .map_err(|_| {
                                    VMNLError::new(VMNLErrorKind::VulkanVertexBufferCreationFailed)
                                })?;
                            if let Some(index_buffer) = &render_item.index_buffer {
                                builder
                                    .bind_index_buffer(index_buffer.clone())
                                    .map_err(|_| {
                                        VMNLError::new(
                                            VMNLErrorKind::VulkanIndexBufferCreationFailed,
                                        )
                                    })?
                                    .draw_indexed(render_item.index_count, 1, 0, 0, 0)
                                    .map_err(|_| {
                                        VMNLError::new(VMNLErrorKind::VulkanValidationFailed)
                                    })?;
                            } else {
                                builder
                                    .draw(render_item.vertex_count, 1, 0, 0)
                                    .map_err(|_| {
                                        VMNLError::new(VMNLErrorKind::VulkanValidationFailed)
                                    })?;
                            }
                        }
                    }
                }
            }
            builder
                .end_render_pass(SubpassEndInfo::default())
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))?;
        }
        builder
            .build()
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanCommandBufferCreationFailed))
    }
}
