////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Line shape utilities for the VMNL graphics module,
/// providing functions to create lines defined by start and end points, width, cap style, and color.
////////////////////////////////////////////////////////////////////////////////
use super::{Shape, Vector2f};
use crate::{Context, Rgba, VMNLError, VMNLErrorKind, VMNLResult};

/// Line cap styles for rendering line endpoints.
#[derive(Debug, Clone, PartialEq, Hash, Eq, Default)]
pub enum LineCap {
    /// No additional geometry is added at the line endpoints; the line simply ends at the specified points.
    #[default]
    Butt,
    /// Semi-circular geometry is added at the line endpoints, creating rounded ends that extend beyond the specified points by half the line width.
    Round,
    /// Rectangular geometry is added at the line endpoints, creating squared ends that extend beyond the specified points by half the line width.
    Square,
}

/// Options for configuring line shape properties such as endpoints, width, cap style, and color.
#[derive(Clone, Debug)]
struct LineOptions {
    /// Starting point of the line as a `Vector2f`.
    from: Vector2f,
    /// Ending point of the line as a `Vector2f`.
    to: Vector2f,
    /// Width of the line in pixels. Must be a positive value.
    width: f32,
    /// Line cap style defining how the endpoints of the line are rendered.
    cap: LineCap,
    /// RGBA color of the line as an array of four `f32` values in the range `[0, 255]`, representing red, green, blue, and alpha components respectively.
    color: Rgba,
}

/// Builder for creating line shapes with configurable properties such as endpoints, width, cap style, and color.
pub struct LineBuilder {
    /// Configuration options for the line shape, including endpoints, width, cap style, and color.
    options: LineOptions,
}

impl Default for LineOptions {
    /// Create a default `LineOptions` instance with default values for all properties.
    fn default() -> Self {
        Self {
            from: Vector2f { x: 0.0, y: 0.0 },
            to: Vector2f { x: 0.0, y: 0.0 },
            width: 1.0,
            cap: LineCap::Butt,
            color: Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
        }
    }
}

impl LineBuilder {
    pub(crate) fn new(from: Vector2f, to: Vector2f) -> Self {
        Self {
            options: LineOptions {
                from,
                to,
                ..Default::default()
            },
        }
    }

    /// Set the width of the line.
    ///
    /// # Arguments
    /// - `width`: The desired width of the line in pixels. Must be a positive value.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, Rgba, Shape, Vector2f};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .width(5.0)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn width(mut self, width: f32) -> Self {
        self.options.width = width;
        self
    }

    /// Set the line cap style.
    ///
    /// # Arguments
    /// - `cap`: The desired line cap style, which can be `Butt`, `Round`, or `Square`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, LineCap, Rgba, Shape, Vector2f};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .cap(LineCap::Round)
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn cap(mut self, cap: LineCap) -> Self {
        self.options.cap = cap;
        self
    }

    /// Set the color of the line using RGBA values in the range `[0, 255]`.
    ///
    /// # Arguments
    /// - `color`: RGBA color of the line as an array of four `f32` values in the range `[0, 255]`, representing red, green, blue, and alpha components respectively.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, Rgba, Shape, Vector2f};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .color(Rgba::new(0, 0, 255, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn color(mut self, color: Rgba) -> Self {
        self.options.color = color;
        self
    }

    /// Create a `Shape` instance by transforming the input line parameters into a vertex buffer.
    ///
    /// # Arguments
    /// - `vmnl_context`: Reference to the VMNL context providing the memory allocator.
    /// - `from`: Starting point of the line as a `Vector2f`.
    /// - `to`: Ending point of the line as a `Vector2f`.
    /// - `width`: Optional width of the line (default is `1.0`).
    /// - `cap`: Optional line cap style (default is `LineCap::Butt`).
    /// - `color`: Optional RGBA color of the line (default is white `Rgba::new(255, 255, 255, 255)`).
    ///
    /// # Returns
    /// A `Shape` instance representing the line, ready for rendering.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_native::{Context, LineCap, Rgba, Shape, Vector2f};
    /// # fn main() -> vmnl_native::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// let line = Shape::line(Vector2f { x: 100.0, y: 150.0 }, Vector2f { x: 300.0, y: 150.0 })
    ///     .width(5.0)
    ///     .cap(LineCap::Round)
    ///     .color(Rgba::new(0, 0, 255, 255))
    ///     .build(&context)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Shape> {
        Self::line(
            context,
            self.options.from,
            self.options.to,
            self.options.width,
            self.options.cap,
            self.options.color,
        )
    }

    /// Create a line shape defined by required `from` and `to` endpoints, optional `width`, optional `cap` style, and optional single `color`.
    ///
    /// `width` defaults to `1.0`, `cap` defaults to `Butt`, and `color` defaults to white.
    ///
    /// # Arguments
    /// - `context`: Reference to the VMNL context providing the memory allocator.
    /// - `from`: Starting point of the line as a `Vector2f`.
    /// - `to`: Ending point of the line as a `Vector2f`.
    /// - `width`: Optional width of the line (default is `1.0`).
    /// - `cap`: Optional line cap style (default is `LineCap::Butt`).
    /// - `color`: Optional RGBA color of the line (default is white `Rgba::new(255, 255, 255, 255)`).
    ///
    /// # Returns
    /// A `Shape` instance representing the line, ready for rendering.
    fn line(
        context: &Context,
        from: Vector2f,
        to: Vector2f,
        width: f32,
        cap: LineCap,
        color: Rgba,
    ) -> VMNLResult<Shape> {
        let _ = (context, cap, color);

        if from == to {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line endpoints must be distinct".to_string(),
            )));
        }
        if width <= 0.0 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "line width must be strictly positive".to_string(),
            )));
        }
        todo!("line shape rendering is not implemented yet")
    }
}
