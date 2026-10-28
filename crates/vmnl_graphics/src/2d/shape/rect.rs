// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Rectangle shape utilities for the VMNL graphics module,
//! providing functions to create axis-aligned rectangles defined by position, size, and color.

use super::{
    arc_sample,
    validation::{validate_finite, validate_positive_finite},
    Shape,
    ShapeKind::Rectangle,
    Vector2f, Vertex2D,
};
use crate::{
    common::{BufferMemoryPreference, Rgba},
    Context, VMNLError, VMNLErrorKind, VMNLResult,
};

/// Default line segments used to approximate each non-zero quarter arc,
/// matching one quarter of the 32-segment circle tessellation.
const DEFAULT_CORNER_SEGMENTS: u16 = 8;

/// Predefined local origins for rectangle rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Anchor {
    /// Local origin at the top-left corner of the rectangle.
    #[default]
    TopLeft,
    /// Local origin at the top edge center of the rectangle.
    Top,
    /// Local origin at the top-right corner of the rectangle.
    TopRight,
    /// Local origin at the left edge center of the rectangle.
    Left,
    /// Local origin at the center of the rectangle.
    Center,
    /// Local origin at the right edge center of the rectangle.
    Right,
    /// Local origin at the bottom-left corner of the rectangle.
    BottomLeft,
    /// Local origin at the bottom edge center of the rectangle.
    Bottom,
    /// Local origin at the bottom-right corner of the rectangle.
    BottomRight,
}

impl Anchor {
    fn origin(self, size: Vector2f) -> Vector2f {
        match self {
            Self::TopLeft => Vector2f { x: 0.0, y: 0.0 },
            Self::Top => Vector2f {
                x: size.x / 2.0,
                y: 0.0,
            },
            Self::TopRight => Vector2f { x: size.x, y: 0.0 },
            Self::Left => Vector2f {
                x: 0.0,
                y: size.y / 2.0,
            },
            Self::Center => Vector2f {
                x: size.x / 2.0,
                y: size.y / 2.0,
            },
            Self::Right => Vector2f {
                x: size.x,
                y: size.y / 2.0,
            },
            Self::BottomLeft => Vector2f { x: 0.0, y: size.y },
            Self::Bottom => Vector2f {
                x: size.x / 2.0,
                y: size.y,
            },
            Self::BottomRight => Vector2f {
                x: size.x,
                y: size.y,
            },
        }
    }
}

/// Circular corner radii of a rounded rectangle.
///
/// Field and constructor order is clockwise from the top-left corner:
/// `top_left`, `top_right`, `bottom_right`, `bottom_left`. A radius of `0.0`
/// keeps the corresponding corner sharp. See [`RectBuilder::corner_radii`] for
/// validation and geometry costs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadii {
    /// Radius of the top-left corner. `0.0` keeps the corner sharp.
    pub top_left: f32,
    /// Radius of the top-right corner. `0.0` keeps the corner sharp.
    pub top_right: f32,
    /// Radius of the bottom-right corner. `0.0` keeps the corner sharp.
    pub bottom_right: f32,
    /// Radius of the bottom-left corner. `0.0` keeps the corner sharp.
    pub bottom_left: f32,
}

impl CornerRadii {
    /// Four equal corner radii.
    ///
    /// # Arguments
    /// - `radius`: Radius applied to every corner. `0.0` keeps all corners sharp.
    #[must_use]
    pub const fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Four independent corner radii, ordered clockwise from the top-left corner.
    ///
    /// # Arguments
    /// - `top_left`: Top-left corner radius.
    /// - `top_right`: Top-right corner radius.
    /// - `bottom_right`: Bottom-right corner radius.
    /// - `bottom_left`: Bottom-left corner radius.
    #[must_use]
    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    /// True when every corner is sharp, selecting the four-vertex rectangle fast path.
    fn is_sharp(self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }
}

#[derive(Clone, Copy, Debug)]
enum RectOrigin {
    /// Predefined anchor point for rectangle rotation pivot.
    Anchor(Anchor),
    /// Custom local origin for rectangle rotation pivot, specified as a `Vector2f` relative to the rectangle's top-left corner.
    Custom(Vector2f),
}

/// Local description of one rounded-rectangle corner.
#[derive(Clone, Copy)]
struct RoundedCornerSpec {
    radius: f32,
    /// Arc center relative to the rectangle top-left corner.
    center: Vector2f,
    /// Start angle in radians; the quarter arc sweeps clockwise in the
    /// built-in X-right/Y-down coordinate system.
    start_angle: f32,
    /// Boundary point used when the radius is zero.
    sharp_point: Vector2f,
}

/// Options for configuring rectangle shape properties such as position, size, and color.
#[derive(Clone, Debug)]
struct RectOptions {
    /// Top-left position of the rectangle as a `Vector2f`.
    position: Vector2f,
    /// Size of the rectangle as a `Vector2f`, where `x` is the width and `y` is the height. Both components must be strictly positive.
    size: Vector2f,
    /// RGBA color of the rectangle, using 8-bit components in the range `[0, 255]`.
    color: Rgba,
    /// Rotation of the rectangle in degrees. Rotation is applied around `origin`.
    rotation: f32,
    /// Local origin used as the rectangle rotation pivot. Defaults to `Anchor::TopLeft`.
    origin: RectOrigin,
    /// Circular corner radii, ordered clockwise from the top-left corner. Defaults to sharp corners.
    corner_radii: CornerRadii,
    /// Line segments used to approximate each non-zero quarter arc. Defaults to `8`.
    corner_segments: u16,
    /// Preferred memory placement for the created vertex and index buffers.
    buffer_memory_preference: BufferMemoryPreference,
}

impl Default for RectOptions {
    fn default() -> Self {
        Self {
            position: Vector2f { x: 0.0, y: 0.0 },
            size: Vector2f { x: 0.0, y: 0.0 },
            color: Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            rotation: 0.0,
            origin: RectOrigin::Anchor(Anchor::TopLeft),
            corner_radii: CornerRadii::default(),
            corner_segments: DEFAULT_CORNER_SEGMENTS,
            buffer_memory_preference: BufferMemoryPreference::default(),
        }
    }
}

