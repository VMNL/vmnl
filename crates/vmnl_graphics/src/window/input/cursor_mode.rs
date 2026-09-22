// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Cursor visibility and confinement modes.

/// Controls how the native cursor behaves over a window.
///
/// The default is [`Normal`](Self::Normal). VMNL stores the selected mode per window. Hidden mode
/// uses a transparent native cursor while GLFW remains in normal mode; disabled or captured
/// confinement becomes effective only while that window is focused. Captured mode is unavailable
/// in GLFW 3.4 on Cocoa and Wayland confinement depends on compositor protocols.
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
    pub(crate) const fn to_effective_glfw(self) -> glfw::CursorMode {
        match self {
            Self::Normal | Self::Hidden => glfw::CursorMode::Normal,
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
    fn cursor_modes_map_to_their_effective_glfw_mode() {
        assert_eq!(
            CursorMode::Normal.to_effective_glfw(),
            glfw::CursorMode::Normal
        );
        assert_eq!(
            CursorMode::Hidden.to_effective_glfw(),
            glfw::CursorMode::Normal
        );
        assert_eq!(
            CursorMode::Disabled.to_effective_glfw(),
            glfw::CursorMode::Disabled
        );
        assert_eq!(
            CursorMode::Captured.to_effective_glfw(),
            glfw::CursorMode::Captured
        );
    }
}
