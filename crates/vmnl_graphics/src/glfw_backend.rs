// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Private boundary for GLFW diagnostics and platform-sensitive operations.

use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::path::Path;

/// Prints an explicitly requested controller diagnostic, without changing mappings.
/// This opt-in output uses stderr so example stdout filters do not hide it.
#[allow(clippy::print_stderr)]
pub(crate) fn print_gamepad_diagnostics(glfw: &glfw::Glfw) {
    if std::env::var_os("VMNL_GAMEPAD_DIAGNOSTICS").is_none() {
        return;
    }
    eprintln!(
        "[gamepad diagnostic] mapping_file={:?}; VMNL reads slot 1 only",
        std::env::var_os("VMNL_GAMEPAD_MAPPINGS")
    );
    let mut found = false;
    for slot in 0..16 {
        let Some(id) = glfw::JoystickId::from_i32(slot) else {
            continue;
        };
        let joystick = glfw.get_joystick(id);
        if !joystick.is_present() {
            continue;
        }
        found = true;
        let axes = joystick.get_axes();
        let buttons = joystick.get_buttons();
        let hat_count = joystick.get_hats().len();
        eprintln!(
            "[gamepad diagnostic] slot={} name={:?} GUID={:?} mapped={}",
            slot + 1,
            joystick.get_name(),
            joystick.get_guid(),
            joystick.is_gamepad()
        );
        eprintln!(
            "[gamepad diagnostic] raw_axes count={} values={axes:?}",
            axes.len()
        );
        eprintln!(
            "[gamepad diagnostic] raw_buttons count={} values={buttons:?}",
            buttons.len()
        );
        eprintln!("[gamepad diagnostic] raw_hat_count={hat_count} (button array may include hat directions)");
        eprintln!(
            "[gamepad diagnostic] mapped_state={:?}",
            joystick.get_gamepad_state()
        );
    }
    if !found {
        eprintln!("[gamepad diagnostic] no controller present in any GLFW slot");
    }
}

/// Loads an optional SDL-format mapping file once during context initialization.
/// Mappings are GLFW-global and remain installed while GLFW stays initialized.
pub(crate) fn configure_gamepad_mappings(glfw: &glfw::Glfw) -> VMNLResult<()> {
    let Some(path) = std::env::var_os("VMNL_GAMEPAD_MAPPINGS") else {
        return Ok(());
    };
    load_gamepad_mappings(Path::new(&path), |mappings| {
        glfw.update_gamepad_mappings(mappings)
    })
}

fn load_gamepad_mappings(path: &Path, apply: impl FnOnce(&str) -> bool) -> VMNLResult<()> {
    let mappings = std::fs::read_to_string(path).map_err(|error| {
        VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "cannot read gamepad mappings from {}: {error}",
            path.display()
        )))
    })?;
    // The GLFW Rust wrapper converts the input to a C string and cannot accept NUL bytes.
    if mappings.contains('\0') || !mappings.is_ascii() {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "gamepad mappings must contain ASCII text without NUL bytes".into(),
        )));
    }
    if !apply(&mappings) {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "GLFW rejected gamepad mappings from {}",
            path.display()
        ))));
    }
    Ok(())
}

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
    #[test]
    fn mapping_file_is_read_once_and_backend_rejection_is_reported() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/window/events_input/gamecontrollerdb.txt");
        let mut calls = 0;
        let result = super::load_gamepad_mappings(&path, |text| {
            calls += 1;
            text.lines()
                .any(|line| line.starts_with("030000005e0400008e02000045050000,"))
        });
        assert!(result.is_ok());
        assert_eq!(calls, 1);
        assert!(super::load_gamepad_mappings(&path, |_| false).is_err());
        assert!(super::load_gamepad_mappings(&path.join("missing"), |_| true).is_err());
    }

    #[test]
    fn invalid_mapping_text_is_rejected_before_glfw() -> std::io::Result<()> {
        let path =
            std::env::temp_dir().join(format!("vmnl-invalid-mapping-{}.txt", std::process::id()));
        for text in ["bad\0mapping", "non-ASCII: é"] {
            std::fs::write(&path, text)?;
            let mut called = false;
            let result = super::load_gamepad_mappings(&path, |_| {
                called = true;
                true
            });
            assert!(result.is_err());
            assert!(!called);
        }
        std::fs::remove_file(path)?;
        Ok(())
    }
}
