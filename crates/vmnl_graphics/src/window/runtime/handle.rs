// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Handle for the window, encapsulating both GLFW and Vulkan resources related
//! to window management and rendering.

use crate::{
    glfw_runtime::{JoystickEventQueue, JoystickEventSubscription},
    vmnl_instance::VMNLInstance,
    window::inner::VMNLWindow,
    window::{event::EventQueue, input::ALL_JOYSTICK_IDS},
    Cursor, CursorMode, Event, EventKind, Input, VMNLErrorKind,
};
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};
use vulkano::{
    pipeline::GraphicsPipeline,
    render_pass::{Framebuffer, RenderPass},
    swapchain::Swapchain,
    sync::GpuFuture,
};

/// Encapsulates low-level resources required to manage a window and its associated rendering state.
///
/// Groups together GLFW windowing objects and Vulkan rendering resources tied to the window.
/// Acts as the bridge between platform-specific window handling and GPU-side rendering execution.
///
/// # Sources
/// - Vulkan synchronization: <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/chap7.html>
/// - Vulkano futures: <https://docs.rs/vulkano/latest/vulkano/sync>/
/// - GLFW windowing: <https://www.glfw.org/docs/latest/window_guide.html>
/// - glfw-rs: <https://github.com/PistonDevelopers/glfw-rs>
pub(crate) struct WindowHandle {
    /// List of framebuffers associated with the swapchain images.
    pub(crate) framebuffers: Vec<Arc<Framebuffer>>,
    /// Render pass shared by the window pipelines and framebuffers.
    pub(crate) render_pass: Arc<RenderPass>,
    /// Preconfigured opaque 2D Vulkan graphics pipeline used to render into the framebuffer.
    pub(crate) pipeline_2d_opaque: Arc<GraphicsPipeline>,
    /// Preconfigured alpha-blended 2D Vulkan graphics pipeline used to render into the framebuffer.
    pub(crate) pipeline_2d_alpha: Arc<GraphicsPipeline>,
    /// Synchronization primitive representing the completion of the previous frame.
    pub(crate) previous_frame_end: Option<Box<dyn GpuFuture>>,
    /// Vulkan surface representing the OS window for presentation.
    pub(crate) swapchain: Arc<Swapchain>,
    /// GLFW context responsible for managing windowing and event polling.
    pub(crate) instance: glfw::Glfw,
    /// Handle to the actual OS window (GLFW window).
    pub(crate) context: glfw::PWindow,
    /// Native cursor retained while assigned to this window.
    pub(crate) cursor: Option<Cursor>,
    /// Transparent cursor allocated lazily for VMNL's logical hidden mode.
    pub(crate) hidden_cursor: Option<Cursor>,
    /// Cursor mode requested through VMNL; hidden maps to GLFW normal plus `hidden_cursor`.
    pub(crate) cursor_mode: CursorMode,
    /// Event receiver channel used to retrieve window events.
    pub(crate) events: EventQueue,
    /// Input state manager for keyboard and mouse events.
    pub(crate) input: Input,
    /// Per-window queue for joystick connection events when delivery is enabled.
    pub(crate) joystick_events: Rc<RefCell<JoystickEventQueue>>,
    pub(crate) joystick_event_subscription: Option<JoystickEventSubscription>,
    /// Whether this window samples the sixteen joystick slots during its own event poll.
    pub(crate) joystick_tracking_enabled: bool,
    /// Whether the first successful full slot sweep has established the transition baseline.
    pub(crate) joystick_snapshot_initialized: bool,
    /// Reference to the Vulkan context and shared GLFW runtime. Kept last so the native window
    /// and its GLFW token are destroyed before the runtime can terminate GLFW.
    pub(crate) vmnl_instance: Rc<VMNLInstance>,
}

impl VMNLWindow {
    /// Internal implementation backing `Window::close`.
    pub(crate) fn close(&mut self) {
        log::debug!("closing window \"{}\"", self.config.title);
        self.handle.context.set_should_close(true);
    }

