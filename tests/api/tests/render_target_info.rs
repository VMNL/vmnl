// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Backend-independent color-format inspection through the public facade.

use vmnl::ColorFormat;

#[test]
fn color_formats_distinguish_component_order_and_encoding() {
    let formats = [
        (ColorFormat::B8G8R8A8_SRGB, "B8G8R8A8_SRGB"),
        (ColorFormat::B8G8R8A8_UNORM, "B8G8R8A8_UNORM"),
        (ColorFormat::R8G8B8A8_SRGB, "R8G8B8A8_SRGB"),
        (ColorFormat::R8G8B8A8_UNORM, "R8G8B8A8_UNORM"),
    ];
    for (index, (format, name)) in formats.iter().enumerate() {
        assert_eq!(format.to_string(), *name);
        for (other, _) in &formats[index + 1..] {
            assert_ne!(format, other);
        }
    }
}
