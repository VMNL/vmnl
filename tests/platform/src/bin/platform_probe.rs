// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! One-operation GLFW probe used by the platform integration tests.

#![allow(clippy::print_stderr, clippy::print_stdout, clippy::too_many_lines)]

use glfw::{ClientApiHint, InitHint, WindowHint, WindowMode};
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
    let Some((mut window, _events)) =
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
