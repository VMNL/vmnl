////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Vulkan graphics pipeline and shader module creation helpers.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLWindow;
use crate::graphics::GpuVertex;
use crate::window::shaders::{
    ShaderInput, WindowShaders, DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER,
};
use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::sync::Arc;
use vulkano::{
    device::Device,
    pipeline::{
        graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState},
        graphics::input_assembly::InputAssemblyState,
        graphics::multisample::MultisampleState,
        graphics::rasterization::RasterizationState,
        graphics::vertex_input::{Vertex as VulkanoVertex, VertexDefinition, VertexInputState},
        graphics::viewport::ViewportState,
        graphics::GraphicsPipelineCreateInfo,
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{RenderPass, Subpass},
    shader::{EntryPoint, ShaderModule},
};

impl VMNLWindow {
    /// Load a shader from GLSL source code, compile it, and create a Vulkan shader module.
    ///
    /// # Arguments
    /// - `device`: Vulkan logical device used to create the shader module.
    /// - `compiler`: Shader compiler instance for compiling GLSL source code.
    /// - `source`: GLSL source code of the shader.
    /// - `kind`: The kind of shader (vertex, fragment, etc.) to compile.
    /// - `input_file_name`: A string used for error reporting during compilation.
    ///
    /// # Returns
    /// An `Arc<ShaderModule>` representing the compiled shader module, or an error if compilation or creation fails.
    fn load_shader_from_src(
        device: &Arc<Device>,
        compiler: &shaderc::Compiler,
        source: &str,
        kind: shaderc::ShaderKind,
        input_file_name: &str,
    ) -> VMNLResult<Arc<ShaderModule>> {
        log::debug!("compiling {kind:?} shader from {input_file_name}");
        let artifact = compiler
            .compile_into_spirv(source, kind, input_file_name, "main", None)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;

        // SAFETY: shaderc successfully compiled `artifact` into valid SPIR-V.
        unsafe {
            ShaderModule::new(
                device.clone(),
                vulkano::shader::ShaderModuleCreateInfo::new(artifact.as_binary()),
            )
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderModuleCreationFailed))
        }
    }

    /// Load a shader from a file path, compile it, and create a Vulkan shader module.
    ///
    /// # Arguments
    /// - `device`: Vulkan logical device used to create the shader module.
    /// - `compiler`: Shader compiler instance for compiling GLSL source code.
    /// - `path`: File path to the shader source code.
    /// - `kind`: The kind of shader (vertex, fragment, etc.) to compile.
    /// - `input_file_name`: A string used for error reporting during compilation.
    ///
    /// # Returns
    /// An `Arc<ShaderModule>` representing the compiled shader module, or an error if compilation or creation fails.
    fn load_shader_from_path(
        device: &Arc<Device>,
        compiler: &shaderc::Compiler,
        path: &std::path::Path,
        kind: shaderc::ShaderKind,
    ) -> VMNLResult<Arc<ShaderModule>> {
        log::debug!("loading {kind:?} shader from {}", path.display());
        let source = std::fs::read_to_string(path)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;

        let input_file_name = path.display().to_string();
        Self::load_shader_from_src(device, compiler, &source, kind, &input_file_name)
    }

    /// Create and configure a Vulkan graphics pipeline.
    ///
    /// # Arguments
    /// - `device`: Logical Vulkan device.
    /// - `swapchain`: Swapchain to determine image extent.
    /// - `render_pass`: Render pass the pipeline must be compatible with.
    ///
    /// # Returns
    /// An `Arc<ShapePipeline>` representing the created graphics pipeline.
    ///
    /// # Sources
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap9.html>
    /// <https://docs.rs/vulkano/latest/vulkano/pipeline/graphics>/
    pub(super) fn create_graphics_pipeline(
        device: &Arc<Device>,
        render_pass: &Arc<RenderPass>,
        shaders: &WindowShaders,
    ) -> VMNLResult<Arc<GraphicsPipeline>> {
        let compiler: shaderc::Compiler = shaderc::Compiler::new()
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;
        let vs: Arc<ShaderModule> = match shaders.vertex.as_ref() {
            Some(ShaderInput::Src(source)) => Self::load_shader_from_src(
                device,
                &compiler,
                source,
                shaderc::ShaderKind::Vertex,
                "user.vert",
            )?,
            Some(ShaderInput::Path(path)) => Self::load_shader_from_path(
                device,
                &compiler,
                path.as_path(),
                shaderc::ShaderKind::Vertex,
            )?,
            _ => Self::load_shader_from_src(
                device,
                &compiler,
                DEFAULT_VERTEX_SHADER,
                shaderc::ShaderKind::Vertex,
                "default.vert",
            )?,
        };
        let fs: Arc<ShaderModule> = match shaders.fragment.as_ref() {
            Some(ShaderInput::Src(source)) => Self::load_shader_from_src(
                device,
                &compiler,
                source,
                shaderc::ShaderKind::Fragment,
                "user.frag",
            )?,
            Some(ShaderInput::Path(path)) => Self::load_shader_from_path(
                device,
                &compiler,
                path.as_path(),
                shaderc::ShaderKind::Fragment,
            )?,
            _ => Self::load_shader_from_src(
                device,
                &compiler,
                DEFAULT_FRAGMENT_SHADER,
                shaderc::ShaderKind::Fragment,
                "default.frag",
            )?,
        };
        let vs: EntryPoint = vs
            .entry_point("main")
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;
        let fs: EntryPoint = fs
            .entry_point("main")
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;
        let stages: [PipelineShaderStageCreateInfo; 2] = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout: Arc<PipelineLayout> = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineLayoutCreationFailed))?,
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineLayoutCreationFailed))?;
        let subpass: Subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))?;
        let vertex_input_state: VertexInputState = GpuVertex::per_vertex()
            .definition(&vs)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
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
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineCreationFailed))
    }
}
