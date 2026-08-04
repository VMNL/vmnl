// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Low-level raw rendering API.

use std::marker::PhantomData;
use std::sync::Arc;

pub use vmnl_macros::{Pod, Vertex, Zeroable};

use vulkano::buffer::BufferContents as VulkanoBufferContents;
use vulkano::buffer::Subbuffer;
use vulkano::device::Device;
use vulkano::pipeline::graphics::color_blend::{
    AttachmentBlend, ColorBlendAttachmentState, ColorBlendState,
};
use vulkano::pipeline::graphics::input_assembly::{
    InputAssemblyState, PrimitiveTopology as VulkanoPrimitiveTopology,
};
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{
    Vertex as VulkanoVertex, VertexBuffersCollection, VertexDefinition,
};
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};

use crate::common::{
    checked_draw_counts, BufferMemoryPreference, GraphicsResourceFactory, IndexBuffer, VertexBuffer,
};
use crate::exception::{VMNLError, VMNLErrorKind, VMNLResult};
use crate::{Context, Window};

#[doc(hidden)]
pub mod __private {
    pub use bytemuck;
    pub use vulkano;
}

/// Marker trait for raw buffer contents.
pub trait BufferContents: VulkanoBufferContents {}

impl<T> BufferContents for T where T: VulkanoBufferContents {}

/// Marker trait for raw vertex layouts.
pub trait Vertex: VulkanoVertex {}

impl<T> Vertex for T where T: VulkanoVertex {}

/// Marker trait for plain-old-data raw values.
pub trait Pod: bytemuck::Pod {}

impl<T> Pod for T where T: bytemuck::Pod {}

/// Marker trait for zero-initializable raw values.
pub trait Zeroable: bytemuck::Zeroable {}

impl<T> Zeroable for T where T: bytemuck::Zeroable {}

pub use crate::common::ShaderSource;

/// Primitive topology used by a raw pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveTopology {
    /// Independent points.
    PointList,
    /// Independent lines.
    LineList,
    /// Connected line strip.
    LineStrip,
    /// Independent triangles.
    TriangleList,
    /// Connected triangle strip.
    TriangleStrip,
}

impl From<PrimitiveTopology> for VulkanoPrimitiveTopology {
    fn from(value: PrimitiveTopology) -> Self {
        match value {
            PrimitiveTopology::PointList => Self::PointList,
            PrimitiveTopology::LineList => Self::LineList,
            PrimitiveTopology::LineStrip => Self::LineStrip,
            PrimitiveTopology::TriangleList => Self::TriangleList,
            PrimitiveTopology::TriangleStrip => Self::TriangleStrip,
        }
    }
}

/// Blend mode used by a raw pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    /// No blending.
    Opaque,
    /// Standard source-alpha blending.
    Alpha,
}

/// Builder/specification for a raw graphics pipeline.
#[derive(Clone, Debug)]
pub struct PipelineSpec<TVertex> {
    vertex_shader: Option<ShaderSource>,
    fragment_shader: Option<ShaderSource>,
    topology: PrimitiveTopology,
    blend_mode: BlendMode,
    _vertex: PhantomData<TVertex>,
}

impl<TVertex> Default for PipelineSpec<TVertex> {
    fn default() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            topology: PrimitiveTopology::TriangleList,
            blend_mode: BlendMode::Opaque,
            _vertex: PhantomData,
        }
    }
}

impl<TVertex> PipelineSpec<TVertex> {
    /// Sets the vertex shader.
    #[must_use]
    pub fn vertex_shader(mut self, source: ShaderSource) -> Self {
        self.vertex_shader = Some(source);
        self
    }

    /// Sets the fragment shader.
    #[must_use]
    pub fn fragment_shader(mut self, source: ShaderSource) -> Self {
        self.fragment_shader = Some(source);
        self
    }

