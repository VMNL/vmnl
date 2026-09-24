// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! One-operation GLFW probe used by the platform integration tests.

#![allow(clippy::print_stderr, clippy::print_stdout, clippy::too_many_lines)]

use glfw::{ClientApiHint, Context as _, InitHint, WindowHint, WindowMode};
use serde_json::{json, Value};
#[cfg(target_os = "linux")]
use std::ffi::c_void;
use std::{
    cell::RefCell,
    env, fs,
    process::ExitCode,
    rc::Rc,
    time::{Duration, Instant},
};
use vmnl_platform_tests::{backend_name, parse_backend, PROBE_SCHEMA_VERSION};

const NATIVE_INPUT_TIMEOUT: Duration = Duration::from_secs(5);

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);
    let Some(requested_name) = arguments.next() else {
        eprintln!("usage: platform_probe <backend> <operation>");
        return ExitCode::from(2);
    };
    let Some(operation) = arguments.next() else {
        eprintln!("usage: platform_probe <backend> <operation>");
        return ExitCode::from(2);
    };
    if arguments.next().is_some() {
        eprintln!("platform_probe accepts exactly two arguments");
        return ExitCode::from(2);
    }

    let Some(requested) = parse_backend(&requested_name) else {
        eprintln!("unknown GLFW backend: {requested_name}");
        return ExitCode::from(2);
    };

    glfw::init_hint(InitHint::Platform(requested));
    if requested == glfw::Platform::Wayland {
        // SAFETY: This is an initialization hint accepted before `glfwInit`; both constants come
        // from the bundled GLFW 3.4 headers. Disabling libdecor keeps the headless compositor
        // probe independent of GTK seat/theme integration.
        unsafe {
            glfw::ffi::glfwInitHint(
                glfw::ffi::GLFW_WAYLAND_LIBDECOR,
                glfw::ffi::GLFW_WAYLAND_DISABLE_LIBDECOR,
            );
        }
    }
    let callbacks = Rc::new(RefCell::new(Vec::<Value>::new()));
    let callback_records = Rc::clone(&callbacks);
    let Ok(mut glfw) = glfw::init(move |error, description| {
        callback_records.borrow_mut().push(json!({
            "code": error.as_raw(),
            "kind": error.to_string(),
            "description": description,
        }));
    }) else {
        emit(
            &requested_name,
            None,
            &operation,
            "initialization",
            &callbacks,
            &Value::Null,
            "error",
        );
        return ExitCode::from(10);
    };

    let actual = glfw.get_platform();
    if actual != requested {
        emit(
            &requested_name,
            Some(actual),
            &operation,
            "backend-selection",
            &callbacks,
            &Value::Null,
            "error",
        );
        return ExitCode::from(11);
    }

    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    if actual == glfw::Platform::Wayland && operation == "keyboard-native-input" {
        glfw.window_hint(WindowHint::Maximized(true));
    }
    glfw.window_hint(WindowHint::Visible(matches!(
        operation.as_str(),
        "keyboard-native-input" | "sticky-keys-manual"
    )));
    let Some((mut window, events)) =
        glfw.create_window(160, 120, "VMNL platform probe", WindowMode::Windowed)
    else {
        emit(
            &requested_name,
            Some(actual),
            &operation,
            "window-creation",
            &callbacks,
            &Value::Null,
            "error",
        );
        return ExitCode::from(12);
    };

    let value = match operation.as_str() {
        "create" => json!(true),
        "set-position" => {
            window.set_pos(37, 41);
            Value::Null
        }
        "get-position" => json!(window.get_pos()),
        "set-opacity" => {
            window.set_opacity(0.75);
            Value::Null
        }
        "get-opacity" => json!(window.get_opacity()),
        "iconify" => {
            window.iconify();
            Value::Null
        }
        "focus" => {
            window.focus();
            Value::Null
        }
        "cursor-position" => {
            window.show();
            window.focus();
            window.set_cursor_mode(glfw::CursorMode::Disabled);
            window.set_cursor_pos(37.5, 41.25);
            json!(window.get_cursor_pos())
        }
        "cursor-hover" => json!(window.is_hovered()),
        "cursor-modes" => {
            let modes = [
                glfw::CursorMode::Normal,
                glfw::CursorMode::Hidden,
                glfw::CursorMode::Disabled,
                glfw::CursorMode::Captured,
            ];
            let values: Vec<String> = modes
                .into_iter()
                .map(|mode| {
                    window.set_cursor_mode(mode);
                    format!("{:?}", window.get_cursor_mode())
                })
                .collect();
            json!(values)
        }
        "mouse-input-modes" => {
            let defaults = json!({
                "sticky_mouse_buttons": window.has_sticky_mouse_buttons(),
                "lock_key_modifier_reporting": window.does_store_lock_key_mods(),
            });
            window.set_sticky_mouse_buttons(true);
            window.set_store_lock_key_mods(true);
            let enabled = json!({
                "sticky_mouse_buttons": window.has_sticky_mouse_buttons(),
                "lock_key_modifier_reporting": window.does_store_lock_key_mods(),
            });
            window.set_sticky_mouse_buttons(false);
            window.set_store_lock_key_mods(false);
            json!({
                "defaults": defaults,
                "enabled": enabled,
                "disabled_after_reset": {
                    "sticky_mouse_buttons": !window.has_sticky_mouse_buttons(),
                    "lock_key_modifier_reporting": !window.does_store_lock_key_mods(),
                },
            })
        }
        "keyboard-metadata" => {
            let scancode = glfw::get_key_scancode(Some(glfw::Key::A));
            if actual == glfw::Platform::Wayland {
                json!({
                    "a_scancode": scancode,
                    "names_deferred_until_keyboard_event": true,
                })
            } else {
                json!({
                    "a_scancode": scancode,
                    "a_name_by_key": glfw::get_key_name(Some(glfw::Key::A), None),
                    "a_name_by_scancode": scancode
                        .and_then(|value| glfw::get_key_name(None, Some(value))),
                    "escape_name": glfw::get_key_name(Some(glfw::Key::Escape), None),
                })
            }
        }
        "keyboard-input-modes" => keyboard_input_modes(&mut window, &events),
        "keyboard-wait-events-then-poll" => {
            keyboard_wait_then_poll(&mut glfw, &mut window, &events, None)
        }
        "keyboard-wait-events-timeout-then-poll" => {
            keyboard_wait_then_poll(&mut glfw, &mut window, &events, Some(0.001))
        }
        "keyboard-native-input" => native_keyboard_input(&mut glfw, &mut window, &events, actual),
        "sticky-keys-manual" => manual_sticky_keys(&mut glfw, &mut window, &events),
        "raw-mouse-motion" => {
            let supported = glfw.supports_raw_motion();
            if supported {
                window.set_raw_mouse_motion(true);
            }
            let enabled = window.uses_raw_mouse_motion();
            if supported {
                window.set_raw_mouse_motion(false);
            }
            json!({
                "supported": supported,
                "enabled": enabled,
                "disabled_after_reset": !window.uses_raw_mouse_motion(),
            })
        }
        "wait-events-then-poll" => wait_then_poll(&mut glfw, &mut window, &events, None),
        "wait-events-timeout-then-poll" => {
            wait_then_poll(&mut glfw, &mut window, &events, Some(0.001))
        }
        "cursor-resources" => cursor_resources(&mut glfw, &mut window),
        "maximize" => {
            window.maximize();
            Value::Null
        }
        _ => {
            eprintln!("unknown probe operation: {operation}");
            return ExitCode::from(2);
        }
    };

    let operation_succeeded = !matches!(
        operation.as_str(),
        "keyboard-native-input" | "sticky-keys-manual"
    ) || value["qualified"] == true;
    emit(
        &requested_name,
        Some(actual),
        &operation,
        "operation",
        &callbacks,
        &value,
        if operation_succeeded { "ok" } else { "error" },
    );
    if operation_succeeded {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(13)
    }
}

