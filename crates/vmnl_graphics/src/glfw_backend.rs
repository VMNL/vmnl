// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Private boundary for GLFW diagnostics and platform-sensitive operations.

use crate::window::CursorImage;
use crate::{StandardCursor, VMNLError, VMNLErrorKind, VMNLResult};
use std::ptr::NonNull;

pub(crate) struct NativeCursor(NonNull<glfw::ffi::GLFWcursor>);

pub(crate) fn init(
    callback: impl FnMut(glfw::Error, String) + 'static,
) -> Result<glfw::Glfw, glfw::InitError> {
    glfw::init(callback)
}

pub(crate) fn set_error_callback(
    glfw: &mut glfw::Glfw,
    mut callback: impl FnMut(VMNLErrorKind, String) + 'static,
) {
    glfw.set_error_callback(move |error, description| {
        callback(map_error(error), callback_message(error, description));
    });
}

pub(crate) fn map_error(error: glfw::Error) -> VMNLErrorKind {
    match error {
        glfw::Error::ApiUnavailable
        | glfw::Error::CursorUnavailable
        | glfw::Error::FeatureUnavailable
        | glfw::Error::FeatureUnimplemented
        | glfw::Error::PlatformUnavailable => VMNLErrorKind::GlfwUnsupportedPlatform,
        glfw::Error::VersionUnavailable => VMNLErrorKind::GlfwVersionMismatch,
        glfw::Error::PlatformError => VMNLErrorKind::GlfwPlatformError,
        glfw::Error::NotInitialized => VMNLErrorKind::GlfwInitFailed,
        glfw::Error::NoCurrentContext | glfw::Error::NoWindowContext => {
            VMNLErrorKind::GlfwContextCreationFailed
        }
        // Invalid values, allocation/format failures, unexpected NoError, unknown raw codes and
        // future non-exhaustive variants all retain the conservative unknown category.
        _ => VMNLErrorKind::GlfwUnknownError,
    }
}

fn callback_message(error: glfw::Error, description: String) -> String {
    match error {
        glfw::Error::Unknown(code) => format!("GLFW unknown error {code}: {description}"),
        _ => description,
    }
}

pub(crate) fn backend_name(glfw: &glfw::Glfw) -> &'static str {
    match glfw.get_platform() {
        glfw::Platform::Any => "any",
        glfw::Platform::Null => "null",
        glfw::Platform::Wayland => "wayland",
        glfw::Platform::X11 => "x11",
        glfw::Platform::Win32 => "win32",
        glfw::Platform::MacOS => "cocoa",
    }
}

pub(crate) fn set_aspect_ratio(
    window: &mut glfw::PWindow,
    numerator: std::ffi::c_int,
    denominator: std::ffi::c_int,
) {
    use glfw::Context as _;

    // SAFETY: `window` owns a live GLFW window for the duration of this exclusive borrow. VMNL
    // validates both terms before this call, or supplies GLFW_DONT_CARE for both terms.
    unsafe {
        glfw::ffi::glfwSetWindowAspectRatio(window.window_ptr(), numerator, denominator);
    }
}

pub(crate) fn create_standard_cursor(shape: StandardCursor) -> VMNLResult<NativeCursor> {
    clear_error();
    let native_shape = standard_cursor_shape(shape);

    // SAFETY: GLFW is kept initialized by the calling `Context`; `native_shape` is one of the ten
    // GLFW 3.4 standard cursor constants, and the call runs on the context's platform thread.
    let cursor = unsafe { glfw::ffi::glfwCreateStandardCursor(native_shape) };
    NonNull::new(cursor)
        .map(NativeCursor)
        .ok_or_else(last_error)
}

