// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Private boundary for GLFW diagnostics and platform-sensitive operations.

use crate::{VMNLErrorKind, VMNLResult};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub(crate) mod joysticks;
mod mappings;
use mappings::load_gamepad_mappings;
pub(crate) use mappings::{apply_gamepad_mappings, init, MappingErrorCapture};
#[cfg(test)]
use mappings::{recording_callback, MappingCaptureScope};

pub(crate) fn set_error_callback(
    glfw: &mut glfw::Glfw,
    capture: &Rc<RefCell<MappingErrorCapture>>,
    mut callback: impl FnMut(VMNLErrorKind, String) + 'static,
) {
    mappings::set_error_callback(glfw, capture, move |error, description| {
        callback(map_error(error), callback_message(error, description));
    });
}

pub(crate) use mappings::unset_error_callback;

/// Prints an explicitly requested controller diagnostic, without changing mappings.
/// This opt-in output uses stderr so example stdout filters do not hide it.
#[allow(clippy::print_stderr)]
pub(crate) fn print_gamepad_diagnostics(glfw: &glfw::Glfw) {
    if std::env::var_os("VMNL_GAMEPAD_DIAGNOSTICS").is_none() {
        return;
    }
    eprintln!(
        "[gamepad diagnostic] mapping_file={:?}; VMNL reads all 16 slots",
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
pub(crate) fn configure_gamepad_mappings(
    glfw: &glfw::Glfw,
    capture: &RefCell<MappingErrorCapture>,
) -> VMNLResult<()> {
    let Some(path) = std::env::var_os("VMNL_GAMEPAD_MAPPINGS") else {
        return Ok(());
    };
    load_gamepad_mappings(Path::new(&path), capture, |mappings| {
        glfw.update_gamepad_mappings(mappings)
    })
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

    fn apply_test_mappings(text: &str, apply: impl FnOnce(&str) -> bool) -> crate::VMNLResult<()> {
        super::apply_gamepad_mappings(text, &std::cell::RefCell::default(), apply)
    }

    #[test]
    fn mapping_callback_error_overrides_true_and_preserves_user_callback() {
        let capture = std::rc::Rc::new(std::cell::RefCell::new(
            super::MappingErrorCapture::default(),
        ));
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let observed = std::rc::Rc::clone(&calls);
        let callback_capture = std::rc::Rc::clone(&capture);
        let mut callback = super::recording_callback(&capture, move |_, _| {
            // The application can borrow the recorder: the adapter released its borrow.
            assert!(callback_capture.borrow().error.is_some());
            observed.set(observed.get() + 1);
        });
        let result = super::apply_gamepad_mappings("0, broken, leftx:a0,", &capture, |_| {
            callback(
                glfw::Error::InvalidValue,
                "Invalid value for parameter".into(),
            );
            true
        });
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(matches!(error.kind(), VMNLErrorKind::InvalidState(_)));
            assert!(error.to_string().contains("Invalid value for parameter"));
        }
        assert_eq!(calls.get(), 1);
        assert!(!capture.borrow().active);
        assert!(super::apply_gamepad_mappings("valid", &capture, |_| true).is_ok());
    }

    #[test]
    fn mapping_capture_ignores_old_errors_and_rejects_nested_updates() {
        let capture = std::cell::RefCell::new(super::MappingErrorCapture::default());
        capture
            .borrow_mut()
            .record(glfw::Error::InvalidValue, "old error");
        assert!(super::apply_gamepad_mappings("valid", &capture, |_| {
            let mut called = false;
            let nested = super::apply_gamepad_mappings("nested", &capture, |_| {
                called = true;
                true
            });
            assert!(nested.is_err());
            assert!(!called);
            assert!(capture.borrow().active);
            true
        })
        .is_ok());
        assert!(!capture.borrow().active);
    }

    #[test]
    fn mapping_capture_scope_cleans_up_on_unwind() {
        let capture = std::cell::RefCell::new(super::MappingErrorCapture::default());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _scope = super::MappingCaptureScope::begin(&capture);
            std::panic::resume_unwind(Box::new("simulated Rust closure panic"));
        }));
        assert!(result.is_err());
        assert!(!capture.borrow().active);
        assert!(super::apply_gamepad_mappings("valid", &capture, |_| true).is_ok());
    }

    #[test]
    fn file_mapping_errors_use_the_same_capture() {
        let capture = std::cell::RefCell::new(super::MappingErrorCapture::default());
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/window/events_input/gamecontrollerdb.txt");
        let result = super::load_gamepad_mappings(&path, &capture, |_| {
            capture
                .borrow_mut()
                .record(glfw::Error::InvalidValue, "file parser error");
            true
        });
        assert!(result.is_err());
    }

    #[test]
    fn runtime_mapping_validation_precedes_backend_and_reports_rejection() {
        for invalid in ["bad\0mapping", "non-ASCII: é"] {
            let mut called = false;
            assert!(apply_test_mappings(invalid, |_| {
                called = true;
                true
            })
            .is_err());
            assert!(!called);
        }
        assert!(apply_test_mappings("mapping", |text| text == "mapping").is_ok());
        assert!(apply_test_mappings("mapping", |_| false).is_err());
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
    #[test]
    fn mapping_file_is_read_once_and_backend_rejection_is_reported() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/window/events_input/gamecontrollerdb.txt");
        let mut calls = 0;
        let result = super::load_gamepad_mappings(&path, &std::cell::RefCell::default(), |text| {
            calls += 1;
            text.lines()
                .any(|line| line.starts_with("030000005e0400008e02000045050000,"))
        });
        assert!(result.is_ok());
        assert_eq!(calls, 1);
        assert!(
            super::load_gamepad_mappings(&path, &std::cell::RefCell::default(), |_| false).is_err()
        );
        assert!(super::load_gamepad_mappings(
            &path.join("missing"),
            &std::cell::RefCell::default(),
            |_| true
        )
        .is_err());
    }

    #[test]
    fn invalid_mapping_text_is_rejected_before_glfw() -> std::io::Result<()> {
        let path =
            std::env::temp_dir().join(format!("vmnl-invalid-mapping-{}.txt", std::process::id()));
        for text in ["bad\0mapping", "non-ASCII: é"] {
            std::fs::write(&path, text)?;
            let mut called = false;
            let result =
                super::load_gamepad_mappings(&path, &std::cell::RefCell::default(), |_| {
                    called = true;
                    true
                });
            assert!(result.is_err());
            assert!(!called);
        }
        std::fs::remove_file(path)?;
        Ok(())
    }
    #[test]
    fn mapping_capture_records_only_while_active() {
        let mut capture = super::MappingErrorCapture::default();

        // Recording is initially disabled.
        capture.record(glfw::Error::InvalidValue, "old error");
        assert!(capture.error.is_none());

        capture.begin();
        capture.record(glfw::Error::InvalidValue, "broken mapping");

        let error = capture.finish();

        assert_eq!(
            error,
            Some((glfw::Error::InvalidValue, "broken mapping".to_owned()))
        );
        assert!(!capture.active);
        assert!(capture.error.is_none());

        // A new operation must not inherit the previous error.
        capture.begin();
        assert!(capture.finish().is_none());
    }
}
