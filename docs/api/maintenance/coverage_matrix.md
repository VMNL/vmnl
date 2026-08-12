# Public API coverage matrix

This matrix has one row per distinct public type, trait, alias, or derive macro reachable through `vmnl`. Reexport aliases of the same type, such as the three `ShaderSource` paths, share one row. Blanket implementations are the only public-surface entries intentionally omitted.

| Kind | Symbol | Canonical page | Rustdoc | Evidence |
|---|---|---|---|---|
| alias | `vmnl::VMNLResult` | [VMNLResult](../reference/errors/vmnl_result.md) | [Rustdoc](../../../target/doc/vmnl/type.VMNLResult.html) | [API errors](../../../tests/api/tests/errors.rs) |
| enum | `vmnl::Event` | [Event](../reference/window/events/event.md) | [Rustdoc](../../../target/doc/vmnl/enum.Event.html) | [event input example](../../../examples/window/events_input/src/main.rs) |
| enum | `vmnl::Key` | [Key](../reference/window/input/key.md) | [Rustdoc](../../../target/doc/vmnl/enum.Key.html) | [API input](../../../tests/api/tests/input.rs) |
| enum | `vmnl::MouseButton` | [MouseButton](../reference/window/input/mouse_button.md) | [Rustdoc](../../../target/doc/vmnl/enum.MouseButton.html) | [API input](../../../tests/api/tests/input.rs) |
| enum | `vmnl::PresentMode` | [PresentMode](../reference/window/present_mode.md) | [Rustdoc](../../../target/doc/vmnl/enum.PresentMode.html) | [API raw spec](../../../tests/api/tests/raw_spec.rs) |
| enum | `vmnl::RenderMode` | [RenderMode](../reference/window/rendering/render_mode.md) | [Rustdoc](../../../target/doc/vmnl/enum.RenderMode.html) | [API raw spec](../../../tests/api/tests/raw_spec.rs) |
| enum | `vmnl::VMNLErrorKind` | [VMNLErrorKind](../reference/errors/vmnl_error_kind.md) | [Rustdoc](../../../target/doc/vmnl/enum.VMNLErrorKind.html) | [API errors](../../../tests/api/tests/errors.rs) |
| enum | `vmnl::common::BufferMemoryPreference` | [BufferMemoryPreference](../reference/common/buffer_memory_preference.md) | [Rustdoc](../../../target/doc/vmnl/common/enum.BufferMemoryPreference.html) | [API raw spec](../../../tests/api/tests/raw_spec.rs) |
| enum | `vmnl::common::ShaderSource` | [ShaderSource](../reference/common/shader_source.md) | [Rustdoc](../../../target/doc/vmnl/common/enum.ShaderSource.html) | [API raw spec](../../../tests/api/tests/raw_spec.rs) |
| enum | `vmnl::d2::Anchor` | [Anchor](../reference/d2/shapes/anchor.md) | [Rustdoc](../../../target/doc/vmnl/d2/enum.Anchor.html) | [2D shapes example](../../../examples/d2/shapes/src/main.rs) |
| enum | `vmnl::d2::LineCap` | [LineCap](../reference/d2/shapes/line_cap.md) | [Rustdoc](../../../target/doc/vmnl/d2/enum.LineCap.html) | [2D shapes example](../../../examples/d2/shapes/src/main.rs) |
| enum | `vmnl::raw::BlendMode` | [BlendMode](../reference/raw/pipeline/blend_mode.md) | [Rustdoc](../../../target/doc/vmnl/raw/enum.BlendMode.html) | [API raw spec](../../../tests/api/tests/raw_spec.rs) |
| enum | `vmnl::raw::PrimitiveTopology` | [PrimitiveTopology](../reference/raw/pipeline/primitive_topology.md) | [Rustdoc](../../../target/doc/vmnl/raw/enum.PrimitiveTopology.html) | [API raw spec](../../../tests/api/tests/raw_spec.rs) |
| macro | `vmnl::raw::Pod` | [Pod derive](../reference/raw/traits/derive_macros.md) | [Rustdoc](../../../target/doc/vmnl/raw/derive.Pod.html) | [API raw traits](../../../tests/api/tests/raw_traits.rs) |
| macro | `vmnl::raw::Vertex` | [Vertex derive](../reference/raw/traits/derive_macros.md) | [Rustdoc](../../../target/doc/vmnl/raw/derive.Vertex.html) | [API raw traits](../../../tests/api/tests/raw_traits.rs) |
| macro | `vmnl::raw::Zeroable` | [Zeroable derive](../reference/raw/traits/derive_macros.md) | [Rustdoc](../../../target/doc/vmnl/raw/derive.Zeroable.html) | [API raw traits](../../../tests/api/tests/raw_traits.rs) |
| struct | `vmnl::Context` | [Context](../reference/context.md) | [Rustdoc](../../../target/doc/vmnl/struct.Context.html) | [GPU context](../../../tests/gpu/tests/context.rs) |
| struct | `vmnl::FrameRenderer` | [FrameRenderer](../reference/window/rendering/frame_renderer.md) | [Rustdoc](../../../target/doc/vmnl/struct.FrameRenderer.html) | [GPU frame renderer](../../../tests/gpu/tests/frame_renderer.rs) |
| struct | `vmnl::Input` | [Input](../reference/window/input/input.md) | [Rustdoc](../../../target/doc/vmnl/struct.Input.html) | [API input](../../../tests/api/tests/input.rs) |
| struct | `vmnl::KeyboardState` | [KeyboardState](../reference/window/input/keyboard_state.md) | [Rustdoc](../../../target/doc/vmnl/struct.KeyboardState.html) | [API input](../../../tests/api/tests/input.rs) |
| struct | `vmnl::MonitorInfo` | [MonitorInfo](../reference/window/monitors/monitor_info.md) | [Rustdoc](../../../target/doc/vmnl/struct.MonitorInfo.html) | [GPU window runtime](../../../tests/gpu/tests/window_runtime.rs) |
| struct | `vmnl::Monitors` | [Monitors](../reference/window/monitors/monitors.md) | [Rustdoc](../../../target/doc/vmnl/struct.Monitors.html) | [GPU window runtime](../../../tests/gpu/tests/window_runtime.rs) |
| struct | `vmnl::MouseState` | [MouseState](../reference/window/input/mouse_state.md) | [Rustdoc](../../../target/doc/vmnl/struct.MouseState.html) | [API input](../../../tests/api/tests/input.rs) |
| struct | `vmnl::VMNLError` | [VMNLError](../reference/errors/vmnl_error.md) | [Rustdoc](../../../target/doc/vmnl/struct.VMNLError.html) | [API errors](../../../tests/api/tests/errors.rs) |
| struct | `vmnl::VMNLErrorLocation` | [VMNLErrorLocation](../reference/errors/vmnl_error_location.md) | [Rustdoc](../../../target/doc/vmnl/struct.VMNLErrorLocation.html) | [API errors](../../../tests/api/tests/errors.rs) |
| struct | `vmnl::VideoMode` | [VideoMode](../reference/window/monitors/video_mode.md) | [Rustdoc](../../../target/doc/vmnl/struct.VideoMode.html) | [GPU window runtime](../../../tests/gpu/tests/window_runtime.rs) |
| struct | `vmnl::Window` | [Window](../reference/window/window.md) | [Rustdoc](../../../target/doc/vmnl/struct.Window.html) | [GPU window runtime](../../../tests/gpu/tests/window_runtime.rs) |
| struct | `vmnl::WindowBuilder` | [WindowBuilder](../reference/window/window_builder.md) | [Rustdoc](../../../target/doc/vmnl/struct.WindowBuilder.html) | [API errors](../../../tests/api/tests/errors.rs) |
| struct | `vmnl::common::Rgba` | [Rgba](../reference/common/rgba.md) | [Rustdoc](../../../target/doc/vmnl/common/struct.Rgba.html) | [API common types](../../../tests/api/tests/common_types.rs) |
| struct | `vmnl::d2::IndexedShapeBuilder` | [IndexedShapeBuilder](../reference/d2/shapes/indexed_shape_builder.md) | [Rustdoc](../../../target/doc/vmnl/d2/struct.IndexedShapeBuilder.html) | [advanced geometry example](../../../examples/d2/advanced_geometry/src/main.rs) |
| struct | `vmnl::d2::LineBuilder` | [LineBuilder](../reference/d2/shapes/line_builder.md) | [Rustdoc](../../../target/doc/vmnl/d2/struct.LineBuilder.html) | [2D shapes example](../../../examples/d2/shapes/src/main.rs) |
| struct | `vmnl::d2::RectBuilder` | [RectBuilder](../reference/d2/shapes/rect_builder.md) | [Rustdoc](../../../target/doc/vmnl/d2/struct.RectBuilder.html) | [2D shapes example](../../../examples/d2/shapes/src/main.rs) |
| struct | `vmnl::d2::RenderItem2D` | [RenderItem2D](../reference/d2/render_item_2d.md) | [Rustdoc](../../../target/doc/vmnl/d2/struct.RenderItem2D.html) | [GPU geometry](../../../tests/gpu/tests/gpu_geometry.rs) |
| struct | `vmnl::d2::Shape` | [Shape](../reference/d2/shapes/shape.md) | [Rustdoc](../../../target/doc/vmnl/d2/struct.Shape.html) | [2D shapes example](../../../examples/d2/shapes/src/main.rs) |
| struct | `vmnl::d2::TriangleBuilder` | [TriangleBuilder](../reference/d2/shapes/triangle_builder.md) | [Rustdoc](../../../target/doc/vmnl/d2/struct.TriangleBuilder.html) | [2D shapes example](../../../examples/d2/shapes/src/main.rs) |
| struct | `vmnl::d2::Vector2f` | [Vector2f](../reference/d2/vector_2f.md) | [Rustdoc](../../../target/doc/vmnl/d2/struct.Vector2f.html) | [API common types](../../../tests/api/tests/common_types.rs) |
| struct | `vmnl::d2::Vertex2D` | [Vertex2D](../reference/d2/vertex_2d.md) | [Rustdoc](../../../target/doc/vmnl/d2/struct.Vertex2D.html) | [API common types](../../../tests/api/tests/common_types.rs) |
| struct | `vmnl::d3::Camera` | [Camera](../reference/d3/camera.md) | [Rustdoc](../../../target/doc/vmnl/d3/struct.Camera.html) | [API 3D types](../../../tests/api/tests/d3_types.rs) |
| struct | `vmnl::d3::Mesh` | [Mesh](../reference/d3/mesh/mesh.md) | [Rustdoc](../../../target/doc/vmnl/d3/struct.Mesh.html) | [GPU 3D scaffold](../../../tests/gpu/tests/d3_scaffold.rs) |
| struct | `vmnl::d3::MeshBuilder` | [MeshBuilder](../reference/d3/mesh/mesh_builder.md) | [Rustdoc](../../../target/doc/vmnl/d3/struct.MeshBuilder.html) | [GPU 3D scaffold](../../../tests/gpu/tests/d3_scaffold.rs) |
| struct | `vmnl::d3::RenderItem3D` | [RenderItem3D](../reference/d3/render_item_3d.md) | [Rustdoc](../../../target/doc/vmnl/d3/struct.RenderItem3D.html) | [GPU 3D scaffold](../../../tests/gpu/tests/d3_scaffold.rs) |
| struct | `vmnl::d3::Vector3f` | [Vector3f](../reference/d3/vector_3f.md) | [Rustdoc](../../../target/doc/vmnl/d3/struct.Vector3f.html) | [API 3D types](../../../tests/api/tests/d3_types.rs) |
| struct | `vmnl::d3::Vertex3D` | [Vertex3D](../reference/d3/vertex_3d.md) | [Rustdoc](../../../target/doc/vmnl/d3/struct.Vertex3D.html) | [API 3D types](../../../tests/api/tests/d3_types.rs) |
| struct | `vmnl::raw::Geometry` | [Geometry](../reference/raw/geometry/geometry.md) | [Rustdoc](../../../target/doc/vmnl/raw/struct.Geometry.html) | [raw triangle example](../../../examples/raw/triangle/src/main.rs) |
| struct | `vmnl::raw::GeometryBuilder` | [GeometryBuilder](../reference/raw/geometry/geometry_builder.md) | [Rustdoc](../../../target/doc/vmnl/raw/struct.GeometryBuilder.html) | [raw triangle example](../../../examples/raw/triangle/src/main.rs) |
| struct | `vmnl::raw::Pipeline` | [Pipeline](../reference/raw/pipeline/pipeline.md) | [Rustdoc](../../../target/doc/vmnl/raw/struct.Pipeline.html) | [raw pipeline example](../../../examples/raw/pipeline/src/main.rs) |
| struct | `vmnl::raw::PipelineSpec` | [PipelineSpec](../reference/raw/pipeline/pipeline_spec.md) | [Rustdoc](../../../target/doc/vmnl/raw/struct.PipelineSpec.html) | [API raw spec](../../../tests/api/tests/raw_spec.rs) |
| struct | `vmnl::raw::Resources` | [Resources](../reference/raw/resources/resources.md) | [Rustdoc](../../../target/doc/vmnl/raw/struct.Resources.html) | [raw uniform example](../../../examples/raw/uniform/src/main.rs) |
| struct | `vmnl::raw::ResourcesBuilder` | [ResourcesBuilder](../reference/raw/resources/resources_builder.md) | [Rustdoc](../../../target/doc/vmnl/raw/struct.ResourcesBuilder.html) | [raw uniform example](../../../examples/raw/uniform/src/main.rs) |
| struct | `vmnl::raw::Uniform` | [Uniform](../reference/raw/uniforms/uniform.md) | [Rustdoc](../../../target/doc/vmnl/raw/struct.Uniform.html) | [raw uniform example](../../../examples/raw/uniform/src/main.rs) |
| struct | `vmnl::raw::UniformBuilder` | [UniformBuilder](../reference/raw/uniforms/uniform_builder.md) | [Rustdoc](../../../target/doc/vmnl/raw/struct.UniformBuilder.html) | [raw uniform example](../../../examples/raw/uniform/src/main.rs) |
| trait | `vmnl::d2::Drawable2D` | [Drawable2D](../reference/d2/drawable_2d.md) | [Rustdoc](../../../target/doc/vmnl/d2/trait.Drawable2D.html) | [GPU frame renderer](../../../tests/gpu/tests/frame_renderer.rs) |
| trait | `vmnl::d3::Drawable3D` | [Drawable3D](../reference/d3/drawable_3d.md) | [Rustdoc](../../../target/doc/vmnl/d3/trait.Drawable3D.html) | [GPU 3D scaffold](../../../tests/gpu/tests/d3_scaffold.rs) |
| trait | `vmnl::raw::BufferContents` | [BufferContents](../reference/raw/traits/buffer_contents.md) | [Rustdoc](../../../target/doc/vmnl/raw/trait.BufferContents.html) | [API raw traits](../../../tests/api/tests/raw_traits.rs) |
| trait | `vmnl::raw::Pod` | [Pod trait](../reference/raw/traits/pod.md) | [Rustdoc](../../../target/doc/vmnl/raw/trait.Pod.html) | [API raw traits](../../../tests/api/tests/raw_traits.rs) |
| trait | `vmnl::raw::Vertex` | [Vertex trait](../reference/raw/traits/vertex.md) | [Rustdoc](../../../target/doc/vmnl/raw/trait.Vertex.html) | [API raw traits](../../../tests/api/tests/raw_traits.rs) |
| trait | `vmnl::raw::Zeroable` | [Zeroable trait](../reference/raw/traits/zeroable.md) | [Rustdoc](../../../target/doc/vmnl/raw/trait.Zeroable.html) | [API raw traits](../../../tests/api/tests/raw_traits.rs) |

Coverage is complete only when `tools/api_docs.py check` reports no missing, duplicate, stale, or unresolvable entry.
