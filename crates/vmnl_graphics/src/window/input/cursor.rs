// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shareable native cursor resources.

use crate::glfw_backend::NativeCursor;
use crate::{glfw_backend, Context, VMNLError, VMNLErrorKind, VMNLResult};
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
    /// Creates one native cursor from a standard system shape.
    ///
    /// This call performs a native allocation. Exact appearance and size come from the active
    /// system cursor theme.
    ///
    /// # Errors
    /// Returns [`GlfwUnsupportedPlatform`](VMNLErrorKind::GlfwUnsupportedPlatform) when the active
    /// cursor theme does not provide the requested shape. Other creation failures retain their
    /// corresponding VMNL GLFW error category.
    pub fn standard(context: &Context, shape: StandardCursor) -> VMNLResult<Self> {
        let glfw = context.inner.glfw.clone();
        let native = glfw_backend::create_standard_cursor(shape)?;

        Ok(Self {
            resource: Rc::new(CursorResource {
                native,
                _glfw: glfw,
            }),
        })
    }

    /// Creates one native cursor from packed, non-premultiplied RGBA8 pixels.
    ///
    /// Pixels are arranged as sequential rows from the upper-left corner, with four bytes per
    /// pixel in red, green, blue, alpha order. GLFW copies the pixel slice before this call
    /// returns. The hotspot uses pixel coordinates relative to the upper-left corner.
    ///
    /// This call validates the image before FFI and performs one native cursor allocation.
    ///
    /// # Errors
    /// Returns [`InvalidState`](VMNLErrorKind::InvalidState) when a dimension is zero or exceeds
    /// GLFW's signed integer range, when `pixels.len()` is not exactly `width * height * 4`, or
    /// when the hotspot is outside the image. Native creation failures retain their VMNL GLFW
    /// error category.
    pub fn from_rgba8(
        context: &Context,
        width: u32,
        height: u32,
        pixels: &[u8],
        hotspot: (u32, u32),
    ) -> VMNLResult<Self> {
        let image = validate_rgba8(width, height, pixels.len(), hotspot)?;
        let glfw = context.inner.glfw.clone();
        let native = glfw_backend::create_cursor(image, pixels)?;

        Ok(Self {
            resource: Rc::new(CursorResource {
                native,
                _glfw: glfw,
            }),
        })
    }

    pub(crate) fn native(&self) -> &NativeCursor {
        &self.resource.native
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

#[cfg(test)]
mod tests {
    use super::validate_rgba8;
    use crate::VMNLErrorKind;

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