const fn standard_cursor_shape(shape: StandardCursor) -> std::os::raw::c_int {
    match shape {
        StandardCursor::Arrow => glfw::ffi::GLFW_ARROW_CURSOR,
        StandardCursor::IBeam => glfw::ffi::GLFW_IBEAM_CURSOR,
        StandardCursor::Crosshair => glfw::ffi::GLFW_CROSSHAIR_CURSOR,
        StandardCursor::PointingHand => glfw::ffi::GLFW_POINTING_HAND_CURSOR,
        StandardCursor::ResizeEastWest => glfw::ffi::GLFW_RESIZE_EW_CURSOR,
        StandardCursor::ResizeNorthSouth => glfw::ffi::GLFW_RESIZE_NS_CURSOR,
        StandardCursor::ResizeNorthwestSoutheast => glfw::ffi::GLFW_RESIZE_NWSE_CURSOR,
        StandardCursor::ResizeNortheastSouthwest => glfw::ffi::GLFW_RESIZE_NESW_CURSOR,
        StandardCursor::ResizeAll => glfw::ffi::GLFW_RESIZE_ALL_CURSOR,
        StandardCursor::NotAllowed => glfw::ffi::GLFW_NOT_ALLOWED_CURSOR,
    }
}

pub(crate) fn create_cursor(image: CursorImage, pixels: &[u8]) -> VMNLResult<NativeCursor> {
    clear_error();
    let native_image = glfw::ffi::GLFWimage {
        width: image.width,
        height: image.height,
        pixels: pixels.as_ptr().cast_mut(),
    };

    // SAFETY: `CursorImage` is constructed after validation or from VMNL's fixed one-pixel
    // transparent image. Both paths provide positive dimensions, an exact RGBA8 byte length and
    // an in-bounds hotspot. GLFW copies `pixels` before returning.
    let cursor = unsafe {
        glfw::ffi::glfwCreateCursor(&raw const native_image, image.hotspot_x, image.hotspot_y)
    };
    NonNull::new(cursor)
        .map(NativeCursor)
        .ok_or_else(last_error)
}

pub(crate) fn set_cursor(
    window: &mut glfw::PWindow,
    cursor: Option<&NativeCursor>,
) -> VMNLResult<()> {
    use glfw::Context as _;

    clear_error();
    // SAFETY: `window` owns a live GLFW window for this exclusive borrow. A supplied cursor is a
    // live native handle retained by the caller until this operation succeeds and the window
    // stores its own shared owner.
    unsafe {
        glfw::ffi::glfwSetCursor(
            window.window_ptr(),
            cursor.map_or(std::ptr::null_mut(), |cursor| cursor.0.as_ptr()),
        );
    }
    operation_result()
}

pub(crate) fn set_cursor_mode(
    window: &mut glfw::PWindow,
    mode: glfw::CursorMode,
) -> VMNLResult<()> {
    use glfw::Context as _;

    clear_error();
    // SAFETY: `window` owns a live GLFW window for this exclusive borrow, `mode` is a valid GLFW
    // cursor-mode constant, and VMNL's window API is confined to the GLFW platform thread.
    unsafe {
        glfw::ffi::glfwSetInputMode(
            window.window_ptr(),
            glfw::ffi::GLFW_CURSOR,
            mode as std::os::raw::c_int,
        );
    }
    operation_result()
}

pub(crate) fn destroy_cursor(cursor: &NativeCursor) {
    // SAFETY: `NativeCursor` is created only from a non-null GLFW cursor handle and destroyed once
    // by its owning `CursorResource`. That resource retains a GLFW token through this call.
    unsafe {
        glfw::ffi::glfwDestroyCursor(cursor.0.as_ptr());
    }
}

fn clear_error() {
    let _ = glfw::get_error();
}

fn operation_result() -> VMNLResult<()> {
    let (error, _) = glfw::get_error_string();
    if error == glfw::Error::NoError {
        Ok(())
    } else {
        Err(VMNLError::new(map_error(error)))
    }
}

fn last_error() -> VMNLError {
    let (error, _) = glfw::get_error_string();
    VMNLError::new(map_error(error))
}

