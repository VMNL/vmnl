////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Shared geometry descriptors and validation helpers.
////////////////////////////////////////////////////////////////////////////////
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
pub(crate) fn validate_triangle_indices(
    vertex_count: usize,
    indices: &[u32],
    label: &str,
) -> VMNLResult<()> {
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

    fn assert_invalid_state(result: VMNLResult<()>, expected: &str) {
        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == expected)
        ));
    }

    #[test]
    fn validate_triangle_indices_accepts_valid_indices() {
        assert!(validate_triangle_indices(3, &[0, 1, 2], "mesh").is_ok());
    }

    #[test]
    fn validate_triangle_indices_rejects_too_few_vertices() {
        assert_invalid_state(
            validate_triangle_indices(2, &[0, 1, 2], "mesh"),
            "mesh requires at least 3 vertices",
        );
    }

    #[test]
    fn validate_triangle_indices_rejects_non_triangle_count() {
        assert_invalid_state(
            validate_triangle_indices(3, &[], "mesh"),
            "mesh requires a non-empty triangle index list",
        );
        assert_invalid_state(
            validate_triangle_indices(3, &[0, 1, 2, 0], "mesh"),
            "mesh requires a non-empty triangle index list",
        );
    }

    #[test]
    fn validate_triangle_indices_rejects_out_of_bounds_indices() {
        assert_invalid_state(
            validate_triangle_indices(3, &[0, 1, 3], "mesh"),
            "mesh index 3 is out of bounds for 3 vertices",
        );
    }

    #[test]
    fn checked_draw_counts_converts_to_u32() {
        assert_eq!(checked_draw_counts(3, 6).map_err(|_| (0, 0)), Ok((3, 6)));
    }
}
