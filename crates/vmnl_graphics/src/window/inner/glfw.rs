////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// GLFW window creation helpers for the internal window backend.
////////////////////////////////////////////////////////////////////////////////
use super::VMNLWindow;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};

impl VMNLWindow {
    /// Initialize a GLFW window along with its associated event receiver.
    ///
    /// # Source
    /// <https://www.glfw.org/docs/latest/window_guide.html>
    pub(super) fn init_window(
        mut instance: ::glfw::Glfw,
        width: u32,
        height: u32,
        title: &str,
    ) -> VMNLResult<(
        ::glfw::PWindow,
        ::glfw::GlfwReceiver<(f64, ::glfw::WindowEvent)>,
    )> {
        log::debug!("creating GLFW window \"{title}\" ({width}x{height})");
        instance
            .create_window(width, height, title, ::glfw::WindowMode::Windowed)
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::GlfwWindowCreationFailed))
    }
}