    /// Internal implementation backing `Window::poll_events`.
    pub(crate) fn poll_events(&mut self) -> Vec<Event> {
        self.handle.instance.poll_events();
        self.handle
            .vmnl_instance
            .glfw
            .resume_joystick_callback_panic();
        let mut events: Vec<Event> = self.handle.events.poll_events(&mut self.handle.input);
        update_joystick_snapshots(
            self.handle.joystick_tracking_enabled,
            &mut self.handle.joystick_snapshot_initialized,
            &mut self.handle.input,
            |id| self.handle.vmnl_instance.glfw.sample_joystick(id),
        );
        events.extend(self.handle.joystick_events.borrow_mut().drain_events());
        events.sort_by(|left, right| {
            left.timestamp_seconds()
                .total_cmp(&right.timestamp_seconds())
        });
        if self.handle.input.keyboard().is_one_used() {
            self.handle
                .vmnl_instance
                .keyboard_name_queries_ready
                .set(true);
        }
        if events.iter().any(|event| {
            matches!(
                event.kind(),
                EventKind::Resized { .. } | EventKind::FramebufferResized { .. }
            )
        }) {
            self.state.swapchain_recreation_requested = true;
        }
        events
    }

    /// Internal implementation backing `Window::input`.
    #[inline]
    pub(crate) const fn input(&self) -> &Input {
        &self.handle.input
    }

    /// Internal implementation backing `Window::clear_input_transitions`.
    pub(crate) const fn clear_input_transitions(&mut self) {
        self.handle.input.clear_transitions();
    }

    pub(crate) fn set_joystick_event_delivery(&mut self, enabled: bool) {
        if enabled == self.handle.joystick_event_subscription.is_some() {
            return;
        }

        if enabled {
            self.handle.joystick_event_subscription = Some(
                self.handle
                    .vmnl_instance
                    .glfw
                    .subscribe_joystick_events(Rc::clone(&self.handle.joystick_events)),
            );
        } else {
            self.handle.joystick_event_subscription = None;
            self.handle.joystick_events.borrow_mut().clear();
        }
    }

    pub(crate) const fn is_joystick_event_delivery_enabled(&self) -> bool {
        self.handle.joystick_event_subscription.is_some()
    }

    pub(crate) fn set_joystick_tracking(&mut self, enabled: bool) -> crate::VMNLResult<()> {
        if enabled == self.handle.joystick_tracking_enabled {
            return Ok(());
        }

        if enabled {
            // Explicitly enabling tracking initializes GLFW's lazy joystick-query subsystem.
            // State is published later, in this window's next `poll_events` batch.
            self.handle
                .vmnl_instance
                .glfw
                .sample_joystick(crate::JoystickId::Joystick1)?;
            self.handle.joystick_tracking_enabled = true;
            self.handle.joystick_snapshot_initialized = false;
        } else {
            self.handle.joystick_tracking_enabled = false;
            self.handle.joystick_snapshot_initialized = false;
            self.handle.input.clear_joystick_tracking();
        }

        Ok(())
    }

    pub(crate) const fn is_joystick_tracking_enabled(&self) -> bool {
        self.handle.joystick_tracking_enabled
    }

    /// Internal implementation backing `Window::wait_events`.
    pub(crate) fn wait_events(&mut self) {
        self.handle.instance.wait_events();
        self.handle
            .vmnl_instance
            .glfw
            .resume_joystick_callback_panic();
    }

    /// Internal implementation backing `Window::wait_events_timeout`.
    pub(crate) fn wait_events_timeout(&mut self, timeout: f64) {
        self.handle.instance.wait_events_timeout(timeout);
        self.handle
            .vmnl_instance
            .glfw
            .resume_joystick_callback_panic();
    }