    /// Sets the primitive topology.
    #[must_use]
    pub fn topology(mut self, topology: PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Sets the blend mode.
    #[must_use]
    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Returns the configured primitive topology.
    #[must_use]
    pub const fn topology_value(&self) -> PrimitiveTopology {
        self.topology
    }

    /// Returns the configured blend mode.
    #[must_use]
    pub const fn blend_mode_value(&self) -> BlendMode {
        self.blend_mode
    }

    /// Builds the raw pipeline against a window render pass.
    ///
    /// # Errors
    /// Returns an error if shaders are missing, fail to compile, declare
    /// descriptors or push constants, or if Vulkan pipeline creation fails.
    pub fn build(self, window: &Window) -> VMNLResult<Pipeline<TVertex>>
    where
        TVertex: BufferContents + Vertex + 'static,
    {
        let vertex_shader = self.vertex_shader.ok_or_else(|| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "raw pipeline requires a vertex shader".into(),
            ))
        })?;
        let fragment_shader = self.fragment_shader.ok_or_else(|| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "raw pipeline requires a fragment shader".into(),
            ))
        })?;

        let device = window.device();
        let render_pass = window.render_pass();

        let vs = compile_shader(device.clone(), &vertex_shader, shaderc::ShaderKind::Vertex)?;
        let fs = compile_shader(
            device.clone(),
            &fragment_shader,
            shaderc::ShaderKind::Fragment,
        )?;

        let vs = vs
            .entry_point("main")
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;
        let fs = fs
            .entry_point("main")
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;

        let stages = [
            PipelineShaderStageCreateInfo::new(vs.clone()),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages);
        validate_raw_pipeline_layout(
            layout_info
                .set_layouts
                .iter()
                .any(|set_layout| !set_layout.bindings.is_empty()),
            !layout_info.push_constant_ranges.is_empty(),
        )?;

        let layout = PipelineLayout::new(
            device.clone(),
            layout_info
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineLayoutCreationFailed))?,
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineLayoutCreationFailed))?;

        let subpass = Subpass::from(render_pass.clone(), 0)
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanRenderPassCreationFailed))?;
        let vertex_input_state = TVertex::per_vertex()
            .definition(&vs)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;

        let graphics_pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            vulkano::pipeline::graphics::GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState {
                    topology: self.topology.into(),
                    ..Default::default()
                }),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(color_blend_state(self.blend_mode)),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..vulkano::pipeline::graphics::GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanPipelineCreationFailed))?;

        Ok(Pipeline {
            inner: graphics_pipeline,
            device,
            render_pass,
            _vertex: PhantomData,
        })
    }
}

/// Raw graphics pipeline.
pub struct Pipeline<TVertex> {
    inner: Arc<GraphicsPipeline>,
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    _vertex: PhantomData<TVertex>,
}

impl<TVertex> Pipeline<TVertex> {
    /// Starts a raw pipeline builder.
    #[must_use]
    pub fn builder() -> PipelineSpec<TVertex> {
        PipelineSpec::default()
    }

    pub(crate) fn render_item(&self, geometry: &Geometry<TVertex>) -> RenderItemRaw {
        RenderItemRaw {
            pipeline: self.inner.clone(),
            pipeline_device: self.device.clone(),
            pipeline_render_pass: self.render_pass.clone(),
            geometry_device: geometry.device.clone(),
            vertex_buffer: geometry.vertex_buffer_bytes(),
            index_buffer: geometry.index_buffer(),
            vertex_count: geometry.vertex_count,
            index_count: geometry.index_count,
        }
    }
}

/// Raw typed geometry.
pub struct Geometry<TVertex> {
    vertex_buffer: VertexBuffer<TVertex>,
    index_buffer: Option<IndexBuffer>,
    vertex_count: u32,
    index_count: u32,
    device: Arc<Device>,
}

impl<TVertex> Geometry<TVertex> {
    /// Starts a raw geometry builder.
    #[must_use]
    pub fn builder<V>(vertices: V) -> GeometryBuilder<TVertex>
    where
        V: Into<Vec<TVertex>>,
    {
        GeometryBuilder {
            vertices: vertices.into(),
            indices: None,
            memory_preference: BufferMemoryPreference::default(),
        }
    }

    fn vertex_buffer_bytes(&self) -> Subbuffer<[u8]> {
        let mut buffers = self.vertex_buffer.as_subbuffer().into_vec();
        debug_assert_eq!(buffers.len(), 1);
        buffers.remove(0)
    }

    fn index_buffer(&self) -> Option<Subbuffer<[u32]>> {
        self.index_buffer.as_ref().map(IndexBuffer::as_subbuffer)
    }
}

/// Builder for raw typed geometry.
pub struct GeometryBuilder<TVertex> {
    vertices: Vec<TVertex>,
    indices: Option<Vec<u32>>,
    memory_preference: BufferMemoryPreference,
}

impl<TVertex> GeometryBuilder<TVertex> {
    /// Sets optional indices.
    #[must_use]
    pub fn indices<I>(mut self, indices: I) -> Self
    where
        I: Into<Vec<u32>>,
    {
        self.indices = Some(indices.into());
        self
    }

    /// Sets the buffer memory preference.
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.memory_preference = preference;
        self
    }

    /// Builds GPU geometry.
    ///
    /// # Errors
    /// Returns an error if geometry validation fails or GPU buffer allocation
    /// fails.
    pub fn build(self, context: &Context) -> VMNLResult<Geometry<TVertex>>
    where
        TVertex: BufferContents,
    {
        let Self {
            vertices,
            indices,
            memory_preference,
        } = self;

        validate_geometry_inputs(vertices.len(), indices.as_deref())?;
        let (vertex_count, index_count) =
            checked_draw_counts(vertices.len(), indices.as_ref().map_or(0, Vec::len))?;

        let memory_allocator = &context.inner.memory_allocator;
        let vertex_buffer = <Self as GraphicsResourceFactory>::create_vertex_buffer(
            vertices,
            memory_preference,
            memory_allocator,
        )?;
        let index_buffer = match indices {
            Some(indices) => Some(<Self as GraphicsResourceFactory>::create_index_buffer(
                &indices,
                memory_preference,
                memory_allocator,
            )?),
            None => None,
        };

        Ok(Geometry {
            vertex_buffer,
            index_buffer,
            vertex_count,
            index_count,
            device: context.inner.device.clone(),
        })
    }
}