fn keyboard_input_modes(
    window: &mut glfw::PWindow,
    events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
) -> Value {
    let default_sticky_keys = window.has_sticky_keys();
    window.set_sticky_keys(true);
    let enabled_sticky_keys = window.has_sticky_keys();
    window.set_key_polling(true);
    let window_ptr = window.window_ptr();

    // SAFETY: The window is live on the GLFW thread. Temporarily removing and restoring the
    // callback obtains the callback installed by glfw-rs without changing its final state.
    let callback = unsafe {
        let callback = glfw::ffi::glfwSetKeyCallback(window_ptr, None);
        glfw::ffi::glfwSetKeyCallback(window_ptr, callback);
        callback
    };
    let actions = callback.map_or_else(Vec::new, |callback| {
        let scancode = glfw::get_key_scancode(Some(glfw::Key::A)).unwrap_or_default();
        // SAFETY: The callback was installed by glfw-rs for this live window. The key, scancode,
        // actions and modifier mask are valid GLFW values and enter the glfw-rs receiver.
        unsafe {
            callback(
                window_ptr,
                glfw::ffi::GLFW_KEY_A,
                scancode,
                glfw::ffi::GLFW_PRESS,
                0,
            );
            callback(
                window_ptr,
                glfw::ffi::GLFW_KEY_A,
                scancode,
                glfw::ffi::GLFW_RELEASE,
                0,
            );
        }

        glfw::flush_messages(events)
            .filter_map(|(_, event)| match event {
                glfw::WindowEvent::Key(glfw::Key::A, _, action, _) => Some(format!("{action:?}")),
                _ => None,
            })
            .collect()
    });

    window.set_sticky_keys(false);
    json!({
        "default_sticky_keys": default_sticky_keys,
        "enabled_sticky_keys": enabled_sticky_keys,
        "disabled_after_reset": !window.has_sticky_keys(),
        "callback_installed": callback.is_some(),
        "actions": actions,
    })
}

