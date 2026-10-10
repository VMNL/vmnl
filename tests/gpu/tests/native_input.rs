// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

#![cfg(target_os = "linux")]
#![allow(clippy::panic)]

use std::time::{Duration, Instant};
use vmnl::{
    common::Rgba, Context, Event, EventKind, Key, MouseButton, VMNLError, VMNLErrorKind,
    VMNLResult, Window,
};
use vmnl_gpu_tests::gpu_test_guard;

const INPUT_TIMEOUT: Duration = Duration::from_secs(5);
const KEY_PRESS: u8 = x11rb::protocol::xproto::KEY_PRESS_EVENT;
const KEY_RELEASE: u8 = x11rb::protocol::xproto::KEY_RELEASE_EVENT;
const BUTTON_PRESS: u8 = x11rb::protocol::xproto::BUTTON_PRESS_EVENT;
const BUTTON_RELEASE: u8 = x11rb::protocol::xproto::BUTTON_RELEASE_EVENT;
const BUTTON_LEFT: u8 = 1;
const BUTTON_SCROLL_UP: u8 = 4;
const BUTTON_SCROLL_DOWN: u8 = 5;
const BUTTON_SCROLL_POSITIVE_X: u8 = 6;
const BUTTON_SCROLL_NEGATIVE_X: u8 = 7;

