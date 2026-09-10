// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless public input-state contracts.

use vmnl::{Input, Joystick, JoystickState, Key, MouseButton, StickState, VMNLResult};

fn assert_empty(input: &Input) {
    let keyboard = input.keyboard();
    assert!(!keyboard.is_down(Key::Escape));
    assert!(!keyboard.is_down(Key::Left));
    assert!(!keyboard.is_pressed(Key::Escape));
    assert!(!keyboard.is_released(Key::Left));
    assert!(!keyboard.is_any_down(&[Key::Escape, Key::Left]));
    assert!(!keyboard.is_one_down());
    assert!(!keyboard.is_one_used());

    let joystick: &JoystickState = input.joystick();
    for stick in [joystick.left(), joystick.right()] {
        let stick: &StickState = stick;
        assert_eq!(stick.degrees(), None);
        assert!(!stick.is_clicked());
    }
    for control in [
        Joystick::JoystickLeft { degrees: None },
        Joystick::JoystickLeftButton,
        Joystick::JoystickRight { degrees: None },
        Joystick::JoystickRightButton,
    ] {
        assert!(!joystick.is_down(control));
        assert!(!joystick.is_pressed(control));
        assert!(!joystick.is_released(control));
    }
    assert!(!joystick.is_one_used());

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