fn keyboard_wait_then_poll(
    glfw: &mut glfw::Glfw,
    window: &mut glfw::PWindow,
    events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    timeout: Option<f64>,
) -> Value {
    window.set_key_polling(true);
    let window_ptr = window.window_ptr();
    // SAFETY: The window is live on the GLFW thread. Temporarily removing and restoring the
    // callback obtains the callback installed by glfw-rs without changing its final state.
    let callback = unsafe {
        let callback = glfw::ffi::glfwSetKeyCallback(window_ptr, None);
        glfw::ffi::glfwSetKeyCallback(window_ptr, callback);
        callback
    };
    let Some(callback) = callback else {
        return json!({"callback_installed": false});
    };
    let scancode = glfw::get_key_scancode(Some(glfw::Key::A)).unwrap_or_default();

    // SAFETY: The callback was installed by glfw-rs for this live window. The key, scancode,
    // actions and modifier mask are valid GLFW values and enter the glfw-rs receiver.
    unsafe {
        callback(
            window_ptr,
            glfw::ffi::GLFW_KEY_A,
            scancode,
            glfw::ffi::GLFW_PRESS,
            0,
        );
        callback(
            window_ptr,
            glfw::ffi::GLFW_KEY_A,
            scancode,
            glfw::ffi::GLFW_RELEASE,
            0,
        );
    }

    if let Some(seconds) = timeout {
        glfw.wait_events_timeout(seconds);
    } else {
        glfw.post_empty_event();
        glfw.wait_events();
    }
    glfw.poll_events();

    let actions: Vec<String> = glfw::flush_messages(events)
        .filter_map(|(_, event)| match event {
            glfw::WindowEvent::Key(glfw::Key::A, _, action, _) => Some(format!("{action:?}")),
            _ => None,
        })
        .collect();
    json!({
        "callback_installed": true,
        "actions_after_poll": actions,
    })
}