/// Builder for creating rectangle shapes with configurable properties such as position,
/// size, color, and rounded corners.
///
/// Corners default to sharp (`0.0` radius) with `8` segments per non-zero quarter arc.
/// A rectangle whose four radii are all zero keeps the four-vertex sharp-rectangle
/// geometry; any non-zero radius generates one clockwise boundary of straight edges,
/// quarter arcs, and sharp corner points, triangulated as one triangle fan around the
/// rectangle center. For four non-zero corners and `S` segments per corner the shape has
/// `4 * (S + 1) + 1` vertices and `3 * 4 * (S + 1)` indices. `corner_segments` controls
/// the geometric approximation only; it does not provide anti-aliasing.
pub struct RectBuilder {
    /// Configuration options for the rectangle shape, including position, size, and color.
    options: RectOptions,
}

impl RectBuilder {
    pub(crate) fn new(size: Vector2f) -> Self {
        Self {
            options: RectOptions {
                size,
                ..Default::default()
            },
        }
    }

    /// Set the position of the rectangle.
    ///
    /// # Arguments
    /// - `x`: X-coordinate of the top-left corner of the rectangle.
    /// - `y`: Y-coordinate of the top-left corner of the rectangle.
    ///
    /// # Returns
    /// The updated `RectBuilder` instance with the specified position.
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .position(50.0, 50.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.options.position = Vector2f { x, y };
        self
    }

    /// Set the color of the rectangle.
    ///
    /// # Arguments
    /// - `color`: Color convertible to `Rgba`, for example `Rgba::RED`, `[r, g, b]`, or `[r, g, b, a]`.
    ///
    /// # Returns
    /// The updated `RectBuilder` instance with the specified color.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .color([255, 0, 0])
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn color<C>(mut self, color: C) -> Self
    where
        C: Into<Rgba>,
    {
        self.options.color = color.into();
        self
    }

    /// Set the rotation of the rectangle in degrees.
    ///
    /// # Arguments
    /// - `degrees`: Rotation angle in degrees. Rotation is applied around the configured origin.
    ///
    /// # Returns
    /// The updated `RectBuilder` instance with the specified rotation.
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .rotation(45.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn rotation(mut self, degrees: f32) -> Self {
        self.options.rotation = degrees;
        self
    }

    /// Set the rectangle rotation pivot using a predefined anchor.
    ///
    /// Replaces any previous custom origin.
    ///
    /// # Arguments
    /// - `anchor`: Predefined local origin used as the rotation pivot.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::{Anchor, Shape};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .anchor(Anchor::Center)
    ///     .rotation(45.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.options.origin = RectOrigin::Anchor(anchor);
        self
    }

