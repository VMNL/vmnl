// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Low-level raw rendering API.

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

pub use vmnl_macros::{Pod, Vertex, Zeroable};

use vulkano::buffer::BufferContents as VulkanoBufferContents;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::Subbuffer;
use vulkano::descriptor_set::layout::{
    DescriptorSetLayout, DescriptorSetLayoutCreateInfo, DescriptorType,
};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
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
    DynamicState, GraphicsPipeline, Pipeline as VulkanoPipeline, PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::sync::HostAccessError;

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
///
/// This is a facade adapter for Vulkano's buffer-content contract. Incorrect
/// unsafe implementations of the underlying trait can make byte transfers
/// unsound; prefer VMNL's marker derives where applicable.
pub trait BufferContents: VulkanoBufferContents {}

impl<T> BufferContents for T where T: VulkanoBufferContents {}

/// Marker trait for raw vertex layouts.
///
/// The layout must match the vertex shader interface. Pipeline construction
/// validates the generated definition and returns an error on a mismatch.
pub trait Vertex: VulkanoVertex {}

impl<T> Vertex for T where T: VulkanoVertex {}

/// Marker trait for plain-old-data raw values.
///
/// This inherits bytemuck's safety contract: all bit patterns are valid and
/// the representation contains no uninitialized padding. Prefer `#[derive(Pod)]`.
pub trait Pod: bytemuck::Pod {}

impl<T> Pod for T where T: bytemuck::Pod {}

