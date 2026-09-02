// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Private boundary for GLFW diagnostics and platform-sensitive operations.

use crate::VMNLErrorKind;

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

#[cfg(test)]
mod tests {
    use super::{callback_message, map_error};
    use crate::VMNLErrorKind;

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