#[test]
#[ignore = "requires Vulkan, an X11 EWMH display, XTEST, and injects native keyboard/mouse input"]
fn vmnl_public_native_keyboard_and_mouse_events_update_input() -> VMNLResult<()> {
    let _guard = gpu_test_guard();
    glfw::init_hint(glfw::InitHint::Platform(glfw::Platform::X11));

    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL public native input test")
        .size(480, 320)
        .set_clear_color(Rgba::rgb(12, 24, 48))
        .build(&context)?;
    window.enable_keyboard_polling();
    window.enable_mouse_polling();
    window.show();
    window.focus();
    window.set_cursor_position(120.0, 100.0)?;

    let focus_deadline = Instant::now() + INPUT_TIMEOUT;
    while !window.is_focused() && Instant::now() < focus_deadline {
        let _ = window.poll_events();
        window.wait_events_timeout(0.02);
    }
    assert!(
        window.is_focused(),
        "VMNL input window did not acquire X11 focus"
    );
    let _ = window.poll_events();
    assert!(!window.input().keyboard().is_down(Key::A));
    assert!(!window.input().mouse().is_down(MouseButton::Left));

    let mut injector = X11Injector::connect().map_err(invalid_state)?;

    injector.press_key_a().map_err(invalid_state)?;
    let key_press_events = poll_until(&mut window, "A key press", |kind| {
        matches!(
            kind,
            EventKind::KeyPressed {
                key: Key::A,
                repeat: false,
                ..
            }
        )
    })?;
    let key_scancode = assert_key_press(&key_press_events)?;
    assert!(window.input().keyboard().is_down(Key::A));
    assert!(window.input().keyboard().is_pressed(Key::A));
    assert!(!window.input().keyboard().is_released(Key::A));

    injector.release_key_a().map_err(invalid_state)?;
    let key_release_events = poll_until(&mut window, "A key release", |kind| {
        matches!(kind, EventKind::KeyReleased { key: Key::A, .. })
    })?;
    assert_key_release(&key_release_events, key_scancode)?;
    assert!(!window.input().keyboard().is_down(Key::A));
    assert!(!window.input().keyboard().is_pressed(Key::A));
    assert!(window.input().keyboard().is_released(Key::A));

    injector.press_left_button().map_err(invalid_state)?;
    let mouse_press_events = poll_until(&mut window, "left mouse button press", |kind| {
        matches!(
            kind,
            EventKind::MouseButtonPressed {
                button: MouseButton::Left,
                ..
            }
        )
    })?;
    assert_mouse_button_event(&mouse_press_events, MouseButton::Left, true)?;
    assert!(window.input().mouse().is_down(MouseButton::Left));
    assert!(window.input().mouse().is_pressed(MouseButton::Left));
    assert!(!window.input().mouse().is_released(MouseButton::Left));

    injector.release_left_button().map_err(invalid_state)?;
    let mouse_release_events = poll_until(&mut window, "left mouse button release", |kind| {
        matches!(
            kind,
            EventKind::MouseButtonReleased {
                button: MouseButton::Left,
                ..
            }
        )
    })?;
    assert_mouse_button_event(&mouse_release_events, MouseButton::Left, false)?;
    assert!(!window.input().mouse().is_down(MouseButton::Left));
    assert!(!window.input().mouse().is_pressed(MouseButton::Left));
    assert!(window.input().mouse().is_released(MouseButton::Left));

    for (x11_button, vmnl_button) in [
        (2, MouseButton::Middle),
        (3, MouseButton::Right),
        (8, MouseButton::Button4),
        (9, MouseButton::Button5),
        (10, MouseButton::Button6),
        (11, MouseButton::Button7),
        (12, MouseButton::Button8),
    ] {
        let name = format!("{vmnl_button:?}");
        injector
            .press_mouse_button(x11_button)
            .map_err(invalid_state)?;
        let press_events = poll_until(
            &mut window,
            &format!("{name} mouse button press"),
            |kind| matches!(kind, EventKind::MouseButtonPressed { button, .. } if *button == vmnl_button),
        )?;
        assert_mouse_button_event(&press_events, vmnl_button, true)?;
        assert!(window.input().mouse().is_down(vmnl_button));
        assert!(window.input().mouse().is_pressed(vmnl_button));
        assert!(!window.input().mouse().is_released(vmnl_button));

        injector
            .release_mouse_button(x11_button)
            .map_err(invalid_state)?;
        let release_events = poll_until(
            &mut window,
            &format!("{name} mouse button release"),
            |kind| matches!(kind, EventKind::MouseButtonReleased { button, .. } if *button == vmnl_button),
        )?;
        assert_mouse_button_event(&release_events, vmnl_button, false)?;
        assert!(!window.input().mouse().is_down(vmnl_button));
        assert!(!window.input().mouse().is_pressed(vmnl_button));
        assert!(window.input().mouse().is_released(vmnl_button));
    }

    injector.scroll_vertical_up().map_err(invalid_state)?;
    let scroll_up_events = poll_until(
        &mut window,
        "vertical scroll up",
        |kind| matches!(kind, EventKind::MouseScrolled { dx, dy } if *dx == 0.0 && *dy == 1.0),
    )?;
    assert_mouse_scroll_event(&scroll_up_events, 0.0, 1.0)?;

    injector.scroll_vertical_down().map_err(invalid_state)?;
    let scroll_down_events = poll_until(
        &mut window,
        "vertical scroll down",
        |kind| matches!(kind, EventKind::MouseScrolled { dx, dy } if *dx == 0.0 && *dy == -1.0),
    )?;
    assert_mouse_scroll_event(&scroll_down_events, 0.0, -1.0)?;

    injector
        .scroll_horizontal_positive()
        .map_err(invalid_state)?;
    let scroll_right_events = poll_until(
        &mut window,
        "positive horizontal scroll",
        |kind| matches!(kind, EventKind::MouseScrolled { dx, dy } if *dx == 1.0 && *dy == 0.0),
    )?;
    assert_mouse_scroll_event(&scroll_right_events, 1.0, 0.0)?;

    injector
        .scroll_horizontal_negative()
        .map_err(invalid_state)?;
    let scroll_left_events = poll_until(
        &mut window,
        "negative horizontal scroll",
        |kind| matches!(kind, EventKind::MouseScrolled { dx, dy } if *dx == -1.0 && *dy == 0.0),
    )?;
    assert_mouse_scroll_event(&scroll_left_events, -1.0, 0.0)?;

    Ok(())
}

fn poll_until(
    window: &mut Window,
    expected: &str,
    matches_expected: impl Fn(&EventKind) -> bool,
) -> VMNLResult<Vec<Event>> {
    let deadline = Instant::now() + INPUT_TIMEOUT;
    let mut observed = Vec::new();
    while Instant::now() < deadline {
        if !window.is_focused() {
            return Err(invalid_state(format!(
                "VMNL input window lost focus while waiting for {expected}; observed={observed:?}"
            )));
        }
        let batch = window.poll_events();
        let found = batch.iter().any(|event| matches_expected(event.kind()));
        observed.extend(batch);
        if found {
            return Ok(observed);
        }
        window.wait_events_timeout(0.02);
    }

    Err(invalid_state(format!(
        "timed out waiting for {expected}; focused={}; observed={observed:?}",
        window.is_focused()
    )))
}

