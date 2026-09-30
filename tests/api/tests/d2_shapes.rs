// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless public API coverage for 2D shape builders.

use vmnl::{
    common::{BufferMemoryPreference, Rgba},
    d2::{LineCap, LineJoin, PolylineBuilder, Shape, Vector2f},
    Context, VMNLResult,
};

fn point(x: f32, y: f32) -> Vector2f {
    Vector2f { x, y }
}

fn build_polyline(builder: PolylineBuilder, context: &Context) -> VMNLResult<Shape> {
    builder.build(context)
}

#[test]
fn polyline_builder_and_line_join_are_exported_by_the_facade() {
    assert_eq!(LineJoin::default(), LineJoin::Bevel);

    let open: PolylineBuilder =
        Shape::polyline([point(0.0, 0.0), point(10.0, 5.0), point(20.0, 0.0)])
            .width(4.0)
            .cap(LineCap::Round)
            .join(LineJoin::Miter)
            .miter_limit(3.0)
            .segment_colors([Rgba::RED, Rgba::BLUE])
            .buffer_memory_preference(BufferMemoryPreference::Host);

    let closed_gradient: PolylineBuilder =
        Shape::polyline(vec![point(0.0, 0.0), point(10.0, 0.0), point(5.0, 10.0)])
            .join(LineJoin::Round)
            .point_colors([Rgba::RED, Rgba::YELLOW, Rgba::BLUE])
            .closed()
            .color(Rgba::CYAN);

    let _build: fn(PolylineBuilder, &Context) -> VMNLResult<Shape> = build_polyline;
    let _ = (open, closed_gradient);
}
