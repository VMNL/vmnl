// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public context submodule for the VMNL graphics API.
//!
//! This module exposes the `Context` wrapper around the internal Vulkan
//! instance state.

use super::VMNLInstance;
use crate::{Key, KeyboardState, Scancode, VMNLResult};
use std::rc::Rc;

fn wayland_scancode_name(
    scancode: Scancode,
    mut scancode_for_key: impl FnMut(Key) -> Option<Scancode>,
    mut name_for_key: impl FnMut(Key) -> Option<String>,
    mut name_for_scancode: impl FnMut(Scancode) -> Option<String>,
) -> Option<String> {
    // The bundled GLFW Wayland backend emits an error for a non-printable scancode instead of
    // returning null. Its named-key path returns null without an error for the same physical key.
    if !(0..=255).contains(&scancode.as_raw()) {
        return None;
    }

    if let Some(key) = KeyboardState::named_keys()
        .iter()
        .copied()
        .find(|&key| scancode_for_key(key) == Some(scancode))
    {
        return name_for_key(key);
    }

    name_for_scancode(scancode)
}

/// `Context` is the main struct of the VMNL library, representing the core Vulkan context.
///
/// It is responsible for initializing and managing the Vulkan resources required for rendering operations
/// and provides a high-level interface for the graphical part of the library.
///
/// Device and queue selection are automatic. VMNL ranks supported physical
/// devices, but does not expose a client override. When multiple candidates
/// have equal rank, the selected device follows backend enumeration order and
/// is therefore not deterministic across equal-ranked devices.
///
/// Clones share the same internal Vulkan state through `Rc`; they do not create
/// another device. Consequently `Context` is intentionally single-threaded.
#[derive(Clone)]
pub struct Context {
    /// Inner `VMNLInstance` containing the Vulkan context and resources.
    /// Wrapped in an `Rc` for shared ownership within a single thread.
    pub(crate) inner: Rc<VMNLInstance>,
}

impl Context {
    /// Returns the active-layout name of a printable named key.
    ///
    /// The returned UTF-8 name is intended for displaying key bindings. It is not text input and
    /// can change when the keyboard layout changes. Non-printable keys and [`Key::Unknown`] return
    /// `None`. On Wayland, the bundled GLFW backend cannot safely query its XKB state before the
    /// first keyboard event; VMNL returns `None` until [`Window::poll_events`](crate::Window::poll_events)
    /// observes one.
    ///
    /// This call allocates an owned [`String`] when GLFW provides a name.
    #[inline]
    #[must_use]
    pub fn get_key_name(&self, key: Key) -> Option<String> {
        if !self.inner.keyboard_name_queries_ready.get() {
            return None;
        }

        let key = KeyboardState::to_glfw(key)?;
        glfw::get_key_name(Some(key), None)
    }

    /// Returns the active-layout name of the printable key mapped to a scancode.
    ///
    /// The returned UTF-8 name is intended for displaying key bindings and may change with the
    /// keyboard layout. Invalid, unmapped, and non-printable scancodes return `None`. On Wayland,
    /// VMNL also returns `None` until [`Window::poll_events`](crate::Window::poll_events) observes
    /// the first keyboard event and proves that the bundled GLFW XKB state is ready.
    /// On Wayland, known scancodes are resolved against the named keys first so non-printable
    /// keys return `None` without triggering a spurious GLFW backend error.
    ///
    /// This call allocates an owned [`String`] when GLFW provides a name.
    #[inline]
    #[must_use]
    pub fn get_scancode_name(&self, scancode: Scancode) -> Option<String> {
        if !self.inner.keyboard_name_queries_ready.get() {
            return None;
        }

        if self.inner.glfw.get_platform() == glfw::Platform::Wayland {
            return wayland_scancode_name(
                scancode,
                |key| self.get_key_scancode(key),
                |key| self.get_key_name(key),
                |scancode| glfw::get_key_name(None, Some(scancode.as_raw())),
            );
        }

        glfw::get_key_name(None, Some(scancode.as_raw()))
    }

    /// Returns the active platform scancode for a named key.
    ///
    /// [`Key::Unknown`] and named keys unsupported by the active platform return `None`. The
    /// returned value is platform-specific and must not be persisted as a portable identifier.
    #[inline]
    #[must_use]
    pub fn get_key_scancode(&self, key: Key) -> Option<Scancode> {
        let key = KeyboardState::to_glfw(key)?;
        glfw::get_key_scancode(Some(key)).map(Scancode::from_raw)
    }