    /// Internal implementation backing `Window::post_empty_event`.
    pub(crate) fn post_empty_event(&mut self) {
        self.handle.instance.post_empty_event();
    }

    /// Internal implementation backing `Window::get_time`.
    pub(crate) fn get_time(&mut self) -> f64 {
        self.handle.instance.get_time()
    }

    /// Internal implementation backing `Window::set_time`.
    pub(crate) fn set_time(&mut self, time: f64) {
        self.handle.instance.set_time(time);
    }

    /// Internal implementation backing `Window::get_timer_value`.
    pub(crate) fn get_timer_value(&self) -> u64 {
        self.handle.instance.get_timer_value()
    }

    /// Internal implementation backing `Window::get_timer_frequency`.
    pub(crate) fn get_timer_frequency(&self) -> u64 {
        self.handle.instance.get_timer_frequency()
    }

    /// Internal implementation backing `Window::set_error_callback`.
    pub(crate) fn set_error_callback(
        &mut self,
        callback: impl FnMut(VMNLErrorKind, String) + 'static,
    ) {
        self.handle
            .vmnl_instance
            .glfw
            .set_error_callback(Some(Box::new(callback)));
    }

    /// Internal implementation backing `Window::unset_error_callback`.
    pub(crate) fn unset_error_callback(&mut self) {
        self.handle.vmnl_instance.glfw.set_error_callback(None);
    }

    /// Internal implementation backing `Window::set_char_polling`.
    pub(crate) fn set_char_polling(&mut self, enabled: bool) {
        self.handle.context.set_char_polling(enabled);
        self.handle.events.set_char_delivery(enabled);
    }

    pub(crate) const fn is_char_polling_enabled(&self) -> bool {
        self.handle.events.is_char_delivery_enabled()
    }

    /// Internal implementation backing `Window::set_mouse_button_polling`.
    pub(crate) fn set_mouse_button_polling(&mut self, enabled: bool) {
        self.handle.events.set_mouse_button_delivery(enabled);
    }

    pub(crate) const fn is_mouse_button_polling_enabled(&self) -> bool {
        self.handle.events.is_mouse_button_delivery_enabled()
    }

    /// Internal implementation backing `Window::set_cursor_pos_polling`.
    pub(crate) fn set_cursor_pos_polling(&mut self, enabled: bool) {
        self.handle.events.set_cursor_pos_delivery(enabled);
    }

    pub(crate) const fn is_cursor_pos_polling_enabled(&self) -> bool {
        self.handle.events.is_cursor_pos_delivery_enabled()
    }

    /// Internal implementation backing `Window::set_cursor_enter_polling`.
    pub(crate) fn set_cursor_enter_polling(&mut self, enabled: bool) {
        self.handle.events.set_cursor_enter_delivery(enabled);
    }

    pub(crate) const fn is_cursor_enter_polling_enabled(&self) -> bool {
        self.handle.events.is_cursor_enter_delivery_enabled()
    }

    /// Internal implementation backing `Window::set_scroll_polling`.
    pub(crate) fn set_scroll_polling(&mut self, enabled: bool) {
        self.handle.events.set_scroll_delivery(enabled);
    }

    pub(crate) const fn is_scroll_polling_enabled(&self) -> bool {
        self.handle.events.is_scroll_delivery_enabled()
    }

    /// Internal implementation backing `Window::set_size_polling`.
    pub(crate) fn set_size_polling(&mut self, enabled: bool) {
        self.handle.context.set_size_polling(enabled);
    }

    /// Internal implementation backing `Window::set_framebuffer_size_polling`.
    pub(crate) fn set_framebuffer_size_polling(&mut self, enabled: bool) {
        self.handle.context.set_framebuffer_size_polling(enabled);
    }

    /// Internal implementation backing `Window::set_focus_polling`.
    pub(crate) fn set_focus_polling(&mut self, enabled: bool) {
        self.handle.context.set_focus_polling(enabled);
    }

