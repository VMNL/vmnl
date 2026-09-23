// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shared geometry descriptors and validation helpers.

use super::{IndexBuffer, VertexBuffer};
use crate::{VMNLError, VMNLErrorKind, VMNLResult};

/// GPU-side geometry owned by a renderable resource.
#[derive(Clone)]
pub(crate) struct GpuGeometry<T> {
    /// Vertex buffer consumed by the active graphics pipeline.
    pub(crate) vertex_buffer: VertexBuffer<T>,
    /// Optional index buffer for indexed draws.
    pub(crate) index_buffer: Option<IndexBuffer>,
    /// Number of vertices to draw when no index buffer is present.
    pub(crate) vertex_count: u32,
    /// Number of indices when an index buffer is present.
    pub(crate) index_count: u32,
}

/// Validate a triangle index list against the number of available vertices.
fn validate_triangle_indices(vertex_count: usize, indices: &[u32], label: &str) -> VMNLResult<()> {
    if vertex_count < 3 {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "{label} requires at least 3 vertices"
        ))));
    }
    if indices.len() < 3 || !indices.len().is_multiple_of(3) {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "{label} requires a non-empty triangle index list"
        ))));
    }
    if let Some(index) = indices
        .iter()
        .copied()
        .find(|&index| usize::try_from(index).map_or(true, |index| index >= vertex_count))
    {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "{label} index {index} is out of bounds for {vertex_count} vertices"
        ))));
    }
    Ok(())
}

/// Validate indexed triangle geometry and return Vulkan-compatible draw counts.
pub(crate) fn validate_indexed_triangle_geometry(
    vertex_count: usize,
    indices: &[u32],
    label: &str,
) -> VMNLResult<(u32, u32)> {
    validate_triangle_indices(vertex_count, indices, label)?;
    checked_draw_counts(vertex_count, indices.len())
}

/// Convert geometry counts to Vulkan draw counts.
pub(crate) fn checked_draw_counts(
    vertex_count: usize,
    index_count: usize,
) -> VMNLResult<(u32, u32)> {
    Ok((
        u32::try_from(vertex_count).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "vertex count out of bounds".to_string(),
            ))
        })?,
        u32::try_from(index_count).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "index count out of bounds".to_string(),
            ))
        })?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_invalid_state<T>(result: VMNLResult<T>, expected: &str) {
        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == expected)
        ));
    }

    #[test]
    fn validate_indexed_triangle_geometry_returns_draw_counts() {
        assert_eq!(
            validate_indexed_triangle_geometry(3, &[0, 1, 2], "mesh").map_err(|_| ()),
            Ok((3, 3))
        );
    }

    #[test]
    fn validate_indexed_triangle_geometry_rejects_too_few_vertices() {
        assert_invalid_state(
            validate_indexed_triangle_geometry(2, &[0, 1, 2], "indexed shape"),
            "indexed shape requires at least 3 vertices",
        );
    }

    #[test]
    fn validate_indexed_triangle_geometry_rejects_non_triangle_count() {
        assert_invalid_state(
            validate_indexed_triangle_geometry(3, &[], "mesh"),
            "mesh requires a non-empty triangle index list",
        );
        assert_invalid_state(
            validate_indexed_triangle_geometry(3, &[0, 1, 2, 0], "mesh"),
            "mesh requires a non-empty triangle index list",
        );
    }

    #[test]
    fn validate_indexed_triangle_geometry_rejects_out_of_bounds_indices() {
        assert_invalid_state(
            validate_indexed_triangle_geometry(3, &[0, 1, 3], "mesh"),
            "mesh index 3 is out of bounds for 3 vertices",
        );
    }

    #[test]
    fn checked_draw_counts_converts_to_u32() {
        assert_eq!(checked_draw_counts(3, 6).map_err(|_| (0, 0)), Ok((3, 6)));
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn checked_draw_counts_rejects_vertex_count_above_u32() {
        assert_invalid_state(
            checked_draw_counts((u32::MAX as usize) + 1, 0),
            "vertex count out of bounds",
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn checked_draw_counts_rejects_index_count_above_u32() {
        assert_invalid_state(
            checked_draw_counts(3, (u32::MAX as usize) + 1),
            "index count out of bounds",
        );
    }
}
