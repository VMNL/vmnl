////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Window runtime module for internal window state and low-level handles.
///
/// This module groups the runtime components owned by `VMNLWindow`: static
/// configuration, transient state, and GLFW/Vulkan handles.
////////////////////////////////////////////////////////////////////////////////
mod config;
mod handle;
mod state;

pub(crate) use config::WindowConfig;
pub(crate) use handle::WindowHandle;
pub(crate) use state::WindowState;
