// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public error and builder validation contracts.

use vmnl::{VMNLError, VMNLErrorKind, VMNLResult, Window};

type SetAspectRatio = fn(&mut Window, Option<(u32, u32)>) -> VMNLResult<()>;

#[test]
fn vmnl_error_exposes_kind_location_and_report() -> VMNLResult<()> {
    let error = VMNLError::new(VMNLErrorKind::InvalidState("api error".to_string()));
    let location = error.location();
    let report = error.report();

    assert!(matches!(
        error.kind(),
        VMNLErrorKind::InvalidState(message) if message == "api error"
    ));
    assert!(location.file().ends_with("errors.rs"));
    assert!(location.line() > 0);
    assert!(location.column() > 0);
    assert!(report.contains("invalid state: api error"));
    assert!(report.contains("errors.rs"));

    Ok(())
}

#[test]
fn window_builder_size_limits_cover_valid_boundaries_and_each_inversion() -> VMNLResult<()> {
    for result in [
        Window::builder().size_limit(None, None, None, None),
        Window::builder().size_limit(Some(64), None, None, None),
        Window::builder().size_limit(None, Some(64), None, None),
        Window::builder().size_limit(Some(64), Some(64), Some(64), Some(64)),
    ] {
        assert!(result.is_ok());
    }

    for result in [
        Window::builder().size_limit(Some(1024), None, Some(800), None),
        Window::builder().size_limit(None, Some(768), None, Some(600)),
    ] {
        assert!(matches!(
            result,
            Err(error) if matches!(error.kind(), VMNLErrorKind::InvalidWindowSize)
        ));
    }

    Ok(())
}

#[test]
fn window_aspect_ratio_returns_a_public_result() {
    let _: SetAspectRatio = Window::set_aspect_ratio;
}
