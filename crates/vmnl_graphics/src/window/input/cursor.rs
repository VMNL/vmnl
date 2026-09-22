// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shareable native cursor resources.

use crate::glfw_backend::NativeCursor;
use crate::{common::Rgba, glfw_backend, Context, VMNLError, VMNLErrorKind, VMNLResult};
use std::fmt;
use std::os::raw::c_int;
use std::rc::Rc;

/// A standard cursor shape supplied by the active system cursor theme.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StandardCursor {
    /// Regular arrow cursor.
    Arrow,
    /// Text-input I-beam cursor.
    IBeam,
    /// Crosshair cursor.
    Crosshair,
    /// Pointing-hand cursor.
    PointingHand,
    /// Horizontal resize cursor.
    ResizeEastWest,
    /// Vertical resize cursor.
    ResizeNorthSouth,
    /// Northwest-to-southeast diagonal resize cursor.
    ResizeNorthwestSoutheast,
    /// Northeast-to-southwest diagonal resize cursor.
    ResizeNortheastSouthwest,
    /// Omnidirectional resize cursor.
    ResizeAll,
    /// Cursor indicating that an action is not allowed.
    NotAllowed,
}

struct CursorResource {
    native: NativeCursor,
    _glfw: glfw::Glfw,
}

impl Drop for CursorResource {
    fn drop(&mut self) {
        glfw_backend::destroy_cursor(&self.native);
    }
}

/// A clonable native cursor resource that can be shared by multiple windows.
///
/// Cloning shares the same native cursor through single-threaded reference counting. Each window
/// retains its own clone while the cursor is assigned, so dropping a client clone cannot invalidate
/// a cursor still used by a window. The native resource is destroyed when its last owner is dropped.
/// `Cursor` is intentionally neither `Send` nor `Sync` and must remain on the GLFW thread.
#[derive(Clone)]
pub struct Cursor {
    resource: Rc<CursorResource>,
}

impl Cursor {
    /// Configures a cursor using one standard system shape.
    ///
    /// Exact appearance and size come from the active system cursor theme. No native resource is
    /// created until [`StandardCursorBuilder::build`] is called.
    pub const fn standard(shape: StandardCursor) -> StandardCursorBuilder {
        StandardCursorBuilder { shape }
    }

    /// Configures a custom cursor from packed, non-premultiplied RGBA8 pixels.
    ///
    /// Pixels are arranged as sequential rows from the upper-left corner, with four bytes per
    /// pixel in red, green, blue, alpha order. The builder borrows the pixels until
    /// [`CursorBuilder::build`] copies them through GLFW. The hotspot defaults to `(0, 0)`.
    pub const fn rgba8(width: u32, height: u32, pixels: &[u8]) -> CursorBuilder<'_> {
        CursorBuilder {
            width,
            height,
            pixels,
            hotspot: (0, 0),
            hotspot_marker: None,
        }
    }

    fn from_native(context: &Context, native: NativeCursor) -> Self {
        Self::from_native_with_glfw(context.inner.glfw.clone(), native)
    }

    fn from_native_with_glfw(glfw: glfw::Glfw, native: NativeCursor) -> Self {
        Self {
            resource: Rc::new(CursorResource {
                native,
                _glfw: glfw,
            }),
        }
    }

    pub(crate) fn transparent(glfw: &glfw::Glfw) -> VMNLResult<Self> {
        const TRANSPARENT_PIXEL: [u8; 4] = [0; 4];
        let image = CursorImage {
            width: 1,
            height: 1,
            hotspot_x: 0,
            hotspot_y: 0,
        };
        let native = glfw_backend::create_cursor(image, &TRANSPARENT_PIXEL)?;

        Ok(Self::from_native_with_glfw(glfw.clone(), native))
    }

    pub(crate) fn native(&self) -> &NativeCursor {
        &self.resource.native
    }
}

/// Builder for a cursor supplied by the active system cursor theme.
#[must_use = "a standard cursor is not created until build is called"]
pub struct StandardCursorBuilder {
    shape: StandardCursor,
}

impl StandardCursorBuilder {
    /// Creates one native cursor from the configured standard shape.
    ///
    /// This call performs one native allocation.
    ///
    /// # Errors
    /// Returns [`GlfwUnsupportedPlatform`](VMNLErrorKind::GlfwUnsupportedPlatform) when the active
    /// cursor theme does not provide the requested shape. Other creation failures retain their
    /// corresponding VMNL GLFW error category.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Cursor, StandardCursor};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// let context = Context::new()?;
    /// let cursor = Cursor::standard(StandardCursor::PointingHand).build(&context)?;
    /// # let _ = cursor;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Cursor> {
        let native = glfw_backend::create_standard_cursor(self.shape)?;
        Ok(Cursor::from_native(context, native))
    }
}