    /// Internal implementation backing `Window::set_close_polling`.
    pub(crate) fn set_close_polling(&mut self, enabled: bool) {
        self.handle.context.set_close_polling(enabled);
    }

    /// Internal implementation backing `Window::set_key_polling`.
    pub(crate) fn set_key_polling(&mut self, enabled: bool) {
        self.handle.events.set_key_delivery(enabled);
    }

    pub(crate) const fn is_key_polling_enabled(&self) -> bool {
        self.handle.events.is_key_delivery_enabled()
    }

    /// Internal implementation backing `Window::set_char_mods_polling`.
    pub(crate) fn set_char_mods_polling(&mut self, enabled: bool) {
        self.handle.context.set_char_mods_polling(enabled);
        self.handle.events.set_char_mods_delivery(enabled);
    }

    pub(crate) const fn is_char_mods_polling_enabled(&self) -> bool {
        self.handle.events.is_char_mods_delivery_enabled()
    }

    /// Internal implementation backing `Window::set_refresh_polling`.
    pub(crate) fn set_refresh_polling(&mut self, enabled: bool) {
        self.handle.context.set_refresh_polling(enabled);
    }

    /// Internal implementation backing `Window::set_iconify_polling`.
    pub(crate) fn set_iconify_polling(&mut self, enabled: bool) {
        self.handle.context.set_iconify_polling(enabled);
    }

    /// Internal implementation backing `Window::set_maximize_polling`.
    pub(crate) fn set_maximize_polling(&mut self, enabled: bool) {
        self.handle.context.set_maximize_polling(enabled);
    }

    /// Internal implementation backing `Window::set_drag_and_drop_polling`.
    pub(crate) fn set_drag_and_drop_polling(&mut self, enabled: bool) {
        self.handle.context.set_drag_and_drop_polling(enabled);
    }

    /// Internal implementation backing `Window::set_content_scale_polling`.
    pub(crate) fn set_content_scale_polling(&mut self, enabled: bool) {
        self.handle.context.set_content_scale_polling(enabled);
    }

    /// Internal implementation backing `Window::enable_keyboard_polling`.
    pub(crate) fn enable_keyboard_polling(&mut self) {
        self.handle.events.set_key_delivery(true);
        self.handle.context.set_char_polling(true);
        self.handle.events.set_char_delivery(true);
        self.handle.context.set_char_mods_polling(true);
        self.handle.events.set_char_mods_delivery(true);
    }

    /// Internal implementation backing `Window::disable_keyboard_polling`.
    pub(crate) fn disable_keyboard_polling(&mut self) {
        self.handle.events.set_key_delivery(false);
        self.handle.context.set_char_polling(false);
        self.handle.events.set_char_delivery(false);
        self.handle.context.set_char_mods_polling(false);
        self.handle.events.set_char_mods_delivery(false);
    }

    /// Internal implementation backing `Window::enable_mouse_polling`.
    pub(crate) fn enable_mouse_polling(&mut self) {
        self.handle.events.set_mouse_button_delivery(true);
        self.handle.events.set_cursor_pos_delivery(true);
        self.handle.events.set_cursor_enter_delivery(true);
        self.handle.events.set_scroll_delivery(true);
    }

    /// Internal implementation backing `Window::disable_mouse_polling`.
    pub(crate) fn disable_mouse_polling(&mut self) {
        self.handle.events.set_mouse_button_delivery(false);
        self.handle.events.set_cursor_pos_delivery(false);
        self.handle.events.set_cursor_enter_delivery(false);
        self.handle.events.set_scroll_delivery(false);
    }

    /// Internal implementation backing `Window::enable_window_state_polling`.
    pub(crate) fn enable_window_state_polling(&mut self) {
        self.handle.context.set_size_polling(true);
        self.handle.context.set_framebuffer_size_polling(true);
        self.handle.context.set_focus_polling(true);
        self.handle.context.set_close_polling(true);
        self.handle.context.set_refresh_polling(true);
        self.handle.context.set_iconify_polling(true);
        self.handle.context.set_maximize_polling(true);
        self.handle.context.set_drag_and_drop_polling(true);
        self.handle.context.set_content_scale_polling(true);
    }

