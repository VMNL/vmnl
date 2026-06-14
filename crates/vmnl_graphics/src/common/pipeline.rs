////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Shared render backend selectors.
////////////////////////////////////////////////////////////////////////////////
/// Backend pipeline selector for draw items.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum PipelineKey {
    /// Default color-only 2D pipeline.
    Color2D,
    /// Default color-only 3D pipeline.
    Color3D,
}

/// Backend material selector for draw items.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum MaterialKey {
    /// Per-vertex color only, without texture sampling.
    VertexColor,
}
