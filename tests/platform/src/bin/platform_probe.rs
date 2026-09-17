// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! One-operation GLFW probe used by the platform integration tests.

#![allow(clippy::print_stderr, clippy::print_stdout, clippy::too_many_lines)]

use glfw::{ClientApiHint, Context as _, InitHint, WindowHint, WindowMode};
use serde_json::{json, Value};
use std::{cell::RefCell, env, process::ExitCode, rc::Rc};
use vmnl_platform_tests::{backend_name, parse_backend, PROBE_SCHEMA_VERSION};

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
    glfw.window_hint(WindowHint::Visible(false));
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

    emit(
        &requested_name,
        Some(actual),
        &operation,
        "operation",
        &callbacks,
        &value,
        "ok",
    );
    ExitCode::SUCCESS
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