/// Builder for a custom cursor backed by borrowed packed RGBA8 pixels.
///
/// The pixel slice only needs to remain valid until [`build`](Self::build) returns because GLFW
/// copies it synchronously. The hotspot defaults to the upper-left pixel `(0, 0)` and its optional
/// visual marker is disabled by default.
#[must_use = "a custom cursor is not created until build is called"]
pub struct CursorBuilder<'pixels> {
    width: u32,
    height: u32,
    pixels: &'pixels [u8],
    hotspot: (u32, u32),
    hotspot_marker: Option<Rgba>,
}

impl CursorBuilder<'_> {
    /// Sets the active hotspot in image pixels relative to the upper-left corner.
    pub const fn hotspot(mut self, x: u32, y: u32) -> Self {
        self.hotspot = (x, y);
        self
    }

    /// Overwrites the final hotspot pixel with a diagnostic color during [`build`](Self::build).
    ///
    /// The marker is applied to an internal temporary copy and never mutates the borrowed source
    /// slice. The caller controls visibility by choosing a color that contrasts with the cursor;
    /// VMNL does not alter the supplied RGBA components. Enabling the marker performs one temporary
    /// `width * height * 4` byte allocation during `build`.
    pub fn hotspot_marker<C>(mut self, color: C) -> Self
    where
        C: Into<Rgba>,
    {
        self.hotspot_marker = Some(color.into());
        self
    }

    /// Creates one native cursor from the configured RGBA8 image and hotspot.
    ///
    /// This call validates the image before FFI, synchronously copies the pixels through GLFW, and
    /// performs one native allocation. When a hotspot marker is configured, it first copies the
    /// complete source image into one temporary VMNL buffer and overwrites exactly the final
    /// hotspot pixel.
    ///
    /// # Errors
    /// Returns [`InvalidState`](VMNLErrorKind::InvalidState) when a dimension is zero or exceeds
    /// GLFW's signed integer range, when `pixels.len()` is not exactly `width * height * 4`, or
    /// when the hotspot is outside the image. Native creation failures retain their VMNL GLFW
    /// error category.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{common::Rgba, Context, Cursor};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// let context = Context::new()?;
    /// let pixels = [255_u8; 4 * 4 * 4];
    /// let cursor = Cursor::rgba8(4, 4, &pixels)
    ///     .hotspot(2, 2)
    ///     .hotspot_marker(Rgba::MAGENTA)
    ///     .build(&context)?;
    /// # let _ = cursor;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self, context: &Context) -> VMNLResult<Cursor> {
        let image = validate_rgba8(self.width, self.height, self.pixels.len(), self.hotspot)?;
        let native = if let Some(color) = self.hotspot_marker {
            let marked_pixels =
                pixels_with_hotspot_marker(self.width, self.pixels, self.hotspot, color)?;
            glfw_backend::create_cursor(image, &marked_pixels)?
        } else {
            glfw_backend::create_cursor(image, self.pixels)?
        };
        Ok(Cursor::from_native(context, native))
    }
}

impl fmt::Debug for Cursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Cursor").finish_non_exhaustive()
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.resource, &other.resource)
    }
}

impl Eq for Cursor {}

#[derive(Clone, Copy)]
pub(crate) struct CursorImage {
    pub(crate) width: c_int,
    pub(crate) height: c_int,
    pub(crate) hotspot_x: c_int,
    pub(crate) hotspot_y: c_int,
}

fn validate_rgba8(
    width: u32,
    height: u32,
    pixel_bytes: usize,
    hotspot: (u32, u32),
) -> VMNLResult<CursorImage> {
    if width == 0 || height == 0 {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "cursor dimensions must be greater than zero".into(),
        )));
    }

    let native_width = c_int::try_from(width).map_err(|_| {
        VMNLError::new(VMNLErrorKind::InvalidState(
            "cursor width exceeds GLFW's signed integer range".into(),
        ))
    })?;
    let native_height = c_int::try_from(height).map_err(|_| {
        VMNLError::new(VMNLErrorKind::InvalidState(
            "cursor height exceeds GLFW's signed integer range".into(),
        ))
    })?;
    let expected_bytes = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "cursor RGBA8 byte length overflows usize".into(),
            ))
        })?;
    if pixel_bytes != expected_bytes {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "cursor RGBA8 data has {pixel_bytes} bytes but requires {expected_bytes}"
        ))));
    }

    if hotspot.0 >= width || hotspot.1 >= height {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "cursor hotspot must be inside the image".into(),
        )));
    }

    Ok(CursorImage {
        width: native_width,
        height: native_height,
        hotspot_x: c_int::try_from(hotspot.0).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "cursor hotspot x exceeds GLFW's signed integer range".into(),
            ))
        })?,
        hotspot_y: c_int::try_from(hotspot.1).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "cursor hotspot y exceeds GLFW's signed integer range".into(),
            ))
        })?,
    })
}

