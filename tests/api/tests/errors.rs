// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use vmnl::{VMNLError, VMNLErrorKind, VMNLResult, Window};

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
fn window_builder_rejects_inverted_size_limits() -> VMNLResult<()> {
    let result = Window::builder()
        .size_limit(Some(1024), Some(768), Some(800), Some(600))
        .map(|_| ());

    match result {
        Err(error) => {
            assert!(matches!(error.kind(), VMNLErrorKind::InvalidWindowSize));
            Ok(())
        }
        Ok(()) => Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "size_limit unexpectedly accepted min > max".to_string(),
        ))),
    }
}