/// Marker trait for zero-initializable raw values.
///
/// This inherits bytemuck's requirement that the all-zero byte pattern is
/// valid. It does not by itself prove the stronger `Pod` contract.
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
    /// unsupported resources or push constants, or if Vulkan pipeline creation fails.
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
        validate_raw_pipeline_layout(&layout_info)?;

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
        self.render_item_with_resources(geometry, None)
    }

    pub(crate) fn render_item_with(
        &self,
        geometry: &Geometry<TVertex>,
        resources: &Resources,
    ) -> RenderItemRaw {
        self.render_item_with_resources(geometry, Some(resources))
    }

    fn render_item_with_resources(
        &self,
        geometry: &Geometry<TVertex>,
        resources: Option<&Resources>,
    ) -> RenderItemRaw {
        RenderItemRaw {
            pipeline: self.inner.clone(),
            pipeline_device: self.device.clone(),
            pipeline_render_pass: self.render_pass.clone(),
            geometry_device: geometry.device.clone(),
            resources_device: resources.map(|resources| resources.device.clone()),
            resources_pipeline_layout: resources.map(|resources| resources.pipeline_layout.clone()),
            descriptor_sets: resources.map_or_else(Vec::new, Resources::descriptor_sets),
            required_descriptor_set_count: required_descriptor_set_count(self.inner.layout()),
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

/// Typed raw uniform buffer.
pub struct Uniform<TData> {
    buffer: Subbuffer<TData>,
    device: Arc<Device>,
}

impl<TData> Uniform<TData> {
    /// Starts a raw uniform buffer builder.
    #[must_use]
    pub fn builder(data: TData) -> UniformBuilder<TData> {
        UniformBuilder {
            data,
            memory_preference: BufferMemoryPreference::default(),
        }
    }

    /// Writes a new value into the existing uniform buffer.
    ///
    /// This updates the same buffer object that already-built [`Resources`]
    /// reference. It does not recreate descriptor sets, submit GPU work, or wait
    /// for in-flight GPU access.
    ///
    /// # Errors
    /// Returns `InvalidState` if the buffer is currently locked or still used by
    /// the GPU. Returns a Vulkan validation error for other backend write
    /// failures.
    pub fn write(&mut self, data: TData) -> VMNLResult<()>
    where
        TData: BufferContents,
    {
        {
            let mut guard = self
                .buffer
                .write()
                .map_err(|error| map_uniform_write_error(&error))?;
            *guard = data;
        }

        Ok(())
    }

    fn buffer_bytes(&self) -> Subbuffer<[u8]> {
        self.buffer.clone().into_bytes()
    }
}

/// Builder for typed raw uniform buffers.
pub struct UniformBuilder<TData> {
    data: TData,
    memory_preference: BufferMemoryPreference,
}

impl<TData> UniformBuilder<TData> {
    /// Sets the buffer memory preference.
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.memory_preference = preference;
        self
    }

    /// Builds a GPU uniform buffer.
    ///
    /// # Errors
    /// Returns an error if GPU buffer allocation fails.
    pub fn build(self, context: &Context) -> VMNLResult<Uniform<TData>>
    where
        TData: BufferContents,
    {
        let buffer = <Self as GraphicsResourceFactory>::create_buffer_from_data(
            self.data,
            BufferUsage::UNIFORM_BUFFER,
            self.memory_preference,
            &context.inner.memory_allocator,
            VMNLErrorKind::VulkanFrameUboBufferCreationFailed,
        )?;

        Ok(Uniform {
            buffer,
            device: context.inner.device.clone(),
        })
    }
}

impl<TData> GraphicsResourceFactory for UniformBuilder<TData> {}

/// Descriptor resources bound by a raw draw call.
pub struct Resources {
    device: Arc<Device>,
    pipeline_layout: Arc<PipelineLayout>,
    descriptor_sets: Vec<Arc<DescriptorSet>>,
}

impl Resources {
    /// Starts a raw resource builder for a specific pipeline layout.
    #[must_use]
    pub fn builder<TVertex>(pipeline: &Pipeline<TVertex>) -> ResourcesBuilder {
        let pipeline_layout = pipeline.inner.layout().clone();
        ResourcesBuilder {
            pipeline_device: pipeline.device.clone(),
            pipeline_layout: pipeline_layout.clone(),
            set_layouts: pipeline_layout.set_layouts().to_vec(),
            bindings: BTreeMap::new(),
            duplicate_binding: None,
        }
    }

    fn descriptor_sets(&self) -> Vec<Arc<DescriptorSet>> {
        self.descriptor_sets.clone()
    }
}

/// Builder for raw descriptor resources.
pub struct ResourcesBuilder {
    pipeline_device: Arc<Device>,
    pipeline_layout: Arc<PipelineLayout>,
    set_layouts: Vec<Arc<DescriptorSetLayout>>,
    bindings: BTreeMap<u32, BTreeMap<u32, ResourceBinding>>,
    duplicate_binding: Option<(u32, u32)>,
}

impl ResourcesBuilder {
    /// Binds a uniform buffer to a shader descriptor binding.
    #[must_use]
    pub fn uniform<TData>(mut self, set: u32, binding: u32, uniform: &Uniform<TData>) -> Self {
        let old = self.bindings.entry(set).or_default().insert(
            binding,
            ResourceBinding::UniformBuffer {
                buffer: uniform.buffer_bytes(),
                device: uniform.device.clone(),
            },
        );
        if old.is_some() && self.duplicate_binding.is_none() {
            self.duplicate_binding = Some((set, binding));
        }
        self
    }

    /// Builds descriptor sets compatible with the pipeline used by this builder.
    ///
    /// # Errors
    /// Returns an error if a required binding is missing, unsupported, duplicated,
    /// or if descriptor set allocation fails.
    pub fn build(mut self, context: &Context) -> VMNLResult<Resources> {
        validate_resources_context(context, &self.pipeline_device)?;
        if let Some((set, binding)) = self.duplicate_binding {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "raw resources duplicate binding set {set} binding {binding}"
            ))));
        }

        validate_supplied_resource_bindings(&self.set_layouts, &self.bindings)?;
        let required_set_count = required_descriptor_set_count_from_layouts(&self.set_layouts);
        let mut descriptor_sets = Vec::with_capacity(required_set_count);

        for set_index in 0..required_set_count {
            let set = u32::try_from(set_index)
                .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
            let set_layout = self
                .set_layouts
                .get(set_index)
                .cloned()
                .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
            let supplied_bindings = self.bindings.remove(&set).unwrap_or_default();
            let writes = descriptor_writes_for_set(context, set, &set_layout, &supplied_bindings)?;
            let descriptor_set = DescriptorSet::new(
                context.inner.descriptor_set_allocator.clone(),
                set_layout,
                writes,
                Vec::new(),
            )
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanDescriptorSetCreationFailed))?;
            descriptor_sets.push(descriptor_set);
        }

        if let Some((&set, _)) = self.bindings.iter().next() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "raw resources set {set} is not declared by the pipeline"
            ))));
        }

        Ok(Resources {
            device: context.inner.device.clone(),
            pipeline_layout: self.pipeline_layout,
            descriptor_sets,
        })
    }
}

