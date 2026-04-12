////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Brief
////////////////////////////////////////////////////////////////////////////////

use crate::{Graphics, VMNLVertex, Context, VMNLRect, VMNLrbg};
use crate::graphics::GraphicsKind::Rectangle;

impl Graphics
{
    /**
     * * Create an axis-aligned rectangle shape described by `VMNLRect` and a single `color`.
     *
     * ! Parameters:
     * - `vmnl_context`: A reference to the VMNL context, which provides access to the memory allocator.
     * - `rect`: A `VMNLRect` struct that defines the position and size of the rectangle to be created.
     * - `color`: A `VMNLrbg` array representing the RGB color of the rectangle.
     *
     * ! Returns:
     * - A new instance of the `Graphics` struct, containing the created vertex and index
     */
    pub fn create_rectangle(
        vmnl_context: &Context,
        rect:          VMNLRect,
        color:         VMNLrbg,
    ) -> Self
    {
        let x0: f32 = rect.position[0];
        let y0: f32 = rect.position[1];
        let x1: f32 = x0 + rect.size[0];
        let y1: f32 = y0 + rect.size[1];
        let vertices: [VMNLVertex; 4] = [
            VMNLVertex {
                position: [x0, y0],
                color
            }, // * top-left
            VMNLVertex {
                position: [x1, y0],
                color
            }, // * top-right
            VMNLVertex {
                position: [x1, y1],
                color
            }, // * bottom-right
            VMNLVertex {
                position: [x0, y1],
                color
            }, // * bottom-left
        ];
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
        let mut graphics: Graphics = Self::create_indexed_shape(vmnl_context, &vertices, &indices);

        graphics.kind = Rectangle;
        println!("{}", crate::vmnl_log(&format!("Creating rectangle at position [{}, {}] with size [{}, {}] and color [{}, {}, {}].",
            rect.position[0], rect.position[1],
            rect.size[0], rect.size[1],
            color[0], color[1], color[2]
        )));
        return graphics;
    }
}