    /// Internal implementation backing `Window::disable_window_state_polling`.
    pub(crate) fn disable_window_state_polling(&mut self) {
        self.handle.context.set_size_polling(false);
        self.handle.context.set_framebuffer_size_polling(false);
        self.handle.context.set_focus_polling(false);
        self.handle.context.set_close_polling(false);
        self.handle.context.set_refresh_polling(false);
        self.handle.context.set_iconify_polling(false);
        self.handle.context.set_maximize_polling(false);
        self.handle.context.set_drag_and_drop_polling(false);
        self.handle.context.set_content_scale_polling(false);
    }

    /// Internal implementation backing `Window::configure_window_polling`.
    pub(crate) fn configure_window_polling(&mut self) {
        self.enable_keyboard_polling();
        self.enable_mouse_polling();
        self.enable_window_state_polling();
    }

    /// Internal implementation backing `Window::unconfigure_window_polling`.
    pub(crate) fn unconfigure_window_polling(&mut self) {
        self.disable_keyboard_polling();
        self.disable_mouse_polling();
        self.disable_window_state_polling();
    }

    /// Internal implementation backing `Window::enable_all_polling`.
    pub(crate) fn enable_all_polling(&mut self) {
        self.handle.context.set_all_polling(true);
        self.handle.events.set_key_delivery(true);
        self.handle.events.set_char_delivery(true);
        self.handle.events.set_char_mods_delivery(true);
        self.handle.events.set_mouse_button_delivery(true);
        self.handle.events.set_cursor_pos_delivery(true);
        self.handle.events.set_cursor_enter_delivery(true);
        self.handle.events.set_scroll_delivery(true);
    }
}

