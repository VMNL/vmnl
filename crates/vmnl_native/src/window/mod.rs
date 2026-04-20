////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Window module of the VMNL library, encapsulating window management and rendering logic.
/// This module defines the `Window` struct, which serves as the primary interface for
/// creating and managing application windows, handling events, and coordinating rendering.
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;
use crate::{
    Graphics,
    Context,
    VMNLError,
    VMNLResult,
    VMNLErrorKind,
    window::inner::VMNLWindow
};
pub mod api;
mod inner;
pub mod handle;
pub mod config;
pub mod state;
pub mod input;
pub mod render;
pub mod shaders;
pub mod event;
pub mod monitors;
pub use event::{
    EventQueue,
    Event
};
pub use input::{
    Input,
    Key,
    MouseButton,
    KeyboardState,
    MouseState
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstants
{
    /// Current size of the window used for scaling and coordinate transformations in shaders.
    window_size: [f32; 2],
}

pub struct WindowBuilder
{
    options: WindowOptions,
}

#[derive(Clone, Debug)]
pub struct WindowOptions
{
    title: String,
    width: u32,
    height: u32
}

impl Default for WindowOptions
{
    fn default() -> Self
    {
        Self {
            title: "VMNL Window".into(),
            width: 800,
            height: 600
        }
    }
}

impl Default for WindowBuilder
{
    fn default() -> Self
    {
        Self {
            options: WindowOptions::default()
        }
    }
}

impl WindowBuilder
{
    pub fn title(mut self, title: &str) -> Self
    {
        self.options.title = title.to_string();
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self
    {
        self.options.width = width;
        self.options.height = height;
        self
    }

    pub fn build(self, context: &Context) -> VMNLResult<Window>
    {
        Window::from_options(context, self.options)
    }
}

pub struct Window
{
    inner: VMNLWindow
}

impl Window
{
    pub fn new(context: &Context) -> VMNLResult<Self>
    {
        Self::builder().build(context)
    }

    pub fn builder() -> WindowBuilder
    {
        WindowBuilder::default()
    }

    fn from_options(context: &Context, options: WindowOptions) -> VMNLResult<Self>
    {
        if options.width < 64 || options.height < 64 {
            return Err(VMNLError::new(VMNLErrorKind::InvalidWindowSize));
        }

        Ok(
            Self {
                inner: VMNLWindow::create(context, options)?
            }
        )
    }
}