    /// Set the local origin used as the rectangle rotation pivot.
    ///
    /// # Arguments
    /// - `x`: X coordinate relative to the rectangle top-left corner.
    /// - `y`: Y coordinate relative to the rectangle top-left corner.
    ///
    /// Replaces any previous anchor.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .origin(50.0, 50.0)
    ///     .rotation(45.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn origin(mut self, x: f32, y: f32) -> Self {
        self.options.origin = RectOrigin::Custom(Vector2f { x, y });
        self
    }

    /// Set one uniform circular corner radius on every corner.
    ///
    /// Equivalent to `corner_radii(CornerRadii::all(radius))`. The last call to this
    /// method or [`Self::corner_radii`] replaces the previous corner configuration.
    /// A radius of `0.0` keeps the existing four-vertex sharp-rectangle path.
    ///
    /// # Arguments
    /// - `radius`: Corner radius in pixel-like 2D coordinates. Must be finite and
    ///   greater than or equal to zero at build time.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .corner_radius(16.0)
    ///     .build(&context)?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.options.corner_radii = CornerRadii::all(radius);
        self
    }

    /// Set independent circular corner radii.
    ///
    /// The last call to this method or [`Self::corner_radius`] replaces the previous
    /// corner configuration. Fields are ordered clockwise from the top-left corner.
    /// A radius of `0.0` keeps the corresponding corner sharp.
    ///
    /// # Arguments
    /// - `radii`: Per-corner radii in pixel-like 2D coordinates. Each radius must be
    ///   finite and greater than or equal to zero at build time.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::{CornerRadii, Shape};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .corner_radii(CornerRadii::new(24.0, 8.0, 32.0, 0.0))
    ///     .build(&context)?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn corner_radii(mut self, radii: CornerRadii) -> Self {
        self.options.corner_radii = radii;
        self
    }

    /// Set the number of line segments used for each non-zero quarter arc.
    ///
    /// This is an explicit quality/cost control over the geometric approximation only;
    /// it does not provide anti-aliasing. Defaults to `8`; at least one segment is
    /// required at build time. Zero-radius corners contribute one sharp boundary point
    /// each and ignore this setting. See [`Self::build`] for the resulting vertex and
    /// index counts.
    ///
    /// # Arguments
    /// - `segments`: Segments per non-zero quarter arc, in `1..=65535`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .corner_radius(16.0)
    ///     .corner_segments(8)
    ///     .build(&context)?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn corner_segments(mut self, segments: u16) -> Self {
        self.options.corner_segments = segments;
        self
    }

    /// Set the preferred memory placement for the created vertex and index buffers.
    ///
    /// This is a preference, not a guarantee. Defaults to `BufferMemoryPreference::Device`.
    ///
    /// # Arguments
    /// - `preference`: Preferred GPU buffer memory placement.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::common::BufferMemoryPreference;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .buffer_memory_preference(BufferMemoryPreference::Host)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn buffer_memory_preference(mut self, preference: BufferMemoryPreference) -> Self {
        self.options.buffer_memory_preference = preference;
        self
    }

    /// Build a rectangle shape from the provided options.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    ///
    /// # Returns
    /// A `Shape` instance representing the rectangle, ready for rendering.
    ///
    /// # Errors
    /// Returns an error if the geometry is invalid or GPU buffer creation fails.
    ///
    /// The final builder state is validated before any GPU allocation. Each corner
    /// radius must be finite and greater than or equal to zero, the segment count must
    /// be at least one, and corner regions must not overlap:
    ///
    /// ```text
    /// top_left + top_right       <= width
    /// bottom_left + bottom_right <= width
    /// top_left + bottom_left     <= height
    /// top_right + bottom_right   <= height
    /// ```
    ///
    /// Overlapping, negative, non-finite, or otherwise unrepresentable corner
    /// configurations return [`VMNLErrorKind::InvalidState`] instead of being clamped
    /// or rescaled. Computed arc centers, contour vertices, transformed vertices, and
    /// generated counts must remain finite and representable.
    ///
    /// Building performs CPU tessellation followed by one vertex-buffer and one
    /// index-buffer allocation/upload; rounded geometry uses checked count arithmetic
    /// and fallible CPU reservation and never panics on invalid input. Rotation is
    /// applied around the configured [`Anchor`] or custom origin after local boundary
    /// generation. Drawing reuses the immutable buffers; there is no per-frame
    /// allocation. Building requires a Vulkan context, and drawing additionally
    /// requires a compatible window/render backend. Exact timing and synchronization
    /// costs are unspecified.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::Context;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let rect = Shape::rect(100.0, 100.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, vmnl_context: &Context) -> VMNLResult<Shape> {
        let options: RectOptions = self.options;
        Self::validate_parameters(
            options.position,
            options.size,
            options.rotation,
            options.origin,
        )?;
        Self::validate_corners(options.size, options.corner_radii, options.corner_segments)?;

        let (vertices, indices) = Self::geometry(
            options.position,
            options.size,
            options.color,
            options.rotation,
            options.origin,
            options.corner_radii,
            options.corner_segments,
        )?;

        let mut graphics: Shape = Shape::indexed(vertices, indices)
            .buffer_memory_preference(options.buffer_memory_preference)
            .build(vmnl_context)?;

        graphics.kind = Rectangle;
        if options.corner_radii.is_sharp() {
            log::trace!(
                "creating rectangle: position=({}, {}), size=({}, {}), color=({}, {}, {}, {})",
                options.position.x,
                options.position.y,
                options.size.x,
                options.size.y,
                options.color.r,
                options.color.g,
                options.color.b,
                options.color.a
            );
        } else {
            log::trace!(
                "creating rounded rectangle: position=({}, {}), size=({}, {}), corner_radii=({}, {}, {}, {}), corner_segments={}, color=({}, {}, {}, {})",
                options.position.x,
                options.position.y,
                options.size.x,
                options.size.y,
                options.corner_radii.top_left,
                options.corner_radii.top_right,
                options.corner_radii.bottom_right,
                options.corner_radii.bottom_left,
                options.corner_segments,
                options.color.r,
                options.color.g,
                options.color.b,
                options.color.a
            );
        }
        Ok(graphics)
    }

    /// Validate the corner configuration against the rectangle size.
    ///
    /// Radii are never clamped or rescaled: overlapping corner regions are rejected.
    fn validate_corners(size: Vector2f, radii: CornerRadii, segments: u16) -> VMNLResult<()> {
        validate_finite(
            &[
                radii.top_left,
                radii.top_right,
                radii.bottom_right,
                radii.bottom_left,
            ],
            "rectangle corner radii",
        )?;

        if radii.top_left < 0.0
            || radii.top_right < 0.0
            || radii.bottom_right < 0.0
            || radii.bottom_left < 0.0
        {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle corner radii must be greater than or equal to zero".to_string(),
            )));
        }

        if segments == 0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle corner segments must be strictly positive".to_string(),
            )));
        }

        let top: f32 = radii.top_left + radii.top_right;
        let bottom: f32 = radii.bottom_left + radii.bottom_right;
        let left: f32 = radii.top_left + radii.bottom_left;
        let right: f32 = radii.top_right + radii.bottom_right;
        validate_finite(
            &[top, bottom, left, right],
            "combined rectangle corner radii",
        )?;

        if top > size.x || bottom > size.x || left > size.y || right > size.y {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "rectangle corner radii must not overlap".to_string(),
            )));
        }

        Ok(())
    }

    /// Generate the CPU geometry for the final builder state.
    ///
    /// All-zero radii keep the existing four-vertex sharp-rectangle fast path.
    fn geometry(
        position: Vector2f,
        size: Vector2f,
        color: Rgba,
        rotation: f32,
        origin: RectOrigin,
        radii: CornerRadii,
        segments: u16,
    ) -> VMNLResult<(Vec<Vertex2D>, Vec<u32>)> {
        if radii.is_sharp() {
            let vertices: [Vertex2D; 4] = Self::vertices(position, size, color, rotation, origin);
            let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
            return Ok((vertices.to_vec(), indices.to_vec()));
        }

        Self::rounded_geometry(position, size, color, rotation, origin, radii, segments)
    }

    /// Number of boundary vertices contributed by one corner.
    fn corner_vertex_count(radius: f32, segments: u16) -> VMNLResult<usize> {
        if radius > 0.0 {
            usize::from(segments)
                .checked_add(1)
                .ok_or_else(Self::rounded_count_error)
        } else {
            Ok(1)
        }
    }

    /// Total vertex and index counts for a rounded-rectangle triangle fan.
    fn rounded_counts(boundary_count: usize) -> VMNLResult<(usize, usize)> {
        let vertex_count: usize = boundary_count
            .checked_add(1)
            .ok_or_else(Self::rounded_count_error)?;
        let index_count: usize = boundary_count
            .checked_mul(3)
            .ok_or_else(Self::rounded_count_error)?;

        Ok((vertex_count, index_count))
    }

    fn rounded_count_error() -> VMNLError {
        VMNLError::new(VMNLErrorKind::InvalidState(
            "rounded rectangle geometry count overflow".to_string(),
        ))
    }

    fn geometry_reservation_error() -> VMNLError {
        VMNLError::new(VMNLErrorKind::InvalidState(
            "rounded rectangle geometry memory reservation failed".to_string(),
        ))
    }

    /// Corner specs ordered clockwise from the top-left corner.
    fn corner_specs(size: Vector2f, radii: CornerRadii) -> [RoundedCornerSpec; 4] {
        [
            RoundedCornerSpec {
                radius: radii.top_left,
                center: Vector2f {
                    x: radii.top_left,
                    y: radii.top_left,
                },
                start_angle: std::f32::consts::PI,
                sharp_point: Vector2f { x: 0.0, y: 0.0 },
            },
            RoundedCornerSpec {
                radius: radii.top_right,
                center: Vector2f {
                    x: size.x - radii.top_right,
                    y: radii.top_right,
                },
                start_angle: 3.0 * std::f32::consts::FRAC_PI_2,
                sharp_point: Vector2f { x: size.x, y: 0.0 },
            },
            RoundedCornerSpec {
                radius: radii.bottom_right,
                center: Vector2f {
                    x: size.x - radii.bottom_right,
                    y: size.y - radii.bottom_right,
                },
                start_angle: 0.0,
                sharp_point: Vector2f {
                    x: size.x,
                    y: size.y,
                },
            },
            RoundedCornerSpec {
                radius: radii.bottom_left,
                center: Vector2f {
                    x: radii.bottom_left,
                    y: size.y - radii.bottom_left,
                },
                start_angle: std::f32::consts::FRAC_PI_2,
                sharp_point: Vector2f { x: 0.0, y: size.y },
            },
        ]
    }

    /// Generate the rounded-rectangle boundary as one clockwise contour, then
    /// triangulate it as a triangle fan around the rectangle center.
    ///
    /// Local boundary points are generated before the existing rotation around
    /// the configured anchor or custom origin is applied. Every transformed
    /// vertex must remain finite; otherwise the configuration is rejected before
    /// any GPU allocation.
    fn rounded_geometry(
        position: Vector2f,
        size: Vector2f,
        color: Rgba,
        rotation: f32,
        origin: RectOrigin,
        radii: CornerRadii,
        segments: u16,
    ) -> VMNLResult<(Vec<Vertex2D>, Vec<u32>)> {
        let corner_specs: [RoundedCornerSpec; 4] = Self::corner_specs(size, radii);

        let mut boundary_count: usize = 0;
        for corner in corner_specs {
            boundary_count = boundary_count
                .checked_add(Self::corner_vertex_count(corner.radius, segments)?)
                .ok_or_else(Self::rounded_count_error)?;
        }
        let (vertex_count, index_count) = Self::rounded_counts(boundary_count)?;

        let mut boundary: Vec<Vector2f> = Vec::new();
        boundary
            .try_reserve(boundary_count)
            .map_err(|_| Self::geometry_reservation_error())?;
        let mut vertices: Vec<Vertex2D> = Vec::new();
        vertices
            .try_reserve(vertex_count)
            .map_err(|_| Self::geometry_reservation_error())?;
        let mut indices: Vec<u32> = Vec::new();
        indices
            .try_reserve(index_count)
            .map_err(|_| Self::geometry_reservation_error())?;

        for corner in corner_specs {
            // Boundary points are generated in local coordinates, then offset to
            // absolute coordinates before the configured rotation is applied.
            let corner_center: Vector2f = Vector2f {
                x: position.x + corner.center.x,
                y: position.y + corner.center.y,
            };
            if corner.radius > 0.0 {
                let arc_radii: Vector2f = Vector2f {
                    x: corner.radius,
                    y: corner.radius,
                };
                for segment in 0..=segments {
                    let sample: super::ArcSample = arc_sample(
                        corner_center,
                        arc_radii,
                        corner.start_angle,
                        std::f32::consts::FRAC_PI_2,
                        segment,
                        segments,
                    );
                    boundary.push(sample.position);
                }
            } else {
                boundary.push(Vector2f {
                    x: position.x + corner.sharp_point.x,
                    y: position.y + corner.sharp_point.y,
                });
            }
        }

        let local_origin: Vector2f = Self::resolve_origin(origin, size);
        let rotation_origin: Vector2f = Vector2f {
            x: position.x + local_origin.x,
            y: position.y + local_origin.y,
        };
        let rotating: bool = rotation.rem_euclid(360.0) != 0.0;
        let transform = |point: Vector2f| -> Vector2f {
            if rotating {
                Self::rotate(point, rotation_origin, rotation)
            } else {
                point
            }
        };

        let fan_center: Vector2f = Vector2f {
            x: position.x + size.x / 2.0,
            y: position.y + size.y / 2.0,
        };
        vertices.push(Vertex2D {
            position: transform(fan_center),
            color,
        });
        for point in boundary {
            vertices.push(Vertex2D {
                position: transform(point),
                color,
            });
        }

        let boundary_index_count: u32 =
            u32::try_from(boundary_count).map_err(|_| Self::rounded_count_error())?;
        for boundary_index in 0..boundary_index_count {
            let current: u32 = boundary_index + 1;
            let next: u32 = if boundary_index + 1 == boundary_index_count {
                1
            } else {
                boundary_index + 2
            };
            indices.extend_from_slice(&[0, current, next]);
        }

        for vertex in &vertices {
            validate_finite(
                &[vertex.position.x, vertex.position.y],
                "rounded rectangle vertices",
            )?;
        }

        Ok((vertices, indices))
    }

    fn validate_parameters(
        position: Vector2f,
        size: Vector2f,
        rotation: f32,
        origin: RectOrigin,
    ) -> VMNLResult<()> {
        validate_positive_finite(&[size.x, size.y], "rectangle size")?;
        validate_finite(&[position.x, position.y], "rectangle position")?;

        let bounds: Vector2f = Vector2f {
            x: position.x + size.x,
            y: position.y + size.y,
        };
        validate_finite(&[bounds.x, bounds.y], "rectangle bounds")?;
        validate_finite(&[rotation], "rectangle rotation")?;

        if let RectOrigin::Custom(origin) = origin {
            validate_finite(&[origin.x, origin.y], "rectangle origin")?;
        }

        Ok(())
    }

    fn resolve_origin(origin: RectOrigin, size: Vector2f) -> Vector2f {
        match origin {
            RectOrigin::Anchor(anchor) => anchor.origin(size),
            RectOrigin::Custom(origin) => origin,
        }
    }

    fn rotate(point: Vector2f, origin: Vector2f, degrees: f32) -> Vector2f {
        let radians: f32 = degrees.rem_euclid(360.0).to_radians();
        let sin: f32 = radians.sin();
        let cos: f32 = radians.cos();
        let x: f32 = point.x - origin.x;
        let y: f32 = point.y - origin.y;

        Vector2f {
            x: origin.x + x * cos - y * sin,
            y: origin.y + x * sin + y * cos,
        }
    }

    fn vertices(
        position: Vector2f,
        size: Vector2f,
        color: Rgba,
        rotation: f32,
        origin: RectOrigin,
    ) -> [Vertex2D; 4] {
        if rotation.rem_euclid(360.0) == 0.0 {
            return Self::axis_aligned_vertices(position, size, color);
        }

        let x0: f32 = position.x;
        let y0: f32 = position.y;
        let x1: f32 = x0 + size.x;
        let y1: f32 = y0 + size.y;
        let local_origin: Vector2f = Self::resolve_origin(origin, size);
        let rotation_origin: Vector2f = Vector2f {
            x: position.x + local_origin.x,
            y: position.y + local_origin.y,
        };

        let corners: [Vector2f; 4] = [
            Vector2f { x: x0, y: y0 },
            Vector2f { x: x1, y: y0 },
            Vector2f { x: x1, y: y1 },
            Vector2f { x: x0, y: y1 },
        ];

        corners.map(|corner| Vertex2D {
            position: Self::rotate(corner, rotation_origin, rotation),
            color,
        })
    }

    fn axis_aligned_vertices(position: Vector2f, size: Vector2f, color: Rgba) -> [Vertex2D; 4] {
        let x0: f32 = position.x;
        let y0: f32 = position.y;
        let x1: f32 = x0 + size.x;
        let y1: f32 = y0 + size.y;

        [
            Vertex2D {
                position: Vector2f { x: x0, y: y0 },
                color,
            },
            Vertex2D {
                position: Vector2f { x: x1, y: y0 },
                color,
            },
            Vertex2D {
                position: Vector2f { x: x1, y: y1 },
                color,
            },
            Vertex2D {
                position: Vector2f { x: x0, y: y1 },
                color,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VMNLErrorKind;

    fn assert_vector_eq(actual: Vector2f, expected: Vector2f) {
        const EPSILON: f32 = 0.00001;

        assert!((actual.x - expected.x).abs() < EPSILON);
        assert!((actual.y - expected.y).abs() < EPSILON);
    }

    fn expect_geometry(result: VMNLResult<(Vec<Vertex2D>, Vec<u32>)>) -> (Vec<Vertex2D>, Vec<u32>) {
        assert!(
            result.is_ok(),
            "valid rectangle configuration must generate geometry"
        );
        result.unwrap_or_default()
    }

    fn assert_invalid_state(result: VMNLResult<()>, expected: &str) {
        assert!(matches!(
            result,
            Err(err) if matches!(err.kind(), VMNLErrorKind::InvalidState(message) if message == expected)
        ));
    }

    #[test]
    fn buffer_memory_preference_defaults_to_device() {
        let builder: RectBuilder = RectBuilder::new(Vector2f { x: 1.0, y: 1.0 });

        assert_eq!(
            builder.options.buffer_memory_preference,
            BufferMemoryPreference::Device
        );
    }

    #[test]
    fn buffer_memory_preference_can_be_overridden() {
        let builder: RectBuilder = RectBuilder::new(Vector2f { x: 1.0, y: 1.0 })
            .buffer_memory_preference(BufferMemoryPreference::Host);

        assert_eq!(
            builder.options.buffer_memory_preference,
            BufferMemoryPreference::Host
        );
    }

    #[test]
    fn validate_parameters_accepts_positive_finite_rect() {
        assert!(RectBuilder::validate_parameters(
            Vector2f { x: 1.0, y: 2.0 },
            Vector2f { x: 3.0, y: 4.0 },
            0.0,
            RectOrigin::Anchor(Anchor::TopLeft),
        )
        .is_ok());
    }

    #[test]
    fn validate_parameters_rejects_invalid_size() {
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: -1.0, y: 1.0 },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle size must be strictly positive",
        );
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 0.0 },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle size must be strictly positive",
        );
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: f32::INFINITY,
                    y: 1.0,
                },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle size must be finite",
        );
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f {
                    x: f32::NAN,
                    y: 1.0,
                },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle size must not be NaN",
        );
    }

    #[test]
    fn validate_parameters_rejects_invalid_position_and_bounds() {
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f {
                    x: f32::INFINITY,
                    y: 0.0,
                },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle position must be finite",
        );
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f {
                    x: f32::NAN,
                    y: 0.0,
                },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle position must not be NaN",
        );
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f {
                    x: f32::MAX,
                    y: 0.0,
                },
                Vector2f {
                    x: f32::MAX,
                    y: 1.0,
                },
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle bounds must be finite",
        );
    }

    #[test]
    fn validate_parameters_rejects_invalid_rotation() {
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                f32::INFINITY,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle rotation must be finite",
        );
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                f32::NAN,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            "rectangle rotation must not be NaN",
        );
    }

    #[test]
    fn validate_parameters_rejects_invalid_origin() {
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
                RectOrigin::Custom(Vector2f {
                    x: f32::INFINITY,
                    y: 0.0,
                }),
            ),
            "rectangle origin must be finite",
        );
        assert_invalid_state(
            RectBuilder::validate_parameters(
                Vector2f { x: 0.0, y: 0.0 },
                Vector2f { x: 1.0, y: 1.0 },
                0.0,
                RectOrigin::Custom(Vector2f {
                    x: 0.0,
                    y: f32::NAN,
                }),
            ),
            "rectangle origin must not be NaN",
        );
    }

    #[test]
    fn vertices_returns_expected_rectangle_corners() {
        let color: Rgba = Rgba::new(1, 2, 3, 4);

        assert_eq!(
            RectBuilder::vertices(
                Vector2f { x: 10.0, y: 20.0 },
                Vector2f { x: 30.0, y: 40.0 },
                color,
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
            ),
            [
                Vertex2D {
                    position: Vector2f { x: 10.0, y: 20.0 },
                    color,
                },
                Vertex2D {
                    position: Vector2f { x: 40.0, y: 20.0 },
                    color,
                },
                Vertex2D {
                    position: Vector2f { x: 40.0, y: 60.0 },
                    color,
                },
                Vertex2D {
                    position: Vector2f { x: 10.0, y: 60.0 },
                    color,
                },
            ]
        );
    }

    #[test]
    fn vertices_applies_rotation_around_center() {
        let color: Rgba = Rgba::new(1, 2, 3, 4);
        let vertices: [Vertex2D; 4] = RectBuilder::vertices(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 2.0, y: 4.0 },
            color,
            90.0,
            RectOrigin::Anchor(Anchor::Center),
        );

        assert_vector_eq(vertices[0].position, Vector2f { x: 3.0, y: 1.0 });
        assert_vector_eq(vertices[1].position, Vector2f { x: 3.0, y: 3.0 });
        assert_vector_eq(vertices[2].position, Vector2f { x: -1.0, y: 3.0 });
        assert_vector_eq(vertices[3].position, Vector2f { x: -1.0, y: 1.0 });
    }

    #[test]
    fn vertices_applies_rotation_around_custom_origin() {
        let color: Rgba = Rgba::new(1, 2, 3, 4);
        let vertices: [Vertex2D; 4] = RectBuilder::vertices(
            Vector2f { x: 10.0, y: 20.0 },
            Vector2f { x: 2.0, y: 4.0 },
            color,
            90.0,
            RectOrigin::Custom(Vector2f { x: 0.0, y: 0.0 }),
        );

        assert_vector_eq(vertices[0].position, Vector2f { x: 10.0, y: 20.0 });
        assert_vector_eq(vertices[1].position, Vector2f { x: 10.0, y: 22.0 });
        assert_vector_eq(vertices[2].position, Vector2f { x: 6.0, y: 22.0 });
        assert_vector_eq(vertices[3].position, Vector2f { x: 6.0, y: 20.0 });
    }

    // ---------------------------------------------------------
    // Rounded corners: configuration
    // ---------------------------------------------------------

    #[test]
    fn corner_defaults_are_sharp_with_eight_segments() {
        let builder: RectBuilder = RectBuilder::new(Vector2f { x: 10.0, y: 10.0 });

        assert_eq!(builder.options.corner_radii, CornerRadii::default());
        assert_eq!(builder.options.corner_radii, CornerRadii::all(0.0));
        assert!(builder.options.corner_radii.is_sharp());
        assert_eq!(builder.options.corner_segments, DEFAULT_CORNER_SEGMENTS);
        assert_eq!(DEFAULT_CORNER_SEGMENTS, 8);
    }

    #[test]
    fn corner_radii_all_and_new_map_fields_clockwise() {
        let all: CornerRadii = CornerRadii::all(7.0);

        assert_eq!(all, CornerRadii::new(7.0, 7.0, 7.0, 7.0));
        assert_eq!(all.top_left.to_bits(), 7.0_f32.to_bits());
        assert_eq!(all.top_right.to_bits(), 7.0_f32.to_bits());
        assert_eq!(all.bottom_right.to_bits(), 7.0_f32.to_bits());
        assert_eq!(all.bottom_left.to_bits(), 7.0_f32.to_bits());

        let clockwise: CornerRadii = CornerRadii::new(1.0, 2.0, 3.0, 4.0);

        assert_eq!(clockwise.top_left.to_bits(), 1.0_f32.to_bits());
        assert_eq!(clockwise.top_right.to_bits(), 2.0_f32.to_bits());
        assert_eq!(clockwise.bottom_right.to_bits(), 3.0_f32.to_bits());
        assert_eq!(clockwise.bottom_left.to_bits(), 4.0_f32.to_bits());
    }

    #[test]
    fn corner_radius_and_corner_radii_replace_each_other() {
        let builder: RectBuilder =
            RectBuilder::new(Vector2f { x: 10.0, y: 10.0 }).corner_radius(5.0);

        assert_eq!(builder.options.corner_radii, CornerRadii::all(5.0));

        let builder: RectBuilder = builder.corner_radii(CornerRadii::new(1.0, 2.0, 3.0, 4.0));

        assert_eq!(
            builder.options.corner_radii,
            CornerRadii::new(1.0, 2.0, 3.0, 4.0)
        );

        let builder: RectBuilder = builder.corner_radius(9.0);

        assert_eq!(builder.options.corner_radii, CornerRadii::all(9.0));
    }

    #[test]
    fn corner_segments_is_stored() {
        let builder: RectBuilder =
            RectBuilder::new(Vector2f { x: 10.0, y: 10.0 }).corner_segments(3);

        assert_eq!(builder.options.corner_segments, 3);
    }

    #[test]
    fn zero_corner_radii_keep_the_sharp_fast_path() {
        let color: Rgba = Rgba::new(1, 2, 3, 4);
        let (vertices, indices): (Vec<Vertex2D>, Vec<u32>) =
            expect_geometry(RectBuilder::geometry(
                Vector2f { x: 10.0, y: 20.0 },
                Vector2f { x: 30.0, y: 40.0 },
                color,
                0.0,
                RectOrigin::Anchor(Anchor::TopLeft),
                CornerRadii::all(0.0),
                DEFAULT_CORNER_SEGMENTS,
            ));

        let sharp_path: [Vertex2D; 4] = RectBuilder::vertices(
            Vector2f { x: 10.0, y: 20.0 },
            Vector2f { x: 30.0, y: 40.0 },
            color,
            0.0,
            RectOrigin::Anchor(Anchor::TopLeft),
        );

        assert_eq!(vertices.len(), 4);
        assert_eq!(indices, vec![0, 1, 2, 2, 3, 0]);
        assert_eq!(vertices, sharp_path.to_vec());
    }

    // ---------------------------------------------------------
    // Rounded corners: validation
    // ---------------------------------------------------------

    #[test]
    fn validate_corners_accepts_valid_configuration() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };

        assert!(RectBuilder::validate_corners(size, CornerRadii::default(), 8).is_ok());
        assert!(RectBuilder::validate_corners(size, CornerRadii::all(25.0), 8).is_ok());
    }

    #[test]
    fn validate_corners_rejects_invalid_radii() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };

        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::all(-1.0), 8),
            "rectangle corner radii must be greater than or equal to zero",
        );
        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::new(0.0, f32::NAN, 0.0, 0.0), 8),
            "rectangle corner radii must not be NaN",
        );
        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::new(f32::INFINITY, 0.0, 0.0, 0.0), 8),
            "rectangle corner radii must be finite",
        );
    }

    #[test]
    fn validate_corners_rejects_zero_segments() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };

        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::all(10.0), 0),
            "rectangle corner segments must be strictly positive",
        );
    }

    #[test]
    fn validate_corners_rejects_overlapping_radii_without_clamping() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };

        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::new(60.0, 60.0, 0.0, 0.0), 8),
            "rectangle corner radii must not overlap",
        );
        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::new(0.0, 0.0, 60.0, 60.0), 8),
            "rectangle corner radii must not overlap",
        );
        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::new(30.0, 0.0, 0.0, 30.0), 8),
            "rectangle corner radii must not overlap",
        );
        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::new(0.0, 30.0, 30.0, 0.0), 8),
            "rectangle corner radii must not overlap",
        );
        // Touching tangent points are accepted: equality is not an overlap.
        assert!(
            RectBuilder::validate_corners(size, CornerRadii::new(25.0, 25.0, 25.0, 25.0), 8)
                .is_ok()
        );
    }

    #[test]
    fn validate_corners_rejects_unrepresentable_combined_radii() {
        let size: Vector2f = Vector2f {
            x: f32::MAX,
            y: f32::MAX,
        };

        assert_invalid_state(
            RectBuilder::validate_corners(size, CornerRadii::new(f32::MAX, f32::MAX, 0.0, 0.0), 8),
            "combined rectangle corner radii must be finite",
        );
    }

    // ---------------------------------------------------------
    // Rounded corners: geometry
    // ---------------------------------------------------------

    fn rounded_geometry(
        size: Vector2f,
        radii: CornerRadii,
        segments: u16,
    ) -> (Vec<Vertex2D>, Vec<u32>) {
        let result: VMNLResult<(Vec<Vertex2D>, Vec<u32>)> = RectBuilder::geometry(
            Vector2f { x: 0.0, y: 0.0 },
            size,
            Rgba::WHITE,
            0.0,
            RectOrigin::Anchor(Anchor::TopLeft),
            radii,
            segments,
        );
        expect_geometry(result)
    }

    fn assert_valid_mesh(vertices: &[Vertex2D], indices: &[u32]) {
        assert_eq!(indices.len() % 3, 0);
        for &index in indices {
            assert!((index as usize) < vertices.len());
        }
        for vertex in vertices {
            assert!(vertex.position.x.is_finite());
            assert!(vertex.position.y.is_finite());
        }
    }

    #[test]
    fn uniform_corners_generate_expected_counts_for_multiple_segment_counts() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };

        for segments in [1_u16, 2, 4, 8, 16] {
            let (vertices, indices) = rounded_geometry(size, CornerRadii::all(10.0), segments);
            let boundary_count: usize = 4 * (usize::from(segments) + 1);

            assert_eq!(vertices.len(), boundary_count + 1, "segments={segments}");
            assert_eq!(indices.len(), 3 * boundary_count, "segments={segments}");
            assert_valid_mesh(&vertices, &indices);
        }
    }

    #[test]
    fn uniform_corners_place_tangent_and_arc_points() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };
        let (vertices, indices) = rounded_geometry(size, CornerRadii::all(10.0), 4);

        // Triangle-fan center at the rectangle center.
        assert_vector_eq(vertices[0].position, Vector2f { x: 50.0, y: 25.0 });

        // Top-left corner arc, ordered clockwise from the left-edge tangent point.
        assert_vector_eq(vertices[1].position, Vector2f { x: 0.0, y: 10.0 });
        assert_vector_eq(
            vertices[3].position,
            Vector2f {
                x: 2.92893,
                y: 2.92893,
            },
        );
        assert_vector_eq(vertices[5].position, Vector2f { x: 10.0, y: 0.0 });

        // Top-right corner arc starts on the top edge and ends on the right edge.
        assert_vector_eq(vertices[6].position, Vector2f { x: 90.0, y: 0.0 });
        assert_vector_eq(vertices[10].position, Vector2f { x: 100.0, y: 10.0 });

        // Every arc point stays on its corner circle.
        for vertex in &vertices[1..6] {
            let dx: f32 = vertex.position.x - 10.0;
            let dy: f32 = vertex.position.y - 10.0;

            assert!((dx * dx + dy * dy).sqrt() - 10.0 < 0.0001);
        }

        // Triangle fan around the center vertex, closing back to the first boundary vertex.
        assert_eq!(&indices[..3], &[0, 1, 2]);
        assert_eq!(&indices[indices.len() - 3..], &[0, 20, 1]);
    }

    #[test]
    fn independent_corners_use_per_corner_radii() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };
        let radii: CornerRadii = CornerRadii::new(10.0, 5.0, 10.0, 5.0);
        let (vertices, indices) = rounded_geometry(size, radii, 4);

        assert_eq!(vertices.len(), 21);
        assert_eq!(indices.len(), 60);

        // Top-left radius 10: left-edge tangent at (0, 10).
        assert_vector_eq(vertices[1].position, Vector2f { x: 0.0, y: 10.0 });
        // Top-right radius 5: top-edge tangent at (95, 0) and right-edge tangent at (100, 5).
        assert_vector_eq(vertices[6].position, Vector2f { x: 95.0, y: 0.0 });
        assert_vector_eq(vertices[10].position, Vector2f { x: 100.0, y: 5.0 });
        // Bottom-right radius 10: bottom-edge tangent at (90, 50).
        assert_vector_eq(vertices[15].position, Vector2f { x: 90.0, y: 50.0 });
        // Bottom-left radius 5: left-edge tangent at (0, 45).
        assert_vector_eq(vertices[20].position, Vector2f { x: 0.0, y: 45.0 });

        assert_valid_mesh(&vertices, &indices);
    }

    #[test]
    fn zero_radius_corners_stay_sharp_next_to_rounded_corners() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };
        let radii: CornerRadii = CornerRadii::new(10.0, 0.0, 10.0, 0.0);
        let (vertices, indices) = rounded_geometry(size, radii, 4);

        // Two rounded corners contribute 5 vertices each; two sharp corners contribute one.
        assert_eq!(vertices.len(), 13);
        assert_eq!(indices.len(), 36);

        // Top-left arc ends on the top edge, then the sharp top-right corner point.
        assert_vector_eq(vertices[5].position, Vector2f { x: 10.0, y: 0.0 });
        assert_vector_eq(vertices[6].position, Vector2f { x: 100.0, y: 0.0 });
        // Bottom-right arc starts on the right edge.
        assert_vector_eq(vertices[7].position, Vector2f { x: 100.0, y: 40.0 });
        // Sharp bottom-left corner closes the contour.
        assert_vector_eq(vertices[12].position, Vector2f { x: 0.0, y: 50.0 });

        assert_valid_mesh(&vertices, &indices);
    }

    #[test]
    fn rounded_geometry_applies_rotation_around_anchor_center() {
        let (vertices, _) = expect_geometry(RectBuilder::geometry(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f { x: 10.0, y: 20.0 },
            Rgba::WHITE,
            90.0,
            RectOrigin::Anchor(Anchor::Center),
            CornerRadii::all(1.0),
            1,
        ));

        // Fan center stays fixed.
        assert_vector_eq(vertices[0].position, Vector2f { x: 5.0, y: 10.0 });
        // Left-edge tangent point (0, 1) rotates to (14, 5).
        assert_vector_eq(vertices[1].position, Vector2f { x: 14.0, y: 5.0 });
    }

    #[test]
    fn rounded_geometry_applies_rotation_around_custom_origin() {
        let (vertices, _) = expect_geometry(RectBuilder::geometry(
            Vector2f { x: 10.0, y: 20.0 },
            Vector2f { x: 4.0, y: 4.0 },
            Rgba::WHITE,
            90.0,
            RectOrigin::Custom(Vector2f { x: 0.0, y: 0.0 }),
            CornerRadii::all(1.0),
            1,
        ));

        // Left-edge tangent point (10, 21) rotates around (10, 20) to (9, 20).
        assert_vector_eq(vertices[1].position, Vector2f { x: 9.0, y: 20.0 });
    }

    #[test]
    fn rounded_geometry_rejects_unrepresentable_transformed_vertices() {
        let result: VMNLResult<(Vec<Vertex2D>, Vec<u32>)> = RectBuilder::geometry(
            Vector2f { x: 0.0, y: 0.0 },
            Vector2f {
                x: 3.0e38,
                y: 3.0e38,
            },
            Rgba::WHITE,
            45.0,
            RectOrigin::Anchor(Anchor::TopLeft),
            CornerRadii::all(1.5e38),
            8,
        );

        assert_invalid_state(
            result.map(|_| ()),
            "rounded rectangle vertices must be finite",
        );
    }

    // ---------------------------------------------------------
    // Rounded corners: checked count arithmetic
    // ---------------------------------------------------------

    #[test]
    fn rounded_counts_use_checked_arithmetic() {
        assert_eq!(
            RectBuilder::rounded_counts(4).map_err(|_| ()),
            Ok((5_usize, 12_usize))
        );
        assert_invalid_state(
            RectBuilder::rounded_counts(usize::MAX).map(|_| ()),
            "rounded rectangle geometry count overflow",
        );
        assert_eq!(
            RectBuilder::corner_vertex_count(1.0, u16::MAX).map_err(|_| ()),
            Ok(65_536_usize)
        );
        assert_eq!(
            RectBuilder::corner_vertex_count(0.0, 8).map_err(|_| ()),
            Ok(1_usize)
        );
    }

    #[test]
    fn maximum_segment_count_generates_representable_counts() {
        let size: Vector2f = Vector2f { x: 100.0, y: 50.0 };
        let segments: u16 = u16::MAX;
        let (vertices, indices) = rounded_geometry(size, CornerRadii::all(10.0), segments);
        let boundary_count: usize = 4 * (usize::from(segments) + 1);

        assert_eq!(vertices.len(), boundary_count + 1);
        assert_eq!(indices.len(), 3 * boundary_count);
        assert_valid_mesh(&vertices, &indices);
    }
}