fn assert_key_press(events: &[Event]) -> VMNLResult<i32> {
    let mut matching = events.iter().filter_map(|event| match event.kind() {
        EventKind::KeyPressed {
            key: Key::A,
            scancode,
            modifiers,
            repeat: false,
        } => Some((*scancode, *modifiers)),
        _ => None,
    });
    let Some((scancode, modifiers)) = matching.next() else {
        return Err(invalid_state(format!(
            "A press event was absent: {events:?}"
        )));
    };
    if matching.next().is_some() || scancode.as_raw() <= 0 || !modifiers.is_empty() {
        return Err(invalid_state(format!(
            "A press event had duplicate or invalid metadata: {events:?}"
        )));
    }
    Ok(scancode.as_raw())
}

fn assert_key_release(events: &[Event], expected_scancode: i32) -> VMNLResult<()> {
    let mut matching = events.iter().filter_map(|event| match event.kind() {
        EventKind::KeyReleased {
            key: Key::A,
            scancode,
            modifiers,
        } => Some((*scancode, *modifiers)),
        _ => None,
    });
    let Some((scancode, modifiers)) = matching.next() else {
        return Err(invalid_state(format!(
            "A release event was absent: {events:?}"
        )));
    };
    if matching.next().is_some() || scancode.as_raw() != expected_scancode || !modifiers.is_empty()
    {
        return Err(invalid_state(format!(
            "A release event had duplicate or mismatched metadata: {events:?}"
        )));
    }
    Ok(())
}

fn assert_mouse_button_event(
    events: &[Event],
    expected_button: MouseButton,
    pressed: bool,
) -> VMNLResult<()> {
    let matching = events.iter().filter(|event| match (pressed, event.kind()) {
        (true, EventKind::MouseButtonPressed { button, modifiers })
        | (false, EventKind::MouseButtonReleased { button, modifiers }) => {
            *button == expected_button && modifiers.is_empty()
        }
        _ => false,
    });
    if matching.count() != 1 {
        return Err(invalid_state(format!(
            "expected one {expected_button:?} mouse button {} event: {events:?}",
            if pressed { "press" } else { "release" },
        )));
    }
    Ok(())
}

fn assert_mouse_scroll_event(
    events: &[Event],
    expected_dx: f64,
    expected_dy: f64,
) -> VMNLResult<()> {
    let mut observed = events.iter().filter_map(|event| match event.kind() {
        EventKind::MouseScrolled { dx, dy } => Some((*dx, *dy)),
        _ => None,
    });
    if observed.next() != Some((expected_dx, expected_dy)) || observed.next().is_some() {
        return Err(invalid_state(format!(
            "expected exactly one scroll event with dx={expected_dx}, dy={expected_dy}: {events:?}"
        )));
    }
    Ok(())
}

fn invalid_state(message: impl Into<String>) -> VMNLError {
    VMNLError::new(VMNLErrorKind::InvalidState(message.into()))
}

struct X11Injector {
    connection: x11rb::rust_connection::RustConnection,
    a_keycode: u8,
    key_is_down: bool,
    mouse_button_is_down: Option<u8>,
    scroll_button_is_down: Option<u8>,
}