fn manual_sticky_keys(
    glfw: &mut glfw::Glfw,
    window: &mut glfw::PWindow,
    events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
) -> Value {
    eprintln!("Focus the probe window, then press and release A within 15 seconds.");
    window.set_key_polling(true);
    window.set_sticky_keys(true);
    window.show();
    window.focus();

    let deadline = Instant::now() + Duration::from_secs(15);
    let mut saw_press = false;
    let mut saw_release = false;
    while Instant::now() < deadline && !saw_release {
        glfw.wait_events_timeout(0.1);
        for (_, event) in glfw::flush_messages(events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::A, _, glfw::Action::Press, _) => {
                    saw_press = true;
                }
                glfw::WindowEvent::Key(glfw::Key::A, _, glfw::Action::Release, _) => {
                    saw_release = true;
                }
                _ => {}
            }
        }
    }

    let first_read = window.get_key(glfw::Key::A);
    let second_read = window.get_key(glfw::Key::A);
    window.set_sticky_keys(false);
    let qualified = saw_press
        && saw_release
        && first_read == glfw::Action::Press
        && second_read == glfw::Action::Release;

    json!({
        "qualified": qualified,
        "saw_press": saw_press,
        "saw_release": saw_release,
        "first_read": format!("{first_read:?}"),
        "second_read": format!("{second_read:?}"),
    })
}

#[cfg(target_os = "linux")]
struct WaylandProbeBuffer(*mut c_void);

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn vmnl_map_wayland_probe(
        display: *mut c_void,
        surface: *mut c_void,
        width: i32,
        height: i32,
        error: *mut i32,
    ) -> *mut c_void;
    fn vmnl_destroy_wayland_probe_buffer(buffer: *mut c_void);
}

#[cfg(target_os = "linux")]
impl Drop for WaylandProbeBuffer {
    fn drop(&mut self) {
        // SAFETY: The helper returned this live wl_buffer; it is destroyed before GLFW ends.
        unsafe { vmnl_destroy_wayland_probe_buffer(self.0) };
    }
}

#[cfg(target_os = "linux")]
fn map_wayland_probe(
    glfw: &mut glfw::Glfw,
    window: &glfw::PWindow,
) -> Result<WaylandProbeBuffer, String> {
    glfw.poll_events();
    let (width, height) = window.get_framebuffer_size();
    let mut error = 0;
    // SAFETY: GLFW owns both live Wayland objects for this window on this thread. The helper
    // attaches a test-only shm buffer and returns its ownership to WaylandProbeBuffer.
    let buffer = unsafe {
        vmnl_map_wayland_probe(
            glfw.get_wayland_display(),
            window.get_wayland_window(),
            width,
            height,
            &raw mut error,
        )
    };
    if buffer.is_null() {
        Err(format!(
            "Wayland shm mapping failed: code={error}, framebuffer={width}x{height}, maximized={}",
            window.is_maximized()
        ))
    } else {
        Ok(WaylandProbeBuffer(buffer))
    }
}

fn native_keyboard_input(
    glfw: &mut glfw::Glfw,
    window: &mut glfw::PWindow,
    events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    platform: glfw::Platform,
) -> Value {
    window.set_key_polling(true);
    window.show();
    if platform != glfw::Platform::Wayland {
        window.focus();
    }

    let Some(ready_file) = env::var_os("VMNL_PLATFORM_READY_FILE") else {
        return json!({
            "qualified": false,
            "stage": "handshake",
            "error": "VMNL_PLATFORM_READY_FILE is missing",
        });
    };

    #[cfg(target_os = "linux")]
    let _wayland_buffer = if platform == glfw::Platform::Wayland {
        match map_wayland_probe(glfw, window) {
            Ok(buffer) => Some(buffer),
            Err(error) => {
                return json!({
                    "qualified": false,
                    "stage": "mapping",
                    "error": error,
                });
            }
        }
    } else {
        None
    };
    if platform == glfw::Platform::Wayland {
        if let Err(error) = fs::write(&ready_file, b"MAPPED\n") {
            return json!({
                "qualified": false,
                "stage": "handshake",
                "error": error.to_string(),
            });
        }
    }
    let focus_deadline = Instant::now() + NATIVE_INPUT_TIMEOUT;
    while !window.is_focused() && Instant::now() < focus_deadline {
        glfw.wait_events_timeout(0.01);
        glfw::flush_messages(events).for_each(drop);
    }
    if !window.is_focused() {
        return json!({
            "qualified": false,
            "stage": "focus",
            "focused": false,
            "hovered": window.is_hovered(),
            "maximized": window.is_maximized(),
            "framebuffer_size": window.get_framebuffer_size(),
        });
    }

    if let Err(error) = fs::write(&ready_file, b"READY\n") {
        return json!({
            "qualified": false,
            "stage": "handshake",
            "error": error.to_string(),
        });
    }

    let input_deadline = Instant::now() + NATIVE_INPUT_TIMEOUT;
    let mut actions = Vec::new();
    let mut scancodes = Vec::new();
    while actions.as_slice() != ["Press", "Release"] && Instant::now() < input_deadline {
        glfw.wait_events_timeout(0.01);
        for (_, event) in glfw::flush_messages(events) {
            if let glfw::WindowEvent::Key(glfw::Key::A, scancode, action, _) = event {
                actions.push(format!("{action:?}"));
                scancodes.push(scancode);
            }
        }
    }

    let qualified = actions.as_slice() == ["Press", "Release"]
        && scancodes.len() == 2
        && scancodes[0] == scancodes[1]
        && window.get_key(glfw::Key::A) == glfw::Action::Release;
    json!({
        "qualified": qualified,
        "stage": "input",
        "focused": window.is_focused(),
        "injector": env::var("VMNL_PLATFORM_INPUT_INJECTOR")
            .unwrap_or_else(|_| "unknown".to_owned()),
        "actions": actions,
        "scancodes": scancodes,
        "final_state": format!("{:?}", window.get_key(glfw::Key::A)),
    })
}

