// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! One-shot joystick/gamepad sampling without a context, Vulkan instance, or window.

use vmnl::{GamepadAxis, InputRuntime, JoystickId, VMNLResult};

fn main() -> VMNLResult<()> {
    let runtime = InputRuntime::acquire()?;
    println!("GLFW input configuration: {:?}", runtime.configuration());

    let mut found_joystick = false;
    for index in 0..JoystickId::COUNT {
        let Some(id) = JoystickId::from_index(index) else {
            continue;
        };
        let Some(sample) = runtime.sample_joystick(id)? else {
            continue;
        };

        found_joystick = true;
        println!(
            "{id:?}: name={:?}, GUID={:?}, axes={}, buttons={}, hats={}",
            sample.name(),
            sample.guid(),
            sample.axes().len(),
            sample.buttons().len(),
            sample.hats().len(),
        );

        if let Some(gamepad) = sample.gamepad() {
            println!(
                "  mapped gamepad={:?}, left stick=({}, {})",
                gamepad.name(),
                gamepad.axis(GamepadAxis::LeftX),
                gamepad.axis(GamepadAxis::LeftY),
            );
        } else {
            println!("  no GLFW gamepad mapping");
        }
    }

    if !found_joystick {
        println!("No joystick is currently connected.");
    }

    Ok(())
}
