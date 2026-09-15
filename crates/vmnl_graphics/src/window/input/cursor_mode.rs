// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Cursor visibility and confinement modes.

/// Controls how the native cursor behaves over a window.
///
/// The default is [`Normal`](Self::Normal). GLFW stores the selected mode per window, but disabled
/// or captured confinement becomes effective only while that window is focused. Captured mode is
/// unavailable in GLFW 3.4 on Cocoa and Wayland confinement depends on compositor protocols.
/// Backend errors are reported through
/// [`Window::set_error_callback`](crate::Window::set_error_callback).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CursorMode {
    /// Shows the cursor and lets it move normally.
    #[default]
    Normal,
    /// Hides the cursor while it is over the window content area.
    Hidden,
    /// Hides and grabs the cursor while reporting virtual, unbounded motion.
    Disabled,
    /// Shows the cursor and confines it to the window content area.
    Captured,
}

impl CursorMode {
    pub(crate) const fn from_glfw(mode: glfw::CursorMode) -> Self {
        match mode {
            glfw::CursorMode::Normal => Self::Normal,
            glfw::CursorMode::Hidden => Self::Hidden,
            glfw::CursorMode::Disabled => Self::Disabled,
            glfw::CursorMode::Captured => Self::Captured,
        }
    }

    pub(crate) const fn to_glfw(self) -> glfw::CursorMode {
        match self {
            Self::Normal => glfw::CursorMode::Normal,
            Self::Hidden => glfw::CursorMode::Hidden,
            Self::Disabled => glfw::CursorMode::Disabled,
            Self::Captured => glfw::CursorMode::Captured,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CursorMode;

    #[test]
    fn default_cursor_mode_is_normal() {
        assert_eq!(CursorMode::default(), CursorMode::Normal);
    }

    #[test]
    fn every_cursor_mode_round_trips_through_glfw() {
        for mode in [
            CursorMode::Normal,
            CursorMode::Hidden,
            CursorMode::Disabled,
            CursorMode::Captured,
        ] {
            assert_eq!(CursorMode::from_glfw(mode.to_glfw()), mode);
        }
    }
}
