// SPDX-FileCopyrightText: 2026 Bouhali Naouel
// SPDX-License-Identifier: MIT

//! Headless coverage of the public polygon builder.

use vmnl::{
    common::{BufferMemoryPreference, Rgba},
    d2::{PolygonBuilder, Shape, Vector2f, Vertex2D},
    Context, VMNLResult,
};

#[test]
fn polygon_builder_is_available_through_the_public_facade() {
    let points = [
        Vector2f { x: 0.0, y: 0.0 },
        Vector2f { x: 100.0, y: 0.0 },
        Vector2f { x: 0.0, y: 100.0 },
    ];

    let _: PolygonBuilder = Shape::polygon(points)
        .color(Rgba::CYAN)
        .vertex_colors([
            Rgba::new(255, 0, 0, 255),
            Rgba::new(0, 255, 0, 255),
            Rgba::new(0, 0, 255, 255),
        ])
        .buffer_memory_preference(BufferMemoryPreference::Host);

    let vertices = points.map(|position| Vertex2D {
        position,
        color: Rgba::CYAN,
    });

    let _: PolygonBuilder = Shape::polygon_from_vertices(vertices)
        .vertex_colors([[255, 0, 0], [0, 255, 0], [0, 0, 255]])
        .color(Rgba::CYAN)
        .buffer_memory_preference(BufferMemoryPreference::Device);

    let _: fn(PolygonBuilder, &Context) -> VMNLResult<Shape> = PolygonBuilder::build;
}
