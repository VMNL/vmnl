// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless public API coverage for 2D shape builders.

use vmnl_graphics::{
    common::{BufferMemoryPreference, Rgba},
    d2::{
        Anchor, CornerRadii, IndexedShapeBuilder, LineCap, LineJoin, PolylineBuilder, RectBuilder,
        Shape, Vector2f, Vertex2D,
    },
    Context, VMNLResult,
};

fn point(x: f32, y: f32) -> Vector2f {
    Vector2f { x, y }
}

fn build_polyline(builder: PolylineBuilder, context: &Context) -> VMNLResult<Shape> {
    builder.build(context)
}

fn build_indexed_shape(builder: IndexedShapeBuilder, context: &Context) -> VMNLResult<Shape> {
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

#[test]
fn indexed_shape_builder_accepts_public_array_and_vector_inputs() {
    let vertices = [
        Vertex2D {
            position: point(0.0, 0.0),
            color: Rgba::RED,
        },
        Vertex2D {
            position: point(10.0, 0.0),
            color: Rgba::GREEN,
        },
        Vertex2D {
            position: point(0.0, 10.0),
            color: Rgba::BLUE,
        },
    ];

    let from_arrays: IndexedShapeBuilder =
        Shape::indexed(vertices, [0, 1, 2]).buffer_memory_preference(BufferMemoryPreference::Host);
    let from_vectors: IndexedShapeBuilder = Shape::indexed(vertices.to_vec(), vec![0, 1, 2])
        .buffer_memory_preference(BufferMemoryPreference::Host);

    let _build_signature: fn(IndexedShapeBuilder, &Context) -> VMNLResult<Shape> =
        build_indexed_shape;
    let _ = (from_arrays, from_vectors);
}

#[test]
fn corner_radii_and_rect_builder_corner_methods_are_exported_by_the_facade() {
    assert_eq!(CornerRadii::default(), CornerRadii::all(0.0));
    assert_eq!(CornerRadii::all(4.0), CornerRadii::new(4.0, 4.0, 4.0, 4.0));

    let uniform: RectBuilder = Shape::rect(240.0, 120.0)
        .position(100.0, 80.0)
        .corner_radius(16.0)
        .corner_segments(8)
        .color(Rgba::CYAN)
        .anchor(Anchor::Center)
        .rotation(20.0)
        .buffer_memory_preference(BufferMemoryPreference::Host);

    let independent: RectBuilder =
        Shape::rect(240.0, 120.0).corner_radii(CornerRadii::new(24.0, 8.0, 32.0, 0.0));

    let _: fn(RectBuilder, &Context) -> VMNLResult<Shape> = RectBuilder::build;
    let _ = (uniform, independent);
}
