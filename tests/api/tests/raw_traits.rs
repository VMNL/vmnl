// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public raw derive-macro compatibility contracts.

use vmnl::{raw, VMNLResult};

#[repr(C)]
#[derive(Clone, Copy, raw::Vertex, raw::Pod, raw::Zeroable)]
struct PublicRawVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, raw::Pod, raw::Zeroable)]
struct PublicRawUniform {
    tint: [f32; 4],
}

fn assert_vertex<T: raw::Vertex + raw::Pod + raw::Zeroable>() {}
fn assert_uniform<T: raw::Pod + raw::Zeroable>() {}

#[test]
fn raw_derive_traits_are_usable_from_vmnl() -> VMNLResult<()> {
    assert_vertex::<PublicRawVertex>();
    assert_uniform::<PublicRawUniform>();
    Ok(())
}

#[test]
fn raw_uniform_write_is_public_from_vmnl() -> VMNLResult<()> {
    let _write: fn(&mut raw::Uniform<PublicRawUniform>, PublicRawUniform) -> VMNLResult<()> =
        raw::Uniform::<PublicRawUniform>::write;

    Ok(())
}
