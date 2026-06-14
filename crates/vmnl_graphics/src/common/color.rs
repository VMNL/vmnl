////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public RGBA color type.
////////////////////////////////////////////////////////////////////////////////
use bytemuck::{Pod, Zeroable};
use std::cmp::Ordering;
use std::ops::{AddAssign, Mul, Sub, SubAssign};

macro_rules! rgba_color_constants {
    ($($name:ident = ($r:literal, $g:literal, $b:literal);)*) => {
        $(
            #[doc = concat!("Named opaque color constant `Rgba::", stringify!($name), "`.")]
            pub const $name: Self = Self::rgb($r, $g, $b);
        )*
    };
}

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
    rgba_color_constants! {
        BLACK = (0, 0, 0);
        WHITE = (255, 255, 255);
        RED = (255, 0, 0);
        GREEN = (0, 255, 0);
        BLUE = (0, 0, 255);
        CYAN = (0, 255, 255);
        MAGENTA = (255, 0, 255);
        YELLOW = (255, 255, 0);
        ALICE_BLUE = (240, 248, 255);
        ANTIQUE_WHITE = (250, 235, 215);
        AQUA = (0, 255, 255);
        AQUAMARINE = (127, 255, 212);
        AZURE = (240, 255, 255);
        BEIGE = (245, 245, 220);
        BISQUE = (255, 228, 196);
        BLANCHED_ALMOND = (255, 235, 205);
        BLUE_VIOLET = (138, 43, 226);
        BROWN = (165, 42, 42);
        BURLY_WOOD = (222, 184, 135);
        CADET_BLUE = (95, 158, 160);
        CHARTREUSE = (127, 255, 0);
        CHOCOLATE = (210, 105, 30);
        CORAL = (255, 127, 80);
        CORNFLOWER_BLUE = (100, 149, 237);
        CORNSILK = (255, 248, 220);
        CRIMSON = (220, 20, 60);
        DARK_BLUE = (0, 0, 139);
        DARK_CYAN = (0, 139, 139);
        DARK_GOLDENROD = (184, 134, 11);
        DARK_GRAY = (169, 169, 169);
        DARK_GREEN = (0, 100, 0);
        DARK_GREY = (169, 169, 169);
        DARK_KHAKI = (189, 183, 107);
        DARK_MAGENTA = (139, 0, 139);
        DARK_OLIVE_GREEN = (85, 107, 47);
        DARK_ORANGE = (255, 140, 0);
        DARK_ORCHID = (153, 50, 204);
        DARK_RED = (139, 0, 0);
        DARK_SALMON = (233, 150, 122);
        DARK_SEA_GREEN = (143, 188, 143);
        DARK_SLATE_BLUE = (72, 61, 139);
        DARK_SLATE_GRAY = (47, 79, 79);
        DARK_SLATE_GREY = (47, 79, 79);
        DARK_TURQUOISE = (0, 206, 209);
        DARK_VIOLET = (148, 0, 211);
        DEEP_PINK = (255, 20, 147);
        DEEP_SKY_BLUE = (0, 191, 255);
        DIM_GRAY = (105, 105, 105);
        DIM_GREY = (105, 105, 105);
        DODGER_BLUE = (30, 144, 255);
        FIREBRICK = (178, 34, 34);
        FLORAL_WHITE = (255, 250, 240);
        FOREST_GREEN = (34, 139, 34);
        FUCHSIA = (255, 0, 255);
        GAINSBORO = (220, 220, 220);
        GHOST_WHITE = (248, 248, 255);
        GOLD = (255, 215, 0);
        GOLDENROD = (218, 165, 32);
        GRAY = (128, 128, 128);
        GREY = (128, 128, 128);
        GREEN_YELLOW = (173, 255, 47);
        HONEYDEW = (240, 255, 240);
        HOT_PINK = (255, 105, 180);
        INDIAN_RED = (205, 92, 92);
        INDIGO = (75, 0, 130);
        IVORY = (255, 255, 240);
        KHAKI = (240, 230, 140);
        LAVENDER = (230, 230, 250);
        LAVENDER_BLUSH = (255, 240, 245);
        LAWN_GREEN = (124, 252, 0);
        LEMON_CHIFFON = (255, 250, 205);
        LIGHT_BLUE = (173, 216, 230);
        LIGHT_CORAL = (240, 128, 128);
        LIGHT_CYAN = (224, 255, 255);
        LIGHT_GOLDENROD_YELLOW = (250, 250, 210);
        LIGHT_GRAY = (211, 211, 211);
        LIGHT_GREEN = (144, 238, 144);
        LIGHT_GREY = (211, 211, 211);
        LIGHT_PINK = (255, 182, 193);
        LIGHT_SALMON = (255, 160, 122);
        LIGHT_SEA_GREEN = (32, 178, 170);
        LIGHT_SKY_BLUE = (135, 206, 250);
        LIGHT_SLATE_GRAY = (119, 136, 153);
        LIGHT_SLATE_GREY = (119, 136, 153);
        LIGHT_STEEL_BLUE = (176, 196, 222);
        LIGHT_YELLOW = (255, 255, 224);
        LIME = (0, 255, 0);
        LIME_GREEN = (50, 205, 50);
        LINEN = (250, 240, 230);
        MAROON = (128, 0, 0);
        MEDIUM_AQUAMARINE = (102, 205, 170);
        MEDIUM_BLUE = (0, 0, 205);
        MEDIUM_ORCHID = (186, 85, 211);
        MEDIUM_PURPLE = (147, 112, 219);
        MEDIUM_SEA_GREEN = (60, 179, 113);
        MEDIUM_SLATE_BLUE = (123, 104, 238);
        MEDIUM_SPRING_GREEN = (0, 250, 154);
        MEDIUM_TURQUOISE = (72, 209, 204);
        MEDIUM_VIOLET_RED = (199, 21, 133);
        MIDNIGHT_BLUE = (25, 25, 112);
        MINT_CREAM = (245, 255, 250);
        MISTY_ROSE = (255, 228, 225);
        MOCCASIN = (255, 228, 181);
        NAVAJO_WHITE = (255, 222, 173);
        NAVY = (0, 0, 128);
        OLD_LACE = (253, 245, 230);
        OLIVE = (128, 128, 0);
        OLIVE_DRAB = (107, 142, 35);
        ORANGE = (255, 165, 0);
        ORANGE_RED = (255, 69, 0);
        ORCHID = (218, 112, 214);
        PALE_GOLDENROD = (238, 232, 170);
        PALE_GREEN = (152, 251, 152);
        PALE_TURQUOISE = (175, 238, 238);
        PALE_VIOLET_RED = (219, 112, 147);
        PAPAYA_WHIP = (255, 239, 213);
        PEACH_PUFF = (255, 218, 185);
        PERU = (205, 133, 63);
        PINK = (255, 192, 203);
        PLUM = (221, 160, 221);
        POWDER_BLUE = (176, 224, 230);
        PURPLE = (128, 0, 128);
        REBECCA_PURPLE = (102, 51, 153);
        ROSY_BROWN = (188, 143, 143);
        ROYAL_BLUE = (65, 105, 225);
        SADDLE_BROWN = (139, 69, 19);
        SALMON = (250, 128, 114);
        SANDY_BROWN = (244, 164, 96);
        SEA_GREEN = (46, 139, 87);
        SEA_SHELL = (255, 245, 238);
        SIENNA = (160, 82, 45);
        SILVER = (192, 192, 192);
        SKY_BLUE = (135, 206, 235);
        SLATE_BLUE = (106, 90, 205);
        SLATE_GRAY = (112, 128, 144);
        SLATE_GREY = (112, 128, 144);
        SNOW = (255, 250, 250);
        SPRING_GREEN = (0, 255, 127);
        STEEL_BLUE = (70, 130, 180);
        TAN = (210, 180, 140);
        TEAL = (0, 128, 128);
        THISTLE = (216, 191, 216);
        TOMATO = (255, 99, 71);
        TURQUOISE = (64, 224, 208);
        VIOLET = (238, 130, 238);
        WEB_GREEN = (0, 128, 0);
        WHEAT = (245, 222, 179);
        WHITE_SMOKE = (245, 245, 245);
        YELLOW_GREEN = (154, 205, 50);
    }

    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    /// Create an opaque `Rgba` color from red, green, and blue components.
    ///
    /// Alpha is set to `255`.
    ///
    /// # Arguments
    /// - `r`: Red component of the color, in the range `[0, 255]`.
    /// - `g`: Green component of the color, in the range `[0, 255]`.
    /// - `b`: Blue component of the color, in the range `[0, 255]`.
    ///
    /// # Returns
    /// A new opaque `Rgba` instance.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::common::Rgba;
    ///
    /// let color = Rgba::rgb(20, 24, 32);
    /// assert_eq!(color.a, 255);
    /// ```
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create an `Rgba` color from red, green, blue, and alpha components.
    ///
    /// # Arguments
    /// - `r`: Red component of the color, in the range `[0, 255]`.
    /// - `g`: Green component of the color, in the range `[0, 255]`.
    /// - `b`: Blue component of the color, in the range `[0, 255]`.
    /// - `a`: Alpha component of the color, in the range `[0, 255]`, where `0` is fully transparent and `255` is fully opaque.
    ///
    /// # Returns
    /// A new `Rgba` instance.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::common::Rgba;
    ///
    /// let color = Rgba::rgba(20, 24, 32, 180);
    /// ```
    #[must_use]
    #[allow(clippy::self_named_constructors)]
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new `Rgba` color from individual components.
    ///
    /// Alias for [`Rgba::rgba`]. Prefer [`Rgba::rgb`] for opaque colors.
    ///
    /// # Arguments
    /// - `r`: Red component of the color, in the range `[0, 255]`.
    /// - `g`: Green component of the color, in the range `[0, 255]`.
    /// - `b`: Blue component of the color, in the range `[0, 255]`.
    /// - `a`: Alpha component of the color, in the range `[0, 255]`, where `0` is fully transparent and `255` is fully opaque.
    ///
    /// # Returns
    /// A new `Rgba` instance.
    ///
    /// # Example
    /// ```rust
    /// use vmnl_graphics::common::Rgba;
    ///
    /// let color = Rgba::new(255, 0, 0, 255);
    /// ```
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(r, g, b, a)
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

