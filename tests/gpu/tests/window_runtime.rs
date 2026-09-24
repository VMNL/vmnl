// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU/display contracts for deterministic window runtime behavior.

use vmnl::{common::Rgba, Context, VMNLError, VMNLErrorKind, VMNLResult, Window};
use vmnl_gpu_tests::gpu_test_guard;

#[test]
#[ignore = "Requires Vulkan + GLFW display; connected controllers extend hardware coverage."]
fn window_poll_events_refreshes_joystick_snapshots_and_delivers_input_events() -> VMNLResult<()> {
    use vmnl::{Event, GamepadButton, JoystickId, Stick};

    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder()
        .unset_configure_window_polling()
        .build(&context)?;
    // Reset after draining callbacks so each present slot must produce a fallback connection.
    let _ = window.poll_events();
    for id in JoystickId::ALL {
        window.joystick_mut(id).reset();
    }
    let events = window.poll_events();
    for id in JoystickId::ALL {
        let state = window.input().joystick(id);
        if state.is_connected() {
            assert!(events.contains(&Event::JoystickConnected { id }));
        } else {
            assert!(state.gamepad().is_none());
            assert!(state.raw().axes().is_empty());
        }
        for button in GamepadButton::ALL {
            assert_eq!(state.is_pressed(button), state.is_down(button));
            assert_eq!(
                events.contains(&Event::JoystickButtonPressed { id, button }),
                state.is_down(button)
            );
        }
        for (stick, snapshot) in [(Stick::Left, state.left()), (Stick::Right, state.right())] {
            let changed = snapshot.axes().map(f32::to_bits) != [0.0_f32; 2].map(f32::to_bits);
            let movement = events.iter().find_map(|event| match event {
                Event::JoystickMoved {
                    id: event_id,
                    stick: event_stick,
                    axes,
                    degrees,
                } if *event_id == id && *event_stick == stick => Some((axes, degrees)),
                _ => None,
            });
            assert_eq!(movement.is_some(), changed);
            if let Some((axes, degrees)) = movement {
                assert_eq!(axes.map(f32::to_bits), snapshot.axes().map(f32::to_bits));
                assert_eq!(*degrees, snapshot.degrees());
            }
        }
    }
    Ok(())
}

fn assert_invalid_window_size<T>(result: VMNLResult<T>) -> VMNLResult<()> {
    match result {
        Err(error) => {
            assert!(matches!(error.kind(), VMNLErrorKind::InvalidWindowSize));
            Ok(())
        }
        Ok(_) => Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "invalid window size unexpectedly succeeded".into(),
        ))),
    }
}

fn assert_invalid_aspect_ratio<T>(result: VMNLResult<T>) -> VMNLResult<()> {
    match result {
        Err(error) => {
            assert!(matches!(
                error.kind(),
                VMNLErrorKind::InvalidState(message)
                    if message == "window aspect ratio terms must be positive and fit GLFW int"
            ));
            Ok(())
        }
        Ok(_) => Err(VMNLError::new(VMNLErrorKind::InvalidState(
            "invalid window aspect ratio unexpectedly succeeded".into(),
        ))),
    }
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn window_configuration_round_trips_and_validates() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL GPU window configuration")
        .size(640, 480)
        .set_clear_color(Rgba::rgb(12, 24, 48))
        .build(&context)?;

    assert!(window.is_ready());
    assert!(window.is_open());
    assert_eq!(window.get_title(), "VMNL GPU window configuration");
    assert_eq!(window.get_size(), (640, 480));
    assert_eq!(window.width(), 640);
    assert_eq!(window.height(), 480);
    let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
    assert!(framebuffer_width > 0);
    assert!(framebuffer_height > 0);
    let (scale_x, scale_y) = window.get_content_scale();
    assert!(scale_x.is_finite() && scale_x > 0.0);
    assert!(scale_y.is_finite() && scale_y > 0.0);
    assert_eq!(
        window.monitor().infos().len(),
        window.monitor().names().len()
    );

    window.set_title("VMNL updated title");
    window.set_size(800, 600)?;
    window.set_size_limits(Some(64), Some(64), Some(1600), Some(1200))?;
    window.set_aspect_ratio(Some((4, 3)))?;
    window.set_aspect_ratio(None)?;
    assert_eq!(window.get_title(), "VMNL updated title");
    assert_eq!(window.get_size(), (800, 600));
    assert_eq!(window.width(), 800);
    assert_eq!(window.height(), 600);
    assert_invalid_window_size(window.set_size_limits(Some(801), None, Some(800), None))?;
    assert_invalid_aspect_ratio(window.set_aspect_ratio(Some((0, 3))))?;
    assert_invalid_aspect_ratio(window.set_aspect_ratio(Some((3, 0))))?;
    assert_invalid_aspect_ratio(window.set_aspect_ratio(Some((u32::MAX, 1))))?;

    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn window_rejects_invalid_initial_and_runtime_sizes() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;

    assert_invalid_window_size(Window::builder().size(63, 64).build(&context))?;
    assert_invalid_window_size(Window::builder().size(64, 63).build(&context))?;

    let mut window = Window::new(&context)?;
    assert_invalid_window_size(window.set_size(63, 64))?;
    assert_invalid_window_size(window.set_size(64, 63))?;
    assert_invalid_window_size(window.set_size(u32::MAX, 64))?;
    Ok(())
}

#[test]
#[ignore = "Requires Vulkan + GLFW display."]
fn window_event_timer_and_close_contracts_hold() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    let context = Context::new()?;
    let mut window = Window::builder()
        .unset_configure_window_polling()
        .build(&context)?;

    window.set_time(10.0);
    assert!(window.get_time() >= 10.0);
    assert!(window.get_timer_frequency() > 0);
    let _timer_value = window.get_timer_value();
    window.post_empty_event();
    window.wait_events();
    window.wait_events_timeout(0.001);
    let _events = window.poll_events();
    assert!(!window.input().keyboard().is_one_used());
    assert!(!window.input().mouse().is_one_used());
    window.close();
    assert!(!window.is_open());

    Ok(())
}
