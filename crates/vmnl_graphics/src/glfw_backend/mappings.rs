// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Checked GLFW mapping operations, shared with the isolated platform probe.

use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::{cell::RefCell, path::Path, rc::Rc};

#[derive(Debug, Default)]
pub(crate) struct MappingErrorCapture {
    pub(super) active: bool,
    pub(super) error: Option<(glfw::Error, String)>,
}

pub(super) struct MappingCaptureScope<'a>(&'a RefCell<MappingErrorCapture>);

impl<'a> MappingCaptureScope<'a> {
    pub(super) fn begin(capture: &'a RefCell<MappingErrorCapture>) -> VMNLResult<Self> {
        let mut state = capture.borrow_mut();
        if state.active {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "gamepad mapping updates cannot be nested inside an error callback".into(),
            )));
        }
        state.begin();
        Ok(Self(capture))
    }
}

impl Drop for MappingCaptureScope<'_> {
    fn drop(&mut self) {
        self.0.borrow_mut().finish();
    }
}

pub(super) fn recording_callback(
    capture: &Rc<RefCell<MappingErrorCapture>>,
    mut callback: impl FnMut(glfw::Error, String) + 'static,
) -> impl FnMut(glfw::Error, String) + 'static {
    let capture = Rc::clone(capture);
    move |error, description| {
        // Release the borrow before running application code, which may re-enter VMNL.
        capture.borrow_mut().record(error, &description);
        callback(error, description);
    }
}

impl MappingErrorCapture {
    pub(super) fn record(&mut self, error: glfw::Error, description: &str) {
        if self.active && self.error.is_none() {
            self.error = Some((error, description.to_owned()));
        }
    }
    pub(super) fn begin(&mut self) {
        self.error = None;
        self.active = true;
    }

    pub(super) fn finish(&mut self) -> Option<(glfw::Error, String)> {
        self.active = false;
        self.error.take()
    }
}

pub(crate) fn load_gamepad_mappings(
    path: &Path,
    capture: &RefCell<MappingErrorCapture>,
    apply: impl FnOnce(&str) -> bool,
) -> VMNLResult<()> {
    let mappings = std::fs::read_to_string(path).map_err(|error| {
        VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "cannot read gamepad mappings from {}: {error}",
            path.display()
        )))
    })?;
    apply_gamepad_mappings(&mappings, capture, apply)
}

pub(crate) fn apply_gamepad_mappings(
    mappings: &str,
    capture: &RefCell<MappingErrorCapture>,
    apply: impl FnOnce(&str) -> bool,
) -> VMNLResult<()> {
    // The GLFW Rust wrapper converts the input to a C string and cannot accept NUL bytes.
    if mappings.contains('\0') || !mappings.is_ascii() {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "gamepad mappings must contain ASCII text without NUL bytes".into(),
        )));
    }
    let _scope = MappingCaptureScope::begin(capture)?;
    let accepted = apply(mappings);
    let error = capture.borrow_mut().finish();
    if let Some((code, description)) = error {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "GLFW rejected gamepad mappings ({}): {description}",
            code.as_raw()
        ))));
    }
    if !accepted {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "GLFW rejected gamepad mappings".into(),
        )));
    }
    Ok(())
}

pub(crate) fn init(
    capture: &Rc<RefCell<MappingErrorCapture>>,
    callback: impl FnMut(glfw::Error, String) + 'static,
) -> Result<glfw::Glfw, glfw::InitError> {
    glfw::init(recording_callback(capture, callback))
}

pub(crate) fn set_error_callback(
    glfw: &mut glfw::Glfw,
    capture: &Rc<RefCell<MappingErrorCapture>>,
    callback: impl FnMut(glfw::Error, String) + 'static,
) {
    glfw.set_error_callback(recording_callback(capture, callback));
}

pub(crate) fn unset_error_callback(
    glfw: &mut glfw::Glfw,
    capture: &Rc<RefCell<MappingErrorCapture>>,
) {
    glfw.set_error_callback(recording_callback(capture, |_, _| {}));
}