impl From<[u8; 3]> for Rgba {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<[u8; 4]> for Rgba {
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Self::rgba(r, g, b, a)
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
    fn rgba_rgb_defaults_alpha_to_opaque() {
        assert_eq!(
            Rgba::rgb(1, 2, 3),
            Rgba {
                r: 1,
                g: 2,
                b: 3,
                a: 255
            }
        );
    }

    #[test]
    fn rgba_constructor_matches_new() {
        assert_eq!(Rgba::rgba(1, 2, 3, 4), Rgba::new(1, 2, 3, 4));
    }

    #[test]
    fn rgba_from_rgb_array_defaults_alpha_to_opaque() {
        let color: Rgba = [1, 2, 3].into();
        assert_eq!(color, Rgba::rgb(1, 2, 3));
    }

    #[test]
    fn rgba_from_rgba_array_stores_components() {
        let color: Rgba = [1, 2, 3, 4].into();
        assert_eq!(color, Rgba::rgba(1, 2, 3, 4));
    }

    #[test]
    fn rgba_named_constants_store_expected_components() {
        assert_eq!(Rgba::RED, Rgba::rgb(255, 0, 0));
        assert_eq!(Rgba::GREEN, Rgba::rgb(0, 255, 0));
        assert_eq!(Rgba::BLUE, Rgba::rgb(0, 0, 255));
        assert_eq!(Rgba::WEB_GREEN, Rgba::rgb(0, 128, 0));
        assert_eq!(Rgba::REBECCA_PURPLE, Rgba::rgb(102, 51, 153));
        assert_eq!(Rgba::TRANSPARENT, Rgba::rgba(0, 0, 0, 0));
    }

    #[test]
    fn rgba_normalized_maps_components_to_unit_range() {
        assert_color_eq(
            Rgba::new(255, 128, 0, 64).normalized(),
            [1.0, 128.0 / 255.0, 0.0, 64.0 / 255.0],
        );
    }
}
