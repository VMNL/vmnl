// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless public input-state contracts.

use vmnl::{
    Input, Joystick, JoystickState, Key, MouseButton, StickSettings, StickState, VMNLResult,
};

#[test]
fn stick_axes_survive_interpretation_through_facade() -> VMNLResult<()> {
    let small = StickState::with_axes([0.05, 0.0], true, StickSettings::default())?;
    let full = StickState::with_axes([1.0, 0.0], false, StickSettings::default())?;
    assert_eq!(
        small.axes().map(f32::to_bits),
        [0.05_f32, 0.0].map(f32::to_bits)
    );
    assert_eq!(small.magnitude().to_bits(), 0.05_f32.to_bits());
    assert_eq!(small.degrees(), None);
    assert!(small.is_clicked());
    assert_eq!(full.magnitude().to_bits(), 1.0_f32.to_bits());
    assert_eq!(full.degrees(), Some(0.0));
    let tilted = StickState::with_axes([0.2, 0.0], false, StickSettings::default())?;
    assert_eq!(tilted.degrees(), full.degrees());
    assert_ne!(
        tilted.axes().map(f32::to_bits),
        full.axes().map(f32::to_bits)
    );
    assert_ne!(tilted.magnitude().to_bits(), full.magnitude().to_bits());

    let settings = StickSettings {
        dead_zone: 0.0,
        zero_degrees: 90.0,
        clockwise: true,
    };
    let reinterpreted = StickState::with_axes(small.axes(), true, settings)?;
    assert_eq!(
        reinterpreted.axes().map(f32::to_bits),
        small.axes().map(f32::to_bits)
    );
    assert_eq!(reinterpreted.degrees(), Some(90.0));
    assert_eq!(
        StickState::with_axes([0.0, 0.0], false, settings)?.degrees(),
        None
    );
    assert!(StickState::with_axes([1.0, 1.0], false, settings)?.magnitude() > 1.0);
    Ok(())
}

#[test]
fn configured_angles_wrap_and_dead_zone_boundary_is_inclusive() -> VMNLResult<()> {
    for zero_degrees in [90.0, 450.0, -270.0] {
        for (axes, expected) in [
            ([1.0, 0.0], 90.0),
            ([0.0, -1.0], 0.0),
            ([-1.0, 0.0], 270.0),
            ([0.0, 1.0], 180.0),
        ] {
            let settings = StickSettings {
                dead_zone: 0.25,
                zero_degrees,
                clockwise: true,
            };
            assert_eq!(
                StickState::with_axes(axes, false, settings)?.degrees(),
                Some(expected)
            );
            assert_eq!(
                StickState::with_axes([0.25, 0.0], false, settings)?.degrees(),
                None
            );
            assert!(StickState::with_axes([0.26, 0.0], false, settings)?
                .degrees()
                .is_some());
        }
    }
    assert!(StickState::with_axes(
        [1.0, 0.0],
        false,
        StickSettings {
            dead_zone: -1.0,
            ..StickSettings::default()
        }
    )
    .is_err());
    Ok(())
}

#[test]
fn stick_settings_are_per_stick_inspectable_and_validated() -> VMNLResult<()> {
    let left = Joystick::JoystickLeftButton;
    let right = Joystick::JoystickRightButton;
    let mut input = Input::new();
    let settings = StickSettings {
        dead_zone: 0.0,
        ..StickSettings::default()
    };
    input.set_stick_settings(left, settings)?;
    assert_eq!(input.joystick().settings(left), settings);
    assert_eq!(input.joystick().settings(right), StickSettings::default());
    for dead_zone in [-0.1, f32::NAN, f32::INFINITY] {
        assert!(input
            .set_stick_settings(
                left,
                StickSettings {
                    dead_zone,
                    ..settings
                }
            )
            .is_err());
        assert_eq!(input.joystick().settings(left), settings);
    }
    assert!(input
        .set_stick_settings(
            left,
            StickSettings {
                zero_degrees: f32::NAN,
                ..settings
            }
        )
        .is_err());
    // Ensure the same configuration entry point is exposed on the window without creating one.
    let _: fn(&mut vmnl::Window, Joystick, StickSettings) -> VMNLResult<()> =
        vmnl::Window::set_stick_settings;
    Ok(())
}

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