    /// Returns whether raw mouse motion is supported by the active GLFW backend and system.
    ///
    /// This value is stable for the lifetime of GLFW after initialization. Raw motion is
    /// configured per window and only affects motion while its cursor mode is disabled. Bundled
    /// GLFW 3.4 reports it unavailable on Cocoa; X11 requires `XInput` 2.
    #[inline]
    #[must_use]
    pub fn is_raw_mouse_motion_supported(&self) -> bool {
        self.inner.glfw.supports_raw_motion()
    }

    /// Initialize a new `Context` required for using the graphical part of the library.
    ///
    /// # Returns
    /// A `VMNLResult<Self>` containing the initialized `Context` on success.
    ///
    /// # Errors
    /// Returns a `VMNLResult::Err` if any step of the Vulkan initialization process
    /// fails, such as instance creation, physical device selection, or logical device creation.
    ///
    /// This call creates Vulkan instance/device/queue and allocator state. Its
    /// allocation count and initialization latency are not specified.
    ///
    /// # Example
    /// ```rust,no_run
    /// use vmnl_graphics::{Context, RenderMode, Window};
    /// use vmnl_graphics::common::Rgba;
    /// use vmnl_graphics::d2::{Shape, Vector2f};
    ///
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// let context = Context::new()?;
    /// let mut window = Window::builder()
    ///     .title("VMNL")
    ///     .size(800, 600)
    ///     .build(&context)?;
    ///
    /// let triangle = Shape::triangle(
    ///     Vector2f { x: 100.0, y: 100.0 },
    ///     Vector2f { x: 300.0, y: 100.0 },
    ///     Vector2f { x: 200.0, y: 300.0 },
    /// )
    /// .vertex_colors(
    ///     Rgba::new(255, 0, 0, 255),
    ///     Rgba::new(0, 255, 0, 255),
    ///     Rgba::new(0, 0, 255, 255),
    /// )
    /// .build(&context)?;
    ///
    /// while window.is_open() {
    ///     for event in window.poll_events() {
    ///         println!("{event:?}");
    ///     }
    ///     window.render()
    ///         .mode(RenderMode::PerObject)
    ///         .draw2d([&triangle])
    ///         .submit()?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> VMNLResult<Self> {
        Ok(Self {
            inner: Rc::new(VMNLInstance::new()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn wayland_non_printable_scancode_avoids_raw_name_lookup() {
        let raw_called = Cell::new(false);
        let name = wayland_scancode_name(
            Scancode::from_raw(29),
            |key| (key == Key::LeftControl).then_some(Scancode::from_raw(29)),
            |key| {
                assert_eq!(key, Key::LeftControl);
                None
            },
            |_| {
                raw_called.set(true);
                None
            },
        );
        assert_eq!(name, None);
        assert!(!raw_called.get());
    }

    #[test]
    fn wayland_printable_scancode_uses_named_key_and_unknown_falls_back() {
        let unexpected_call = Cell::new(false);
        let mapped = wayland_scancode_name(
            Scancode::from_raw(30),
            |key| (key == Key::A).then_some(Scancode::from_raw(30)),
            |key| {
                assert_eq!(key, Key::A);
                Some("a".to_owned())
            },
            |_| {
                unexpected_call.set(true);
                None
            },
        );
        assert_eq!(mapped.as_deref(), Some("a"));
        assert!(!unexpected_call.get());

        let unknown = wayland_scancode_name(
            Scancode::from_raw(254),
            |_| None,
            |_| {
                unexpected_call.set(true);
                None
            },
            |scancode| {
                assert_eq!(scancode.as_raw(), 254);
                Some("é".to_owned())
            },
        );
        assert_eq!(unknown.as_deref(), Some("é"));
        assert!(!unexpected_call.get());
    }

    #[test]
    fn wayland_invalid_scancode_never_calls_glfw() {
        let called = Cell::new(false);
        for raw in [-1, 256] {
            assert_eq!(
                wayland_scancode_name(
                    Scancode::from_raw(raw),
                    |_| {
                        called.set(true);
                        None
                    },
                    |_| {
                        called.set(true);
                        None
                    },
                    |_| {
                        called.set(true);
                        None
                    },
                ),
                None
            );
        }
        assert!(!called.get());
    }
}