fn update_joystick_snapshots(
    tracking: bool,
    initialized: &mut bool,
    input: &mut Input,
    mut sample: impl FnMut(crate::JoystickId) -> crate::VMNLResult<Option<crate::JoystickSample>>,
) {
    use crate::JoystickId;

    if !tracking {
        return;
    }

    let mut samples: [Option<crate::JoystickSample>; JoystickId::COUNT] =
        std::array::from_fn(|_| None);
    let mut errors = Vec::new();
    for (index, id) in ALL_JOYSTICK_IDS.iter().copied().enumerate() {
        match sample(id) {
            Ok(sample) => samples[index] = sample,
            Err(error) => errors.push(error),
        }
    }

    if !errors.is_empty() {
        for error in errors {
            log::error!("joystick snapshot update failed: {error}");
        }
        return;
    }

    input.apply_joystick_samples(samples, !*initialized);
    *initialized = true;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]

    use super::*;
    use crate::{
        GamepadAxis, GamepadButton, GamepadState, JoystickId, JoystickSample, JoystickStatus,
    };

    fn held_sample(id: JoystickId) -> Option<JoystickSample> {
        (id == JoystickId::Joystick6).then(|| {
            JoystickSample::from_native(
                id,
                Some("snapshot fixture".into()),
                None,
                vec![],
                vec![1],
                vec![],
                Some(GamepadState::from_native(
                    None,
                    [1; GamepadButton::COUNT],
                    [0.0; GamepadAxis::COUNT],
                )),
            )
        })
    }

    #[test]
    fn keyboard_mouse_batches_do_not_sample_joysticks_with_default_options() {
        let options = crate::Window::builder().options;
        let mut input = Input::new();
        let mut initialized = false;
        assert!(!options.joystick_event_delivery);

        for action in [glfw::Action::Press, glfw::Action::Release] {
            input.begin_batch();
            input.apply_event(&glfw::WindowEvent::Key(
                glfw::Key::A,
                0,
                action,
                glfw::Modifiers::empty(),
            ));
            input.apply_event(&glfw::WindowEvent::MouseButton(
                glfw::MouseButtonLeft,
                action,
                glfw::Modifiers::empty(),
            ));
            update_joystick_snapshots(
                options.joystick_tracking,
                &mut initialized,
                &mut input,
                |_| panic!("a keyboard/mouse-only batch must not query any joystick slot"),
            );
            let pressed = action == glfw::Action::Press;
            assert_eq!(input.keyboard().is_down(crate::Key::A), pressed);
            assert_eq!(input.mouse().is_down(crate::MouseButton::Left), pressed);
            assert_eq!(input.keyboard().is_pressed(crate::Key::A), pressed);
            assert_eq!(
                input.mouse().is_released(crate::MouseButton::Left),
                !pressed
            );
        }
        input.begin_batch();
        update_joystick_snapshots(
            options.joystick_tracking,
            &mut initialized,
            &mut input,
            |_| panic!("an empty keyboard/mouse batch must not query any joystick slot"),
        );
        assert!(!initialized);
        assert!(!input.keyboard().is_released(crate::Key::A));
        assert!(!input.mouse().is_released(crate::MouseButton::Left));
        for id in ALL_JOYSTICK_IDS {
            assert_eq!(input.joystick(id).status(), JoystickStatus::NotTracked);
        }
    }

    #[test]
    fn explicit_tracking_samples_each_slot_once_per_batch_and_stops_when_disabled() {
        let mut input = Input::new();
        let mut initialized = false;
        for _ in 0..2 {
            let mut slots = Vec::new();
            update_joystick_snapshots(true, &mut initialized, &mut input, |id| {
                slots.push(id.index());
                Ok(held_sample(id))
            });
            assert_eq!(slots, (0..JoystickId::COUNT).collect::<Vec<_>>());
            assert!(initialized);
            assert!(input
                .joystick(JoystickId::Joystick6)
                .is_down(GamepadButton::A));
            assert!(!input
                .joystick(JoystickId::Joystick6)
                .is_pressed(GamepadButton::A));
        }
        update_joystick_snapshots(false, &mut initialized, &mut input, |_| {
            panic!("disabled tracking must stop subsequent slot queries")
        });
        assert!(initialized);
        assert!(input
            .joystick(JoystickId::Joystick6)
            .is_down(GamepadButton::A));
    }

    #[test]
    fn failed_slot_sweep_preserves_previous_snapshot_and_initialization_state() {
        for previous_initialized in [false, true] {
            let mut input = Input::new();
            let mut initialized = false;
            if previous_initialized {
                update_joystick_snapshots(true, &mut initialized, &mut input, |_| Ok(None));
            }
            let mut slots = Vec::new();
            update_joystick_snapshots(true, &mut initialized, &mut input, |id| {
                slots.push(id.index());
                if id == JoystickId::Joystick8 {
                    Err(crate::VMNLError::new(VMNLErrorKind::InvalidState(
                        "test query failure".into(),
                    )))
                } else {
                    Ok(held_sample(id))
                }
            });
            assert_eq!(slots, (0..JoystickId::COUNT).collect::<Vec<_>>());
            assert_eq!(initialized, previous_initialized);
            for id in ALL_JOYSTICK_IDS {
                assert_eq!(
                    input.joystick(id).status(),
                    if previous_initialized {
                        JoystickStatus::Absent
                    } else {
                        JoystickStatus::NotTracked
                    }
                );
            }
            update_joystick_snapshots(true, &mut initialized, &mut input, |id| Ok(held_sample(id)));
            assert!(initialized);
            assert!(input
                .joystick(JoystickId::Joystick6)
                .is_down(GamepadButton::A));
            assert_eq!(
                input
                    .joystick(JoystickId::Joystick6)
                    .is_pressed(GamepadButton::A),
                previous_initialized
            );
        }
    }
}