#[derive(Clone)]
enum ResourceBinding {
    UniformBuffer {
        buffer: Subbuffer<[u8]>,
        device: Arc<Device>,
    },
}

#[derive(Clone)]
pub(crate) struct RenderItemRaw {
    pub(crate) pipeline: Arc<GraphicsPipeline>,
    pub(crate) pipeline_device: Arc<Device>,
    pub(crate) pipeline_render_pass: Arc<RenderPass>,
    pub(crate) geometry_device: Arc<Device>,
    pub(crate) resources_device: Option<Arc<Device>>,
    pub(crate) resources_pipeline_layout: Option<Arc<PipelineLayout>>,
    pub(crate) descriptor_sets: Vec<Arc<DescriptorSet>>,
    pub(crate) required_descriptor_set_count: usize,
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

fn validate_resources_context(context: &Context, expected_device: &Arc<Device>) -> VMNLResult<()> {
    if !Arc::ptr_eq(&context.inner.device, expected_device) {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "raw resources must be built from the same context as the raw pipeline".into(),
        )));
    }

    Ok(())
}

fn map_uniform_write_error(error: &HostAccessError) -> VMNLError {
    match error {
        HostAccessError::AccessConflict(_) => VMNLError::new(VMNLErrorKind::InvalidState(
            "raw uniform write conflicts with active CPU or GPU access".into(),
        )),
        _ => VMNLError::new(VMNLErrorKind::VulkanValidationFailed),
    }
}

fn validate_supplied_resource_bindings(
    set_layouts: &[Arc<DescriptorSetLayout>],
    supplied_sets: &BTreeMap<u32, BTreeMap<u32, ResourceBinding>>,
) -> VMNLResult<()> {
    for (&set, supplied_bindings) in supplied_sets {
        let set_layout = set_layouts.get(usize::try_from(set).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "raw resources set index out of bounds".into(),
            ))
        })?);
        let Some(set_layout) = set_layout else {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "raw resources set {set} is not declared by the pipeline"
            ))));
        };

        for &binding in supplied_bindings.keys() {
            if !set_layout.bindings().contains_key(&binding) {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                    "raw resources set {set} binding {binding} is not declared by the pipeline"
                ))));
            }
        }
    }

    Ok(())
}

fn descriptor_writes_for_set(
    context: &Context,
    set: u32,
    set_layout: &Arc<DescriptorSetLayout>,
    supplied_bindings: &BTreeMap<u32, ResourceBinding>,
) -> VMNLResult<Vec<WriteDescriptorSet>> {
    let mut writes = Vec::with_capacity(supplied_bindings.len());

    for (&binding, binding_layout) in set_layout.bindings() {
        validate_raw_descriptor_binding(set, binding, binding_layout.descriptor_type)?;
        if binding_layout.descriptor_count != 1 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "raw resources set {set} binding {binding} descriptor arrays are not supported yet"
            ))));
        }

        let resource = supplied_bindings.get(&binding).ok_or_else(|| {
            VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "raw resources missing set {set} binding {binding}"
            )))
        })?;

        match resource {
            ResourceBinding::UniformBuffer { buffer, device } => {
                if !Arc::ptr_eq(device, &context.inner.device) {
                    return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                        "raw resources set {set} binding {binding} must belong to this context"
                    ))));
                }
                writes.push(WriteDescriptorSet::buffer(binding, buffer.clone()));
            }
        }
    }

    Ok(writes)
}

fn validate_raw_pipeline_layout(
    layout_info: &PipelineDescriptorSetLayoutCreateInfo,
) -> VMNLResult<()> {
    if !layout_info.push_constant_ranges.is_empty() {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "raw pipeline push constants are not supported yet".into(),
        )));
    }

    for (set, set_layout) in layout_info.set_layouts.iter().enumerate() {
        let set = u32::try_from(set)
            .map_err(|_| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
        validate_raw_descriptor_set_layout(set, set_layout)?;
    }

    Ok(())
}