impl X11Injector {
    fn connect() -> Result<Self, String> {
        const XK_A_UPPER: u32 = 0x0041;
        const XK_A_LOWER: u32 = 0x0061;

        use x11rb::{
            connection::Connection as _, protocol::xproto::ConnectionExt as _,
            protocol::xtest::ConnectionExt as _,
        };

        let (connection, _) = x11rb::connect(None)
            .map_err(|error| format!("could not connect to DISPLAY: {error}"))?;
        connection
            .xtest_get_version(2, 2)
            .map_err(|error| format!("could not query XTEST: {error}"))?
            .reply()
            .map_err(|error| format!("XTEST is unavailable: {error}"))?;

        let setup = connection.setup();
        let keycode_count = setup.max_keycode - setup.min_keycode + 1;
        let mapping = connection
            .get_keyboard_mapping(setup.min_keycode, keycode_count)
            .map_err(|error| format!("could not request the X11 keyboard map: {error}"))?
            .reply()
            .map_err(|error| format!("could not read the X11 keyboard map: {error}"))?;
        let keysyms_per_keycode = usize::from(mapping.keysyms_per_keycode);
        if keysyms_per_keycode == 0 {
            return Err("X11 returned an empty keyboard map".to_owned());
        }
        let keycode_offset = mapping
            .keysyms
            .chunks(keysyms_per_keycode)
            .position(|keysyms| keysyms.contains(&XK_A_UPPER) || keysyms.contains(&XK_A_LOWER))
            .ok_or_else(|| "X11 keyboard map has no A keysym".to_owned())?;
        let a_keycode = u16::from(setup.min_keycode)
            + u16::try_from(keycode_offset)
                .map_err(|_| "X11 A keycode offset is too large".to_owned())?;
        let a_keycode =
            u8::try_from(a_keycode).map_err(|_| "X11 A keycode is invalid".to_owned())?;

        Ok(Self {
            connection,
            a_keycode,
            key_is_down: false,
            mouse_button_is_down: None,
            scroll_button_is_down: None,
        })
    }

    fn press_key_a(&mut self) -> Result<(), String> {
        self.key_is_down = true;
        self.fake_input(KEY_PRESS, self.a_keycode, "A key press")
    }

    fn release_key_a(&mut self) -> Result<(), String> {
        self.fake_input(KEY_RELEASE, self.a_keycode, "A key release")?;
        self.key_is_down = false;
        Ok(())
    }

    fn press_left_button(&mut self) -> Result<(), String> {
        self.press_mouse_button(BUTTON_LEFT)
    }

    fn release_left_button(&mut self) -> Result<(), String> {
        self.release_mouse_button(BUTTON_LEFT)
    }

    fn press_mouse_button(&mut self, button: u8) -> Result<(), String> {
        self.mouse_button_is_down = Some(button);
        self.fake_input(BUTTON_PRESS, button, "mouse button press")
    }

    fn release_mouse_button(&mut self, button: u8) -> Result<(), String> {
        self.fake_input(BUTTON_RELEASE, button, "mouse button release")?;
        self.mouse_button_is_down = None;
        Ok(())
    }

    fn scroll_vertical_up(&mut self) -> Result<(), String> {
        self.scroll(BUTTON_SCROLL_UP, "vertical scroll up")
    }

    fn scroll_vertical_down(&mut self) -> Result<(), String> {
        self.scroll(BUTTON_SCROLL_DOWN, "vertical scroll down")
    }

    fn scroll_horizontal_positive(&mut self) -> Result<(), String> {
        self.scroll(BUTTON_SCROLL_POSITIVE_X, "positive horizontal scroll")
    }

    fn scroll_horizontal_negative(&mut self) -> Result<(), String> {
        self.scroll(BUTTON_SCROLL_NEGATIVE_X, "negative horizontal scroll")
    }

    fn scroll(&mut self, button: u8, direction: &str) -> Result<(), String> {
        self.scroll_button_is_down = Some(button);
        self.fake_input(BUTTON_PRESS, button, &format!("{direction} press"))?;
        self.fake_input(BUTTON_RELEASE, button, &format!("{direction} release"))?;
        self.scroll_button_is_down = None;
        Ok(())
    }

    fn fake_input(&self, event_type: u8, detail: u8, name: &str) -> Result<(), String> {
        use x11rb::{connection::Connection as _, protocol::xtest::ConnectionExt as _};

        self.connection
            .xtest_fake_input(event_type, detail, 0, 0, 0, 0, 0)
            .map_err(|error| format!("could not enqueue {name}: {error}"))?
            .check()
            .map_err(|error| format!("XTEST {name} failed: {error}"))?;
        self.connection
            .flush()
            .map_err(|error| format!("could not flush {name}: {error}"))
    }
}

impl Drop for X11Injector {
    fn drop(&mut self) {
        if self.key_is_down {
            let _ = self.fake_input(KEY_RELEASE, self.a_keycode, "cleanup A key release");
        }
        if let Some(button) = self.mouse_button_is_down {
            let _ = self.fake_input(BUTTON_RELEASE, button, "cleanup mouse release");
        }
        if let Some(button) = self.scroll_button_is_down {
            let _ = self.fake_input(BUTTON_RELEASE, button, "cleanup scroll release");
        }
    }
}
