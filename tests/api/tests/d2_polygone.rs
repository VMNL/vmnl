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

#[test]
fn concave_boundary_uses_the_existing_public_builder_in_both_windings() {
    let mut points = [
        Vector2f { x: 0.0, y: 0.0 },
        Vector2f { x: 100.0, y: 0.0 },
        Vector2f { x: 40.0, y: 40.0 },
        Vector2f { x: 100.0, y: 100.0 },
        Vector2f { x: 0.0, y: 100.0 },
    ];
    for _ in 0..2 {
        let _: PolygonBuilder = Shape::polygon(points)
            .color(Rgba::CYAN)
            .vertex_colors([Rgba::RED, Rgba::GREEN, Rgba::BLUE, Rgba::CYAN, Rgba::WHITE])
            .buffer_memory_preference(BufferMemoryPreference::Host);
        let _: PolygonBuilder = Shape::polygon_from_vertices(points.map(|position| Vertex2D {
            position,
            color: Rgba::rgba(255, 0, 255, 128),
        }));
        points.reverse();
    }
    // Headless configuration proves facade compatibility; CPU acceptance is covered
    // by unit tests, since the public build boundary requires a Vulkan context.
}
