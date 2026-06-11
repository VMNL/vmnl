////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public RGBA color type.
////////////////////////////////////////////////////////////////////////////////
use bytemuck::{Pod, Zeroable};
use std::cmp::Ordering;
use std::ops::{AddAssign, Mul, Sub, SubAssign};

/// RGBA color represented as 8-bit components.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable, PartialEq)]
pub struct Rgba {
    /// Red component of the color, in the range `[0, 255]`.
    pub r: u8,
    /// Green component of the color, in the range `[0, 255]`.
    pub g: u8,
    /// Blue component of the color, in the range `[0, 255]`.
    pub b: u8,
    /// Alpha component of the color, in the range `[0, 255]`, where `0` is fully transparent and `255` is fully opaque.
    pub a: u8,
}

impl Rgba {
    /// Create a new `Rgba` color from individual components.
    ///
    /// # Arguments
    /// - `r`: Red component of the color, in the range `[0, 255]`.
    /// - `g`: Green component of the color, in the range `[0, 255]`.
    /// - `b`: Blue component of the color, in the range `[0, 255]`.
    /// - `a`: Alpha component of the color, in the range `[0, 255]`, where `0` is fully transparent and `255` is fully opaque.
    ///
    /// # Returns
    /// A new `Rgba` instance representing the specified color.
    /// # Example
    /// ```rust
    /// use vmnl_graphics::Rgba;
    /// let color = Rgba::new(255, 0, 0, 255); /// Creates a fully opaque red color.
    /// ```
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Normalize the RGBA color components to the range `[0.0, 1.0]` for use in shader uniforms.
    ///
    /// # Returns
    /// An array of four `f32` values representing the normalized RGBA color,
    /// where each component is in the range `[0.0, 1.0]`.
    pub(crate) fn normalized(self) -> [f32; 4] {
        [
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
            f32::from(self.a) / 255.0,
        ]
    }

    fn scaled_u8(component: u8, scalar: u8) -> u8 {
        u8::try_from(u16::from(component) * u16::from(scalar) / 255).unwrap_or(u8::MAX)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn scaled_f32(component: u8, scalar: f32) -> u8 {
        (f32::from(component) * scalar).clamp(0.0, 255.0) as u8
    }
}

impl Eq for Rgba {}

impl PartialOrd for Rgba {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rgba {
    fn cmp(&self, other: &Self) -> Ordering {
        self.r
            .cmp(&other.r)
            .then_with(|| self.g.cmp(&other.g))
            .then_with(|| self.b.cmp(&other.b))
            .then_with(|| self.a.cmp(&other.a))
    }
}

impl Sub for Rgba {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            r: self.r.saturating_sub(other.r),
            g: self.g.saturating_sub(other.g),
            b: self.b.saturating_sub(other.b),
            a: self.a.saturating_sub(other.a),
        }
    }
}

impl SubAssign for Rgba {
    fn sub_assign(&mut self, other: Self) {
        self.r = self.r.saturating_sub(other.r);
        self.g = self.g.saturating_sub(other.g);
        self.b = self.b.saturating_sub(other.b);
        self.a = self.a.saturating_sub(other.a);
    }
}

impl AddAssign for Rgba {
    fn add_assign(&mut self, other: Self) {
        self.r = self.r.saturating_add(other.r);
        self.g = self.g.saturating_add(other.g);
        self.b = self.b.saturating_add(other.b);
        self.a = self.a.saturating_add(other.a);
    }
}

impl Mul<u8> for Rgba {
    type Output = Self;

    fn mul(self, scalar: u8) -> Self {
        Self {
            r: Self::scaled_u8(self.r, scalar),
            g: Self::scaled_u8(self.g, scalar),
            b: Self::scaled_u8(self.b, scalar),
            a: Self::scaled_u8(self.a, scalar),
        }
    }
}

impl Mul<f32> for Rgba {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            r: Self::scaled_f32(self.r, scalar),
            g: Self::scaled_f32(self.g, scalar),
            b: Self::scaled_f32(self.b, scalar),
            a: Self::scaled_f32(self.a, scalar),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_color_eq(actual: [f32; 4], expected: [f32; 4]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn rgba_new_stores_components() {
        assert_eq!(
            Rgba::new(1, 2, 3, 4),
            Rgba {
                r: 1,
                g: 2,
                b: 3,
                a: 4
            }
        );
    }

    #[test]
    fn rgba_normalized_maps_components_to_unit_range() {
        assert_color_eq(
            Rgba::new(255, 128, 0, 64).normalized(),
            [1.0, 128.0 / 255.0, 0.0, 64.0 / 255.0],
        );
    }
}