fn validate_raw_descriptor_set_layout(
    set: u32,
    set_layout: &DescriptorSetLayoutCreateInfo,
) -> VMNLResult<()> {
    for (&binding, binding_layout) in &set_layout.bindings {
        validate_raw_descriptor_binding(set, binding, binding_layout.descriptor_type)?;
        if binding_layout.descriptor_count != 1 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
                "raw pipeline set {set} binding {binding} descriptor arrays are not supported yet"
            ))));
        }
    }

    Ok(())
}

fn validate_raw_descriptor_binding(
    set: u32,
    binding: u32,
    descriptor_type: DescriptorType,
) -> VMNLResult<()> {
    if descriptor_type != DescriptorType::UniformBuffer {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "raw pipeline set {set} binding {binding} only supports uniform buffers for now"
        ))));
    }

    Ok(())
}

fn required_descriptor_set_count(layout: &PipelineLayout) -> usize {
    required_descriptor_set_count_from_layouts(layout.set_layouts())
}

fn required_descriptor_set_count_from_layouts(set_layouts: &[Arc<DescriptorSetLayout>]) -> usize {
    set_layouts
        .iter()
        .rposition(|set_layout| !set_layout.bindings().is_empty())
        .map_or(0, |index| index + 1)
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
    use vulkano::descriptor_set::layout::DescriptorSetLayoutBinding;
    use vulkano::pipeline::layout::{PipelineLayoutCreateFlags, PushConstantRange};
    use vulkano::shader::ShaderStages;
    use vulkano::sync::AccessConflict;

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
    fn raw_pipeline_accepts_uniform_descriptor_layouts() {
        let layout_info = pipeline_layout_info(vec![descriptor_set_layout(
            DescriptorType::UniformBuffer,
            1,
        )]);

        assert!(validate_raw_pipeline_layout(&layout_info).is_ok());
    }

    #[test]
    fn raw_pipeline_refuses_unsupported_descriptor_layouts() {
        let layout_info = pipeline_layout_info(vec![descriptor_set_layout(
            DescriptorType::CombinedImageSampler,
            1,
        )]);

        assert!(validate_raw_pipeline_layout(&layout_info).is_err());
    }

    #[test]
    fn raw_pipeline_refuses_descriptor_arrays() {
        let layout_info = pipeline_layout_info(vec![descriptor_set_layout(
            DescriptorType::UniformBuffer,
            2,
        )]);

        assert!(validate_raw_pipeline_layout(&layout_info).is_err());
    }

    #[test]
    fn raw_pipeline_refuses_push_constants() {
        let layout_info = PipelineDescriptorSetLayoutCreateInfo {
            flags: PipelineLayoutCreateFlags::empty(),
            set_layouts: Vec::new(),
            push_constant_ranges: vec![PushConstantRange {
                stages: ShaderStages::VERTEX,
                offset: 0,
                size: 4,
            }],
        };

        assert!(validate_raw_pipeline_layout(&layout_info).is_err());
    }

    #[test]
    fn uniform_write_conflict_maps_to_invalid_state() {
        let error =
            map_uniform_write_error(&HostAccessError::AccessConflict(AccessConflict::DeviceRead));

        assert!(matches!(
            error.kind(),
            VMNLErrorKind::InvalidState(message)
                if message == "raw uniform write conflicts with active CPU or GPU access"
        ));
    }

    #[test]
    fn uniform_write_backend_error_maps_to_vulkan_validation() {
        let error = map_uniform_write_error(&HostAccessError::NotHostMapped);

        assert!(matches!(
            error.kind(),
            VMNLErrorKind::VulkanValidationFailed
        ));
    }

    fn pipeline_layout_info(
        set_layouts: Vec<DescriptorSetLayoutCreateInfo>,
    ) -> PipelineDescriptorSetLayoutCreateInfo {
        PipelineDescriptorSetLayoutCreateInfo {
            flags: PipelineLayoutCreateFlags::empty(),
            set_layouts,
            push_constant_ranges: Vec::new(),
        }
    }

    fn descriptor_set_layout(
        descriptor_type: DescriptorType,
        descriptor_count: u32,
    ) -> DescriptorSetLayoutCreateInfo {
        let mut layout = DescriptorSetLayoutCreateInfo::default();
        let mut binding = DescriptorSetLayoutBinding::descriptor_type(descriptor_type);
        binding.stages = ShaderStages::VERTEX;
        binding.descriptor_count = descriptor_count;
        layout.bindings.insert(0, binding);
        layout
    }
}
