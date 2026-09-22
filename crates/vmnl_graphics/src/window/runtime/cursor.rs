// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Internal cursor position and input-mode operations.

use crate::{Cursor, CursorMode, VMNLError, VMNLErrorKind, VMNLResult};

use super::super::inner::VMNLWindow;

impl VMNLWindow {
    pub(crate) fn cursor(&self) -> Option<&Cursor> {
        self.handle.cursor.as_ref()
    }

    pub(crate) fn set_cursor(&mut self, cursor: Option<&Cursor>) -> VMNLResult<()> {
        if self.handle.cursor_mode != CursorMode::Hidden {
            crate::glfw_backend::set_cursor(&mut self.handle.context, cursor.map(Cursor::native))?;
        }
        self.handle.cursor = cursor.cloned();
        Ok(())
    }

    pub(crate) fn get_cursor_position(&self) -> (f64, f64) {
        self.handle.context.get_cursor_pos()
    }

    pub(crate) fn set_cursor_position(&mut self, x: f64, y: f64) -> VMNLResult<()> {
        validate_cursor_position(x, y)?;
        self.handle.context.set_cursor_pos(x, y);
        Ok(())
    }

    pub(crate) fn is_cursor_hovered(&self) -> bool {
        self.handle.context.is_hovered()
    }

    pub(crate) fn get_cursor_mode(&self) -> CursorMode {
        self.handle.cursor_mode
    }

    pub(crate) fn set_cursor_mode(&mut self, mode: CursorMode) -> VMNLResult<()> {
        let previous_mode = self.handle.cursor_mode;
        if mode == previous_mode {
            return Ok(());
        }

        if mode == CursorMode::Hidden {
            self.apply_hidden_cursor()?;
            if let Err(error) = self.apply_native_cursor_mode(mode) {
                self.restore_native_cursor_state(previous_mode);
                return Err(error);
            }
        } else if previous_mode == CursorMode::Hidden {
            if let Err(error) = self.apply_native_cursor_mode(mode) {
                self.restore_native_cursor_state(previous_mode);
                return Err(error);
            }
            if let Err(error) = self.apply_client_cursor() {
                self.restore_native_cursor_state(previous_mode);
                return Err(error);
            }
        } else if let Err(error) = self.apply_native_cursor_mode(mode) {
            let _ = self.apply_native_cursor_mode(previous_mode);
            return Err(error);
        }

        self.handle.cursor_mode = mode;
        Ok(())
    }

    fn apply_native_cursor_mode(&mut self, mode: CursorMode) -> VMNLResult<()> {
        crate::glfw_backend::set_cursor_mode(&mut self.handle.context, mode.to_effective_glfw())
    }

    fn apply_client_cursor(&mut self) -> VMNLResult<()> {
        let cursor = self.handle.cursor.clone();
        crate::glfw_backend::set_cursor(
            &mut self.handle.context,
            cursor.as_ref().map(Cursor::native),
        )
    }

    fn apply_hidden_cursor(&mut self) -> VMNLResult<()> {
        // A non-animated cursor prevents GLFW's Wayland theme timer from restoring a visible
        // frame after a hidden-mode request and keeps VMNL's hidden behavior backend-independent.
        let cursor = if let Some(cursor) = &self.handle.hidden_cursor {
            cursor.clone()
        } else {
            let cursor = Cursor::transparent(&self.handle.instance)?;
            self.handle.hidden_cursor = Some(cursor.clone());
            cursor
        };

        crate::glfw_backend::set_cursor(&mut self.handle.context, Some(cursor.native()))
    }

    fn restore_native_cursor_state(&mut self, mode: CursorMode) {
        let _ = self.apply_native_cursor_mode(mode);
        if mode == CursorMode::Hidden {
            let _ = self.apply_hidden_cursor();
        } else {
            let _ = self.apply_client_cursor();
        }
    }

    pub(crate) fn is_sticky_mouse_buttons_enabled(&self) -> bool {
        self.handle.context.has_sticky_mouse_buttons()
    }

    pub(crate) fn set_sticky_mouse_buttons(&mut self, enabled: bool) {
        self.handle.context.set_sticky_mouse_buttons(enabled);
    }

    pub(crate) fn is_sticky_keys_enabled(&self) -> bool {
        self.handle.context.has_sticky_keys()
    }

    pub(crate) fn set_sticky_keys(&mut self, enabled: bool) {
        self.handle.context.set_sticky_keys(enabled);
    }

    pub(crate) fn is_lock_key_modifier_reporting_enabled(&self) -> bool {
        self.handle.context.does_store_lock_key_mods()
    }

    pub(crate) fn set_lock_key_modifier_reporting(&mut self, enabled: bool) {
        self.handle.context.set_store_lock_key_mods(enabled);
    }

    pub(crate) fn is_raw_mouse_motion_enabled(&self) -> bool {
        self.handle.context.uses_raw_mouse_motion()
    }

    pub(crate) fn set_raw_mouse_motion(&mut self, enabled: bool) -> VMNLResult<()> {
        let is_supported = self.handle.instance.supports_raw_motion();
        validate_raw_mouse_motion_request(is_supported, enabled)?;

        if is_supported {
            self.handle.context.set_raw_mouse_motion(enabled);
        }
        Ok(())
    }
}

fn validate_cursor_position(x: f64, y: f64) -> VMNLResult<()> {
    if !x.is_finite() || !y.is_finite() {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "cursor coordinates must be finite".into(),
        )));
    }

    Ok(())
}

fn validate_raw_mouse_motion_request(is_supported: bool, enabled: bool) -> VMNLResult<()> {
    if enabled && !is_supported {
        return Err(VMNLError::new(VMNLErrorKind::GlfwUnsupportedPlatform));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_cursor_position, validate_raw_mouse_motion_request};
    use crate::VMNLErrorKind;

    #[test]
    fn cursor_position_accepts_finite_fractional_and_negative_coordinates() {
        assert!(validate_cursor_position(-0.5, 42.25).is_ok());
        assert!(validate_cursor_position(f64::MAX, -f64::MAX).is_ok());
    }

    #[test]
    fn cursor_position_rejects_non_finite_coordinates() {
        for coordinates in [
            (f64::NAN, 0.0),
            (0.0, f64::NAN),
            (f64::INFINITY, 0.0),
            (0.0, f64::NEG_INFINITY),
        ] {
            assert!(matches!(
                validate_cursor_position(coordinates.0, coordinates.1),
                Err(error)
                    if matches!(
                        error.kind(),
                        VMNLErrorKind::InvalidState(message)
                            if message == "cursor coordinates must be finite"
                    )
            ));
        }
    }

    #[test]
    fn raw_mouse_motion_rejects_only_unsupported_enable_requests() {
        assert!(validate_raw_mouse_motion_request(true, true).is_ok());
        assert!(validate_raw_mouse_motion_request(true, false).is_ok());
        assert!(validate_raw_mouse_motion_request(false, false).is_ok());
        assert!(matches!(
            validate_raw_mouse_motion_request(false, true),
            Err(error) if matches!(error.kind(), VMNLErrorKind::GlfwUnsupportedPlatform)
        ));
    }
}
