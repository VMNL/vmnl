// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use std::ffi::c_void;
use vmnl::{
    Event, EventKind, GamepadAxis, GamepadButton, GamepadState, InputRuntime, InputRuntimeBuilder,
    InputRuntimeConfig, JoystickButtonState, JoystickHatState, JoystickId, JoystickSample,
    VMNLResult, Window, WindowBuilder,
};

#[test]
fn input_runtime_lifecycle_api_is_exported_by_the_facade() {
    let _: InputRuntimeBuilder = InputRuntime::builder().hat_buttons(false);
    let _: fn() -> VMNLResult<InputRuntime> = InputRuntime::acquire;
    let _: fn(InputRuntimeBuilder) -> VMNLResult<InputRuntime> = InputRuntimeBuilder::build;
    let _: fn(&InputRuntime) -> InputRuntimeConfig = InputRuntime::configuration;
    let _: fn(InputRuntimeConfig) -> bool = InputRuntimeConfig::hat_buttons;
    let _: fn(&InputRuntime, JoystickId) -> VMNLResult<Option<JoystickSample>> =
        InputRuntime::sample_joystick;
    let _: fn(&mut InputRuntime) -> Vec<Event> = InputRuntime::poll_events;
    let _: fn(&InputRuntime, &str) -> VMNLResult<()> = InputRuntime::update_gamepad_mappings;
    let _: unsafe fn(&InputRuntime, JoystickId) -> *mut c_void =
        InputRuntime::joystick_user_pointer;
    let _: unsafe fn(&InputRuntime, JoystickId, *mut c_void) =
        InputRuntime::set_joystick_user_pointer;
    let _: fn(&mut Window, bool) = Window::set_joystick_event_delivery;
    let _: fn(&Window) -> bool = Window::is_joystick_event_delivery_enabled;
    let _: WindowBuilder = Window::builder().joystick_event_delivery(true);

    let _: fn(JoystickId) -> usize = JoystickId::index;
    let _: fn(usize) -> Option<JoystickId> = JoystickId::from_index;
    let _: fn(&JoystickSample) -> Option<&GamepadState> = JoystickSample::gamepad;
    let _: fn(&GamepadState, GamepadButton) -> JoystickButtonState = GamepadState::button;
    let _: fn(&GamepadState, GamepadAxis) -> f32 = GamepadState::axis;
}

#[test]
fn joystick_ids_cover_the_public_sixteen_slot_contract() {
    for index in 0..JoystickId::COUNT {
        let id = JoystickId::from_index(index).expect("every GLFW joystick slot is valid");
        assert_eq!(id.index(), index);
    }

    assert_eq!(JoystickId::from_index(JoystickId::COUNT), None);
    assert_eq!(JoystickId::from_index(usize::MAX), None);
    assert_eq!(GamepadButton::COUNT, 15);
    assert_eq!(GamepadAxis::COUNT, 6);
    assert_eq!(JoystickHatState::RIGHT_UP.as_raw(), 3);
    assert!(matches!(
        EventKind::JoystickConnected {
            id: JoystickId::Joystick1
        },
        EventKind::JoystickConnected { .. }
    ));
}

#[test]
fn standalone_joystick_runtime_samples_and_updates_mappings_without_vulkan() {
    glfw::init_hint(glfw::InitHint::Platform(glfw::Platform::Null));
    let mut runtime = InputRuntime::acquire().expect("GLFW Null runtime should initialize");

    for index in 0..JoystickId::COUNT {
        let id = JoystickId::from_index(index).expect("valid joystick slot");
        assert!(runtime
            .sample_joystick(id)
            .expect("Null query should succeed")
            .is_none());
    }
    runtime.set_joystick_callback(|_, _| {});
    assert!(runtime.poll_events().is_empty());
    runtime.unset_joystick_callback();

    let mapping = "00000000000000000000000000000001,VMNL test,a:b0,";
    runtime
        .update_gamepad_mappings(mapping)
        .expect("valid mapping should be accepted");
    runtime
        .update_gamepad_mappings("00000000000000000000000000000001,VMNL replacement,a:b1,")
        .expect("mapping replacement should be accepted");

    let invalid_guid = runtime
        .update_gamepad_mappings("abcdef,invalid GUID,a:b0,")
        .expect_err("GLFW parser callback error must not be reported as success");
    assert!(matches!(
        invalid_guid.kind(),
        vmnl::VMNLErrorKind::GlfwInputOperationFailed { .. }
    ));
    assert!(matches!(
        runtime.update_gamepad_mappings("non-ASCII é"),
        Err(error) if matches!(error.kind(), vmnl::VMNLErrorKind::InvalidGamepadMapping(_))
    ));
    assert!(matches!(
        runtime.update_gamepad_mappings("contains\0nul"),
        Err(error) if matches!(error.kind(), vmnl::VMNLErrorKind::InvalidGamepadMapping(_))
    ));
}
