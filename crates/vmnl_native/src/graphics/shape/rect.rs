////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Rectangle shape utilities for the VMNL graphics module,
/// providing functions to create axis-aligned rectangles defined by position, size, and color.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, ShapeKind::Rectangle, VMNLRect, VMNLVertex};
use crate::{Context, VMNLResult, VMNLrbg};

impl Shape {
    /// Create an axis-aligned rectangle described by `VMNLRect` and a single `color`.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `rect`: `VMNLRect` defining position and size.
    /// - `color`: `VMNLrbg` array representing the RGB color.
    ///
    /// # Returns
    /// A `Shape` instance containing the created vertex and index buffers.
    ///
    /// # Example
    /// ```
    /// let rect = VMNLRect {
    ///     position: [100.0, 150.0],
    ///     size: [200.0, 100.0]
    /// };
    /// let color = [255.0, 0.0, 0.0]; // Red color
    /// let rectangle = Shape::create_rectangle(&vmnl_context, rect, color);
    /// // Now `rectangle` can be rendered using the appropriate rendering method.
    /// ```
    pub fn create_rectangle(
        vmnl_context: &Context,
        rect: VMNLRect,
        color: VMNLrbg,
    ) -> VMNLResult<Self> {
        let x0: f32 = rect.position[0];
        let y0: f32 = rect.position[1];
        let x1: f32 = x0 + rect.size[0];
        let y1: f32 = y0 + rect.size[1];
        let vertices: [VMNLVertex; 4] = [
            VMNLVertex {
                position: [x0, y0],
                color,
            }, // top-left
            VMNLVertex {
                position: [x1, y0],
                color,
            }, // top-right
            VMNLVertex {
                position: [x1, y1],
                color,
            }, // bottom-right
            VMNLVertex {
                position: [x0, y1],
                color,
            }, // bottom-left
        ];
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
        let mut graphics: Self =
            Self::create_indexed_shape(vmnl_context, vertices.as_slice(), indices.as_slice())?;

        graphics.kind = Rectangle;
        println!("{}", crate::vmnl_log(format!("Creating rectangle at position [{}, {}] with size [{}, {}] and color [{}, {}, {}].",
            rect.position[0], rect.position[1],
            rect.size[0], rect.size[1],
            color[0], color[1], color[2]
        )));
        Ok(graphics)
    }
}
