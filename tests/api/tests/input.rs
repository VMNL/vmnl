// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless public input-state contracts.

use vmnl::{
    Context, Cursor, CursorBuilder, CursorMode, Event, EventKind, Input, Key, Modifiers,
    MouseButton, StandardCursor, StandardCursorBuilder, VMNLResult, Window,
};

fn assert_empty(input: &Input) {
    let keyboard = input.keyboard();
    assert!(!keyboard.is_down(Key::Escape));
    assert!(!keyboard.is_down(Key::Left));
    assert!(!keyboard.is_pressed(Key::Escape));
    assert!(!keyboard.is_released(Key::Left));
    assert!(!keyboard.is_any_down(&[Key::Escape, Key::Left]));
    assert!(!keyboard.is_one_down());
    assert!(!keyboard.is_one_used());

    let mouse = input.mouse();
    assert!(!mouse.is_down(MouseButton::Left));
    assert!(!mouse.is_down(MouseButton::Right));
    assert!(!mouse.is_pressed(MouseButton::Left));
    assert!(!mouse.is_released(MouseButton::Right));
    assert!(!mouse.is_any_down(&[MouseButton::Left, MouseButton::Right]));
    assert!(!mouse.is_one_down());
    assert!(!mouse.is_one_used());
}

#[test]
fn input_initial_state_is_empty_through_public_facade() -> VMNLResult<()> {
    assert_empty(&Input::new());
    assert_empty(&Input::default());
    Ok(())
}

#[test]
fn event_and_batch_control_are_exposed_through_public_facade() {
    fn assert_event_accessors(event: Event) {
        let _: f64 = event.timestamp_seconds();
        let _: &EventKind = event.kind();
        let _: EventKind = event.into_kind();
    }

    let _: fn(Event) = assert_event_accessors;
    let _: fn(&mut Window) = Window::clear_input_transitions;
    let _: fn(&Window) -> bool = Window::is_mouse_button_polling_enabled;
    let _: fn(&Window) -> bool = Window::is_cursor_pos_polling_enabled;
    let _: fn(&Window) -> bool = Window::is_cursor_enter_polling_enabled;
    let _: fn(&Window) -> bool = Window::is_scroll_polling_enabled;
}

#[test]
fn cursor_controls_are_exposed_through_public_facade() {
    let _: fn(&Context) -> bool = Context::is_raw_mouse_motion_supported;
    let _: fn(&Window) -> (f64, f64) = Window::get_cursor_position;
    let _: fn(&mut Window, f64, f64) -> VMNLResult<()> = Window::set_cursor_position;
    let _: fn(&Window) -> bool = Window::is_cursor_hovered;
    let _: fn(&Window) -> CursorMode = Window::get_cursor_mode;
    let _: fn(&mut Window, CursorMode) = Window::set_cursor_mode;
    let _: fn(&Window) -> bool = Window::is_sticky_mouse_buttons_enabled;
    let _: fn(&mut Window, bool) = Window::set_sticky_mouse_buttons;
    let _: fn(&Window) -> bool = Window::is_lock_key_modifier_reporting_enabled;
    let _: fn(&mut Window, bool) = Window::set_lock_key_modifier_reporting;
    let _: fn(&Window) -> bool = Window::is_raw_mouse_motion_enabled;
    let _: fn(&mut Window, bool) -> VMNLResult<()> = Window::set_raw_mouse_motion;

    assert_eq!(CursorMode::default(), CursorMode::Normal);
    assert_ne!(CursorMode::Hidden, CursorMode::Disabled);
    assert_ne!(CursorMode::Disabled, CursorMode::Captured);
}

#[test]
fn cursor_resources_are_exposed_through_public_facade() {
    fn assert_cursor_traits<T: Clone + std::fmt::Debug + Eq>() {}
    fn assert_standard_cursor_traits<T: Clone + Copy + std::fmt::Debug + Eq + std::hash::Hash>() {}
    fn build_custom_cursor(builder: CursorBuilder<'_>, context: &Context) -> VMNLResult<Cursor> {
        builder.build(context)
    }

    assert_cursor_traits::<Cursor>();
    assert_standard_cursor_traits::<StandardCursor>();

    let _: fn(StandardCursor) -> StandardCursorBuilder = Cursor::standard;
    let _: fn(StandardCursorBuilder, &Context) -> VMNLResult<Cursor> = StandardCursorBuilder::build;
    let pixels = [255_u8; 4];
    let builder: CursorBuilder<'_> = Cursor::rgba8(1, 1, &pixels)
        .hotspot(0, 0)
        .hotspot_marker([255, 0, 255, 255]);
    let _: fn(CursorBuilder<'_>, &Context) -> VMNLResult<Cursor> = build_custom_cursor;
    let _ = builder;
    let _: fn(&Window) -> Option<&Cursor> = Window::cursor;
    let _: fn(&mut Window, Option<&Cursor>) -> VMNLResult<()> = Window::set_cursor;

    let shapes = [
        StandardCursor::Arrow,
        StandardCursor::IBeam,
        StandardCursor::Crosshair,
        StandardCursor::PointingHand,
        StandardCursor::ResizeEastWest,
        StandardCursor::ResizeNorthSouth,
        StandardCursor::ResizeNorthwestSoutheast,
        StandardCursor::ResizeNortheastSouthwest,
        StandardCursor::ResizeAll,
        StandardCursor::NotAllowed,
    ];
    assert_eq!(shapes.len(), 10);
}

#[test]
fn modifier_flags_preserve_independent_bits() {
    let modifiers = Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::CAPS_LOCK;

    assert!(modifiers.contains(Modifiers::SHIFT));
    assert!(modifiers.contains(Modifiers::CONTROL));
    assert!(modifiers.contains(Modifiers::CAPS_LOCK));
    assert!(!modifiers.contains(Modifiers::ALT));
    assert!(!modifiers.is_empty());
    assert_eq!(Modifiers::NONE.bits(), 0);
}

#[test]
fn mouse_event_kind_exposes_modifiers_and_fractional_values() {
    let pressed = EventKind::MouseButtonPressed {
        button: MouseButton::Left,
        modifiers: Modifiers::SHIFT,
    };
    let moved = EventKind::MouseMoved { x: -0.5, y: 1.25 };

    assert!(matches!(
        pressed,
        EventKind::MouseButtonPressed {
            button: MouseButton::Left,
            modifiers
        } if modifiers == Modifiers::SHIFT
    ));
    assert!(matches!(moved, EventKind::MouseMoved { x: -0.5, y: 1.25 }));
}
