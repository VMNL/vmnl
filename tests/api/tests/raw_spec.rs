// SPDX-FileCopyrightText: 2026 VMNL
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use vmnl::{raw, VMNLResult};

struct RawSpecVertex;

#[test]
fn raw_shader_sources_store_inline_or_path_inputs() -> VMNLResult<()> {
    let inline = raw::ShaderSource::Src("#version 460\nvoid main() {}".to_string());
    let path = raw::ShaderSource::Path(PathBuf::from("shader.vert"));

    assert!(matches!(inline, raw::ShaderSource::Src(source) if source.contains("#version 460")));
    assert!(matches!(path, raw::ShaderSource::Path(path) if path.ends_with("shader.vert")));

    Ok(())
}

#[test]
fn raw_pipeline_spec_exposes_topology_and_blend_mode() -> VMNLResult<()> {
    let topologies = [
        raw::PrimitiveTopology::PointList,
        raw::PrimitiveTopology::LineList,
        raw::PrimitiveTopology::LineStrip,
        raw::PrimitiveTopology::TriangleList,
        raw::PrimitiveTopology::TriangleStrip,
    ];

    for topology in topologies {
        let spec = raw::Pipeline::<RawSpecVertex>::builder()
            .topology(topology)
            .blend_mode(raw::BlendMode::Alpha);
        assert_eq!(spec.topology_value(), topology);
        assert_eq!(spec.blend_mode_value(), raw::BlendMode::Alpha);
    }

    let spec = raw::Pipeline::<RawSpecVertex>::builder().blend_mode(raw::BlendMode::Opaque);
    assert_eq!(spec.topology_value(), raw::PrimitiveTopology::TriangleList);
    assert_eq!(spec.blend_mode_value(), raw::BlendMode::Opaque);

    Ok(())
}