#[cfg(test)]
mod tests {
    use super::{callback_message, map_error, standard_cursor_shape};
    use crate::{StandardCursor, VMNLErrorKind};

    #[test]
    fn maps_every_standard_cursor_to_its_glfw_3_4_shape() {
        let cases = [
            (StandardCursor::Arrow, glfw::ffi::GLFW_ARROW_CURSOR),
            (StandardCursor::IBeam, glfw::ffi::GLFW_IBEAM_CURSOR),
            (StandardCursor::Crosshair, glfw::ffi::GLFW_CROSSHAIR_CURSOR),
            (
                StandardCursor::PointingHand,
                glfw::ffi::GLFW_POINTING_HAND_CURSOR,
            ),
            (
                StandardCursor::ResizeEastWest,
                glfw::ffi::GLFW_RESIZE_EW_CURSOR,
            ),
            (
                StandardCursor::ResizeNorthSouth,
                glfw::ffi::GLFW_RESIZE_NS_CURSOR,
            ),
            (
                StandardCursor::ResizeNorthwestSoutheast,
                glfw::ffi::GLFW_RESIZE_NWSE_CURSOR,
            ),
            (
                StandardCursor::ResizeNortheastSouthwest,
                glfw::ffi::GLFW_RESIZE_NESW_CURSOR,
            ),
            (StandardCursor::ResizeAll, glfw::ffi::GLFW_RESIZE_ALL_CURSOR),
            (
                StandardCursor::NotAllowed,
                glfw::ffi::GLFW_NOT_ALLOWED_CURSOR,
            ),
        ];

        for (shape, expected) in cases {
            assert_eq!(standard_cursor_shape(shape), expected);
        }
    }

    #[test]
    fn maps_glfw_errors_to_vmnl_categories() {
        let cases = [
            (
                glfw::Error::ApiUnavailable,
                VMNLErrorKind::GlfwUnsupportedPlatform,
            ),
            (
                glfw::Error::CursorUnavailable,
                VMNLErrorKind::GlfwUnsupportedPlatform,
            ),
            (
                glfw::Error::FeatureUnavailable,
                VMNLErrorKind::GlfwUnsupportedPlatform,
            ),
            (
                glfw::Error::FeatureUnimplemented,
                VMNLErrorKind::GlfwUnsupportedPlatform,
            ),
            (
                glfw::Error::PlatformUnavailable,
                VMNLErrorKind::GlfwUnsupportedPlatform,
            ),
            (
                glfw::Error::VersionUnavailable,
                VMNLErrorKind::GlfwVersionMismatch,
            ),
            (glfw::Error::PlatformError, VMNLErrorKind::GlfwPlatformError),
            (glfw::Error::NotInitialized, VMNLErrorKind::GlfwInitFailed),
            (
                glfw::Error::NoWindowContext,
                VMNLErrorKind::GlfwContextCreationFailed,
            ),
            (
                glfw::Error::NoCurrentContext,
                VMNLErrorKind::GlfwContextCreationFailed,
            ),
            (glfw::Error::NoError, VMNLErrorKind::GlfwUnknownError),
            (glfw::Error::InvalidEnum, VMNLErrorKind::GlfwUnknownError),
            (glfw::Error::InvalidValue, VMNLErrorKind::GlfwUnknownError),
            (glfw::Error::OutOfMemory, VMNLErrorKind::GlfwUnknownError),
            (
                glfw::Error::FormatUnavailable,
                VMNLErrorKind::GlfwUnknownError,
            ),
            (glfw::Error::Unknown(-42), VMNLErrorKind::GlfwUnknownError),
        ];

        for (source, expected) in cases {
            assert_eq!(
                std::mem::discriminant(&map_error(source)),
                std::mem::discriminant(&expected)
            );
        }
    }

    #[test]
    fn unknown_error_message_preserves_raw_code() {
        assert_eq!(
            callback_message(glfw::Error::Unknown(-42), "details".to_owned()),
            "GLFW unknown error -42: details"
        );
    }
}