fn wait_then_poll(
    glfw: &mut glfw::Glfw,
    window: &mut glfw::PWindow,
    events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    timeout: Option<f64>,
) -> Value {
    window.set_mouse_button_polling(true);
    let window_ptr = window.window_ptr();
    // SAFETY: The window is live on the GLFW thread. Temporarily removing and restoring the
    // callback obtains the callback installed by glfw-rs without changing its final state.
    let callback = unsafe {
        let callback = glfw::ffi::glfwSetMouseButtonCallback(window_ptr, None);
        glfw::ffi::glfwSetMouseButtonCallback(window_ptr, callback);
        callback
    };
    let Some(callback) = callback else {
        return json!({"callback_installed": false});
    };

    // SAFETY: The callback was installed by glfw-rs for this live window. Valid GLFW mouse-button
    // and action constants are used, so both events enter the window's glfw-rs receiver.
    unsafe {
        callback(
            window_ptr,
            glfw::ffi::GLFW_MOUSE_BUTTON_LEFT,
            glfw::ffi::GLFW_PRESS,
            0,
        );
        callback(
            window_ptr,
            glfw::ffi::GLFW_MOUSE_BUTTON_LEFT,
            glfw::ffi::GLFW_RELEASE,
            0,
        );
    }

    if let Some(seconds) = timeout {
        glfw.wait_events_timeout(seconds);
    } else {
        glfw.post_empty_event();
        glfw.wait_events();
    }
    glfw.poll_events();

    let actions: Vec<String> = glfw::flush_messages(events)
        .filter_map(|(_, event)| match event {
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, action, _) => {
                Some(format!("{action:?}"))
            }
            _ => None,
        })
        .collect();
    json!({
        "callback_installed": true,
        "actions_after_poll": actions,
    })
}