impl<TVertex> GraphicsResourceFactory for GeometryBuilder<TVertex> {}

#[derive(Clone)]
pub(crate) struct RenderItemRaw {
    pub(crate) pipeline: Arc<GraphicsPipeline>,
    pub(crate) pipeline_device: Arc<Device>,
    pub(crate) pipeline_render_pass: Arc<RenderPass>,
    pub(crate) geometry_device: Arc<Device>,
    pub(crate) vertex_buffer: Subbuffer<[u8]>,
    pub(crate) index_buffer: Option<Subbuffer<[u32]>>,
    pub(crate) vertex_count: u32,
    pub(crate) index_count: u32,
}

fn validate_geometry_inputs(vertex_count: usize, indices: Option<&[u32]>) -> VMNLResult<()> {
    if vertex_count == 0 {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "raw geometry requires at least one vertex".into(),
        )));
    }

    if let Some(indices) = indices {
        if indices.is_empty() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "raw geometry indices cannot be empty".into(),
            )));
        }

        let vertex_count = u32::try_from(vertex_count).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "raw geometry vertex count out of bounds".into(),
            ))
        })?;
        if indices.iter().any(|&index| index >= vertex_count) {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "raw geometry index out of bounds".into(),
            )));
        }
    }

    Ok(())
}

fn validate_raw_pipeline_layout(
    has_descriptor_bindings: bool,
    has_push_constants: bool,
) -> VMNLResult<()> {
    if has_descriptor_bindings {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "raw pipeline descriptors are not supported in checkpoint 1".into(),
        )));
    }

    if has_push_constants {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "raw pipeline push constants are not supported in checkpoint 1".into(),
        )));
    }

    Ok(())
}

fn compile_shader(
    device: Arc<Device>,
    source: &ShaderSource,
    kind: shaderc::ShaderKind,
) -> VMNLResult<Arc<ShaderModule>> {
    let (source, name) = match source {
        ShaderSource::Src(source) => (source.clone(), "raw_inline_shader.glsl".to_string()),
        ShaderSource::Path(path) => {
            let source = std::fs::read_to_string(path).map_err(|err| {
                VMNLError::new(VMNLErrorKind::InvalidState(format!(
                    "failed to read shader '{}': {}",
                    path.display(),
                    err
                )))
            })?;
            (source, path.display().to_string())
        }
    };

    let compiler = shaderc::Compiler::new()
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;

    let spirv = compiler
        .compile_into_spirv(&source, kind, &name, "main", None)
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderCompilationFailed))?;

    let create_info = ShaderModuleCreateInfo::new(spirv.as_binary());
    // SAFETY: shaderc returns valid SPIR-V for the requested stage or an error.
    unsafe { ShaderModule::new(device, create_info) }
        .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanShaderModuleCreationFailed))
}

fn color_blend_state(blend_mode: BlendMode) -> ColorBlendState {
    match blend_mode {
        BlendMode::Opaque => {
            ColorBlendState::with_attachment_states(1, ColorBlendAttachmentState::default())
        }
        BlendMode::Alpha => ColorBlendState::with_attachment_states(
            1,
            ColorBlendAttachmentState {
                blend: Some(AttachmentBlend::alpha()),
                ..Default::default()
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct TestVertex {
        position: [f32; 2],
    }

    #[test]
    fn pipeline_spec_defaults_are_triangle_list_and_opaque() {
        let spec = PipelineSpec::<TestVertex>::default();

        assert_eq!(spec.topology_value(), PrimitiveTopology::TriangleList);
        assert_eq!(spec.blend_mode_value(), BlendMode::Opaque);
    }

    #[test]
    fn geometry_builder_refuses_empty_vertices() {
        let result = validate_geometry_inputs(0, None);

        assert!(result.is_err());
    }

    #[test]
    fn geometry_builder_refuses_index_out_of_bounds() {
        let result = validate_geometry_inputs(3, Some(&[0, 3]));

        assert!(result.is_err());
    }

    #[test]
    fn geometry_builder_accepts_indexed_and_non_indexed() {
        assert!(validate_geometry_inputs(3, None).is_ok());
        assert!(validate_geometry_inputs(3, Some(&[0, 1, 2])).is_ok());
    }

    #[test]
    fn raw_pipeline_refuses_descriptor_layouts() {
        let result = validate_raw_pipeline_layout(true, false);

        assert!(result.is_err());
    }

    #[test]
    fn raw_pipeline_refuses_push_constants() {
        let result = validate_raw_pipeline_layout(false, true);

        assert!(result.is_err());
    }
}
