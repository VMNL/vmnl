// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public cursor position and input-mode API.

use crate::{Cursor, CursorMode, VMNLResult, Window};

impl Window {
    /// Returns the custom or standard cursor currently assigned to this window.
    ///
    /// `None` means that the backend default cursor is used. The returned borrow remains owned by
    /// the window and compares equal to clones of the resource passed to [`set_cursor`](Self::set_cursor).
    #[inline]
    #[must_use]
    pub fn cursor(&self) -> Option<&Cursor> {
        self.inner.cursor()
    }

    /// Assigns a custom or standard cursor, or restores the backend default with `None`.
    ///
    /// The window retains a shared clone of the resource, so the caller may drop its own clone or
    /// assign the same cursor to other windows. The selected image is visible only in
    /// [`CursorMode::Normal`] and may additionally require window focus on some platforms.
    /// This call performs no VMNL allocation and preserves the previous cursor if GLFW rejects the
    /// request.
    ///
    /// # Errors
    /// Returns the VMNL category produced by this `glfwSetCursor` call.
    #[inline]
    pub fn set_cursor(&mut self, cursor: Option<&Cursor>) -> VMNLResult<()> {
        self.inner.set_cursor(cursor)
    }

    /// Returns the cursor position relative to the upper-left corner of the content area.
    ///
    /// X increases to the right and Y increases downward. Coordinates are `f64` screen
    /// coordinates, not framebuffer pixels, and can be fractional or negative. A native cursor
    /// backend may quantize physical positions. In [`CursorMode::Disabled`], the returned virtual
    /// position is unbounded and retains GLFW's `f64` precision.
    #[inline]
    #[must_use]
    pub fn get_cursor_position(&self) -> (f64, f64) {
        self.inner.get_cursor_position()
    }

    /// Sets the cursor position relative to the upper-left corner of the content area.
    ///
    /// The request has no effect while the window is unfocused. Wayland only supports updating
    /// the virtual position in [`CursorMode::Disabled`]; other modes report the limitation
    /// through the configured GLFW error callback.
    ///
    /// # Errors
    /// Returns [`VMNLErrorKind::InvalidState`](crate::VMNLErrorKind::InvalidState) when either
    /// coordinate is not finite.
    #[inline]
    pub fn set_cursor_position(&mut self, x: f64, y: f64) -> VMNLResult<()> {
        self.inner.set_cursor_position(x, y)
    }

    /// Returns whether the cursor is currently over the window content area.
    ///
    /// On platforms that cannot provide the attribute, GLFW reports an error and returns
    /// `false`.
    #[inline]
    #[must_use]
    pub fn is_cursor_hovered(&self) -> bool {
        self.inner.is_cursor_hovered()
    }

    /// Returns the cursor mode stored for this window.
    ///
    /// The stored mode can differ from effective native behavior while the window is unfocused
    /// or when a backend cannot implement the requested mode.
    #[inline]
    #[must_use]
    pub fn get_cursor_mode(&self) -> CursorMode {
        self.inner.get_cursor_mode()
    }

    /// Sets the cursor visibility and confinement mode.
    ///
    /// Disabled and captured modes become effective only while the window is focused. Captured
    /// mode is not implemented by GLFW 3.4 on Cocoa. Backend failures are reported through the
    /// configured GLFW error callback; [`get_cursor_mode`](Self::get_cursor_mode) returns GLFW's
    /// stored mode.
    #[inline]
    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.inner.set_cursor_mode(mode);
    }

    /// Returns whether GLFW sticky mouse buttons are enabled for this window.
    ///
    /// This mode affects consuming `glfwGetMouseButton` reads. VMNL does not use those reads for
    /// [`MouseState`](crate::MouseState), so querying VMNL snapshots never consumes a sticky
    /// press or changes batch-transition semantics.
    #[inline]
    #[must_use]
    pub fn is_sticky_mouse_buttons_enabled(&self) -> bool {
        self.inner.is_sticky_mouse_buttons_enabled()
    }

    /// Enables or disables GLFW sticky mouse buttons for this window.
    ///
    /// Enabling the mode latches a native press until the next consuming `glfwGetMouseButton`
    /// read. VMNL's event-derived [`MouseState`](crate::MouseState) remains non-consuming and
    /// continues to report final state plus every transition in the current event batch.
    #[inline]
    pub fn set_sticky_mouse_buttons(&mut self, enabled: bool) {
        self.inner.set_sticky_mouse_buttons(enabled);
    }

    /// Returns whether lock-key modifier reporting is enabled for this window.
    ///
    /// When enabled, mouse-button events can include
    /// [`Modifiers::CAPS_LOCK`](crate::Modifiers::CAPS_LOCK) and
    /// [`Modifiers::NUM_LOCK`](crate::Modifiers::NUM_LOCK).
    #[inline]
    #[must_use]
    pub fn is_lock_key_modifier_reporting_enabled(&self) -> bool {
        self.inner.is_lock_key_modifier_reporting_enabled()
    }

    /// Enables or disables lock-key modifier reporting for this window.
    ///
    /// Enabling this mode asks GLFW to include Caps Lock and Num Lock state in modifier payloads
    /// delivered with input callbacks. VMNL preserves those bits in mouse-button events.
    #[inline]
    pub fn set_lock_key_modifier_reporting(&mut self, enabled: bool) {
        self.inner.set_lock_key_modifier_reporting(enabled);
    }

    /// Returns whether raw mouse motion is configured for this window.
    ///
    /// A `true` value records the option; raw deltas are delivered only while the cursor mode is
    /// [`CursorMode::Disabled`].
    #[inline]
    #[must_use]
    pub fn is_raw_mouse_motion_enabled(&self) -> bool {
        self.inner.is_raw_mouse_motion_enabled()
    }

    /// Enables or disables raw, unscaled and unaccelerated mouse motion for this window.
    ///
    /// The option can be configured independently of the cursor mode, but only affects cursor
    /// motion while [`CursorMode::Disabled`] is effective.
    ///
    /// # Errors
    /// Returns [`GlfwUnsupportedPlatform`](crate::VMNLErrorKind::GlfwUnsupportedPlatform) when
    /// enabling raw motion on a system where
    /// [`Context::is_raw_mouse_motion_supported`](crate::Context::is_raw_mouse_motion_supported)
    /// is `false`. Disabling remains a successful no-op on such systems.
    #[inline]
    pub fn set_raw_mouse_motion(&mut self, enabled: bool) -> VMNLResult<()> {
        self.inner.set_raw_mouse_motion(enabled)
    }
}