fn cursor_resources(glfw: &mut glfw::Glfw, window: &mut glfw::PWindow) -> Value {
    let shapes = [
        glfw::ffi::GLFW_ARROW_CURSOR,
        glfw::ffi::GLFW_IBEAM_CURSOR,
        glfw::ffi::GLFW_CROSSHAIR_CURSOR,
        glfw::ffi::GLFW_POINTING_HAND_CURSOR,
        glfw::ffi::GLFW_RESIZE_EW_CURSOR,
        glfw::ffi::GLFW_RESIZE_NS_CURSOR,
        glfw::ffi::GLFW_RESIZE_NWSE_CURSOR,
        glfw::ffi::GLFW_RESIZE_NESW_CURSOR,
        glfw::ffi::GLFW_RESIZE_ALL_CURSOR,
        glfw::ffi::GLFW_NOT_ALLOWED_CURSOR,
    ];
    let standard_cursors: Vec<*mut glfw::ffi::GLFWcursor> = shapes
        .into_iter()
        .map(|shape| {
            // SAFETY: GLFW is initialized on this thread and every value is a GLFW 3.4 standard
            // cursor constant. Each returned non-null handle is destroyed below.
            unsafe { glfw::ffi::glfwCreateStandardCursor(shape) }
        })
        .collect();
    if standard_cursors.iter().any(|cursor| cursor.is_null()) {
        destroy_cursors(&standard_cursors);
        return json!({"standard_created": false});
    }

    let mut pixels = vec![
        255_u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ];
    let image = glfw::ffi::GLFWimage {
        width: 2,
        height: 2,
        pixels: pixels.as_mut_ptr(),
    };
    // SAFETY: The 2x2 image owns exactly four packed RGBA8 pixels, its hotspot is in bounds, and
    // GLFW copies the image before returning. The returned non-null handle is destroyed below.
    let custom = unsafe { glfw::ffi::glfwCreateCursor(&raw const image, 1, 1) };
    if custom.is_null() {
        destroy_cursors(&standard_cursors);
        return json!({"standard_created": true, "custom_created": false});
    }

    glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    glfw.window_hint(WindowHint::Visible(false));
    let Some((second_window, _events)) =
        glfw.create_window(160, 120, "VMNL cursor probe", WindowMode::Windowed)
    else {
        destroy_cursor(custom);
        destroy_cursors(&standard_cursors);
        return json!({"second_window_created": false});
    };

    // SAFETY: Both windows and cursor handles are live on the GLFW thread. The shared standard
    // cursor remains alive until both windows have restored their default cursor.
    unsafe {
        glfw::ffi::glfwSetCursor(window.window_ptr(), standard_cursors[0]);
        glfw::ffi::glfwSetCursor(second_window.window_ptr(), standard_cursors[0]);
        glfw::ffi::glfwSetCursor(window.window_ptr(), custom);
        glfw::ffi::glfwSetCursor(window.window_ptr(), std::ptr::null_mut());
        glfw::ffi::glfwSetCursor(second_window.window_ptr(), std::ptr::null_mut());
    }

    destroy_cursor(custom);
    destroy_cursors(&standard_cursors);
    json!({
        "standard_created": standard_cursors.len(),
        "custom_created": true,
        "shared_between_windows": true,
        "replaced_and_removed": true,
    })
}

fn destroy_cursor(cursor: *mut glfw::ffi::GLFWcursor) {
    // SAFETY: The probe calls this exactly once for a non-null cursor it created, after removing
    // that cursor from every live window.
    unsafe {
        glfw::ffi::glfwDestroyCursor(cursor);
    }
}

fn destroy_cursors(cursors: &[*mut glfw::ffi::GLFWcursor]) {
    for &cursor in cursors {
        if !cursor.is_null() {
            destroy_cursor(cursor);
        }
    }
}

fn emit(
    requested: &str,
    actual: Option<glfw::Platform>,
    operation: &str,
    phase: &str,
    callbacks: &Rc<RefCell<Vec<Value>>>,
    value: &Value,
    result: &str,
) {
    let compiled = glfw::Version {
        major: glfw::ffi::GLFW_VERSION_MAJOR as u64,
        minor: glfw::ffi::GLFW_VERSION_MINOR as u64,
        patch: glfw::ffi::GLFW_VERSION_REVISION as u64,
    };
    let runtime = glfw::get_version();
    println!(
        "{}",
        json!({
            "schema": PROBE_SCHEMA_VERSION,
            "os": env::consts::OS,
            "backend_requested": requested,
            "backend_actual": actual.map(backend_name),
            "glfw_compiled": format!("{}.{}.{}", compiled.major, compiled.minor, compiled.patch),
            "glfw_runtime": format!("{}.{}.{}", runtime.major, runtime.minor, runtime.patch),
            "glfw_runtime_string": glfw::get_version_string(),
            "operation": operation,
            "phase": phase,
            "callbacks": callbacks.borrow().clone(),
            "value": value,
            "result": result,
        })
    );
}
