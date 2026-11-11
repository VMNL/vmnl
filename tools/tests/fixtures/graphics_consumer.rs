// SPDX-FileCopyrightText: 2026 VMNL contributors
// SPDX-License-Identifier: MIT

//! External-client contract; GRAPHICS_DEP is replaced with the manifest dependency name.

use GRAPHICS_DEP::{raw, Context, ContextBuilder, VMNLResult};

#[repr(C)]
#[derive(Clone, Copy, raw::Vertex, raw::Pod, raw::Zeroable)]
struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, raw::Pod, raw::Zeroable)]
struct Uniform {
    color: [f32; 4],
}

fn vertex_contract<T: raw::Vertex + raw::Pod + raw::Zeroable>() {}
fn uniform_contract<T: raw::BufferContents + raw::Pod + raw::Zeroable>() {}

fn main() {
    vertex_contract::<Vertex>();
    uniform_contract::<Uniform>();
    let _builder: fn() -> ContextBuilder = Context::builder;
    let result: VMNLResult<()> = Ok(());
    assert!(result.is_ok());
    let point = GRAPHICS_DEP::d2::Vector2f { x: 1.0, y: 2.0 };
    assert_eq!(point.x + point.y, 3.0);
}