fn pixels_with_hotspot_marker(
    width: u32,
    pixels: &[u8],
    hotspot: (u32, u32),
    color: Rgba,
) -> VMNLResult<Vec<u8>> {
    let byte_offset = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(hotspot.1)
                .ok()
                .and_then(|y| y.checked_mul(width))
        })
        .and_then(|row| {
            usize::try_from(hotspot.0)
                .ok()
                .and_then(|x| row.checked_add(x))
        })
        .and_then(|pixel| pixel.checked_mul(4))
        .ok_or_else(|| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "cursor hotspot byte offset overflows usize".into(),
            ))
        })?;
    let marker_end = byte_offset.checked_add(4).ok_or_else(|| {
        VMNLError::new(VMNLErrorKind::InvalidState(
            "cursor hotspot byte range overflows usize".into(),
        ))
    })?;
    let mut marked_pixels = pixels.to_vec();
    let marker = marked_pixels
        .get_mut(byte_offset..marker_end)
        .ok_or_else(|| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "cursor hotspot marker falls outside the image".into(),
            ))
        })?;
    marker.copy_from_slice(&[color.r, color.g, color.b, color.a]);
    Ok(marked_pixels)
}

#[cfg(test)]
mod tests {
    use super::{pixels_with_hotspot_marker, validate_rgba8, Cursor, StandardCursor};
    use crate::{common::Rgba, VMNLErrorKind};

    #[test]
    fn standard_builder_preserves_shape() {
        let builder = Cursor::standard(StandardCursor::PointingHand);

        assert_eq!(builder.shape, StandardCursor::PointingHand);
    }

    #[test]
    fn rgba8_builder_uses_upper_left_hotspot_by_default() {
        let pixels = [255_u8; 16];
        let builder = Cursor::rgba8(2, 2, &pixels);

        assert_eq!(builder.width, 2);
        assert_eq!(builder.height, 2);
        assert_eq!(builder.pixels, pixels);
        assert_eq!(builder.hotspot, (0, 0));
        assert_eq!(builder.hotspot_marker, None);
    }

    #[test]
    fn rgba8_builder_configures_hotspot() {
        let pixels = [255_u8; 16];
        let builder = Cursor::rgba8(2, 2, &pixels).hotspot(1, 1);

        assert_eq!(builder.hotspot, (1, 1));
    }

    #[test]
    fn rgba8_builder_configures_hotspot_marker_from_public_color_input() {
        let pixels = [0_u8; 16];
        let builder = Cursor::rgba8(2, 2, &pixels)
            .hotspot_marker([1, 2, 3])
            .hotspot(1, 1);

        assert_eq!(builder.hotspot, (1, 1));
        assert_eq!(builder.hotspot_marker, Some(Rgba::rgba(1, 2, 3, 255)));
    }

    #[test]
    fn hotspot_marker_overwrites_only_the_final_hotspot_pixel() -> crate::VMNLResult<()> {
        let pixels = [0_u8; 16];

        let marked = pixels_with_hotspot_marker(2, &pixels, (1, 0), Rgba::rgba(10, 20, 30, 40))?;

        assert_eq!(pixels, [0_u8; 16]);
        assert_eq!(&marked[0..4], &[0, 0, 0, 0]);
        assert_eq!(&marked[4..8], &[10, 20, 30, 40]);
        assert_eq!(&marked[8..16], &[0_u8; 8]);
        Ok(())
    }

    #[test]
    fn rgba8_validation_accepts_exact_data_and_in_bounds_hotspot() {
        let image = validate_rgba8(2, 3, 24, (1, 2));
        assert!(image.is_ok());
    }

    #[test]
    fn rgba8_validation_rejects_empty_dimensions() {
        for dimensions in [(0, 1), (1, 0), (0, 0)] {
            let result = validate_rgba8(dimensions.0, dimensions.1, 0, (0, 0));
            assert!(matches!(
                result,
                Err(error) if matches!(error.kind(), VMNLErrorKind::InvalidState(message) if message == "cursor dimensions must be greater than zero")
            ));
        }
    }

    #[test]
    fn rgba8_validation_rejects_wrong_byte_length() {
        for length in [15, 17] {
            let result = validate_rgba8(2, 2, length, (0, 0));
            assert!(matches!(
                result,
                Err(error) if matches!(error.kind(), VMNLErrorKind::InvalidState(message) if message.contains("requires 16"))
            ));
        }
    }

    #[test]
    fn rgba8_validation_rejects_out_of_bounds_hotspot() {
        for hotspot in [(2, 0), (0, 3), (2, 3)] {
            let result = validate_rgba8(2, 3, 24, hotspot);
            assert!(matches!(
                result,
                Err(error) if matches!(error.kind(), VMNLErrorKind::InvalidState(message) if message == "cursor hotspot must be inside the image")
            ));
        }
    }

    #[test]
    fn rgba8_validation_rejects_dimensions_outside_glfw_range() {
        let result = validate_rgba8(u32::MAX, 1, 0, (0, 0));
        assert!(matches!(
            result,
            Err(error) if matches!(error.kind(), VMNLErrorKind::InvalidState(message) if message == "cursor width exceeds GLFW's signed integer range")
        ));
    }
}
