// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Bounded native-case observer; expected values come from the parent fixture.

#[path = "../../input_case_contract.rs"]
mod input_case_contract;

use glfw::{Context as _, WindowEvent};
use serde_json::{json, Value};
use std::{
    env, fs,
    time::{Duration, Instant},
};

const DEADLINE: Duration = Duration::from_secs(5);

fn failure(stage: &str, reason: &impl ToString) -> Value {
    json!({"qualified": false, "stage": stage, "failure_reason": reason.to_string()})
}

pub(super) fn run(
    glfw: &mut glfw::Glfw,
    window: &mut glfw::PWindow,
    events: &glfw::GlfwReceiver<(f64, WindowEvent)>,
    platform: glfw::Platform,
) -> Value {
    let spec = match env::var("VMNL_PLATFORM_INPUT_CASE")
        .map_err(|e| e.to_string())
        .and_then(|s| serde_json::from_str::<Value>(&s).map_err(|e| e.to_string()))
    {
        Ok(spec) => spec,
        Err(error) => return failure("case", &error),
    };
    let Some(expected) = spec["expected"].as_array() else {
        return failure("case", &"expected event array missing");
    };
    let Some(ready) = env::var_os("VMNL_PLATFORM_READY_FILE") else {
        return failure("handshake", &"ready file missing");
    };
    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_char_polling(true);
    window.set_char_mods_polling(true);
    window.set_focus_polling(true);
    window.set_store_lock_key_mods(spec["lock_mods"] == true);
    window.show();
    if platform != glfw::Platform::Wayland {
        window.focus();
    }
    #[cfg(target_os = "linux")]
    let _buffer = if platform == glfw::Platform::Wayland {
        match super::map_wayland_probe(glfw, window) {
            Ok(buffer) => Some(buffer),
            Err(error) => return failure("mapping", &error),
        }
    } else {
        None
    };
    if platform == glfw::Platform::Wayland {
        if let Err(error) = fs::write(&ready, b"MAPPED\n") {
            return failure("handshake", &error);
        }
    }
    let deadline = Instant::now() + DEADLINE;
    while !window.is_focused() && Instant::now() < deadline {
        glfw.wait_events_timeout(0.01);
        glfw::flush_messages(events).for_each(drop);
    }
    if platform != glfw::Platform::Wayland {
        let (width, height) = window.get_size();
        window.set_cursor_pos(f64::from(width) / 2.0, f64::from(height) / 2.0);
    }
    while !(window.is_focused() && window.is_hovered()) && Instant::now() < deadline {
        glfw.wait_events_timeout(0.01);
        glfw::flush_messages(events).for_each(drop);
    }
    glfw.poll_events();
    glfw::flush_messages(events).for_each(drop);
    if !window.is_focused() || !window.is_hovered() {
        return failure("readiness", &"focused and hovered window required");
    }
    if let Err(error) = fs::write(&ready, b"READY\n") {
        return failure("handshake", &error);
    }

    let deadline = Instant::now() + DEADLINE;
    let mut observed = Vec::new();
    let mut pressed_snapshot = Value::Null;
    let mut minimized = false;
    // Wait for the parent's completion marker as well as events. A fixed settling interval
    // rejects extra callbacks, including delayed repeats, instead of accepting a matching prefix.
    let mut completed_at = None;
    let mut focus_observed_at = None;
    while Instant::now() < deadline {
        glfw.wait_events_timeout(0.01);
        for (_, event) in glfw::flush_messages(events) {
            let item = match event {
                WindowEvent::Key(key, scancode, action, modifiers) if spec["family"] != "text" => {
                    Some(json!({"kind":"key", "key":key as i32, "scancode":scancode,
                        "action":format!("{action:?}"), "modifiers":modifiers.bits()}))
                }
                WindowEvent::MouseButton(button, action, modifiers) if spec["family"] != "text" => {
                    Some(json!({"kind":"mouse", "button":button as i32,
                        "action":format!("{action:?}"), "modifiers":modifiers.bits()}))
                }
                WindowEvent::Char(ch) if spec["family"] == "text" => {
                    Some(json!({"kind":"char", "text":ch.to_string()}))
                }
                WindowEvent::CharModifiers(ch, mods) if spec["family"] == "text" => Some(
                    json!({"kind":"char-mods", "text":ch.to_string(), "modifiers":mods.bits()}),
                ),
                WindowEvent::Focus(false) => {
                    if let Err(error) = fs::write(
                        std::path::Path::new(&ready).with_extension("focus"),
                        b"LOST\n",
                    ) {
                        return failure("handshake", &error);
                    }
                    Some(json!({"kind":"focus", "focused":false}))
                }
                _ => None,
            };
            if let Some(item) = item {
                observed.push(item);
            }
        }
        if spec["target_ack"] == true
            && observed
                .iter()
                .filter(|e| {
                    e["kind"] == "mouse"
                        && e["button"] == spec["mouse_button"]
                        && matches!(e["action"].as_str(), Some("Press" | "Release"))
                })
                .count()
                >= 2
        {
            if let Err(error) = fs::write(
                std::path::Path::new(&ready).with_extension("target"),
                b"TARGET\n",
            ) {
                return failure("handshake", &error);
            }
        }
        if spec["family"] == "focus-loss" && !minimized && observed.len() >= 2 {
            pressed_snapshot = states(window, &spec);
            window.iconify();
            minimized = true;
        }
        if spec["family"] == "focus-loss"
            && observed.len() >= expected.len()
            && !window.is_focused()
        {
            let at = *focus_observed_at.get_or_insert_with(Instant::now);
            if at.elapsed() >= Duration::from_millis(100) {
                // Freeze the focus-loss trace before authorizing physical cleanup. Cocoa reports
                // an additional mouse-up for cleanup even after GLFW synthesized its release.
                if let Err(error) = fs::write(
                    std::path::Path::new(&ready).with_extension("observed"),
                    b"OBSERVED\n",
                ) {
                    return failure("handshake", &error);
                }
                while Instant::now() < deadline {
                    if fs::read_to_string(std::path::Path::new(&ready).with_extension("done"))
                        .is_ok_and(|s| s == "DONE\n")
                    {
                        completed_at = Some(Instant::now());
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                break;
            }
        }
        if fs::read_to_string(std::path::Path::new(&ready).with_extension("done"))
            .is_ok_and(|s| s == "DONE\n")
            && completed_at.is_none()
        {
            completed_at = Some(Instant::now());
        }
        if completed_at.is_some_and(|at| at.elapsed() >= Duration::from_millis(100))
            && observed.len() >= expected.len()
        {
            break;
        }
    }
    let final_states = states(window, &spec);
    let events_match = input_case_contract::matches_events(&spec, &observed)
        && input_case_contract::lock_events_match(&spec, &observed);
    let released = final_states
        .as_array()
        .is_some_and(|s| s.iter().all(|v| v["state"] == "Release"));
    let focus_match = if spec["family"] == "focus-loss" {
        !window.is_focused()
            && pressed_snapshot
                .as_array()
                .is_some_and(|s| s.len() == 2 && s.iter().all(|v| v["state"] == "Press"))
    } else {
        window.is_focused()
    };
    let qualified = completed_at.is_some() && events_match && released && focus_match;
    json!({"qualified":qualified, "stage":"input", "case":spec["case"],
        "family":spec["family"], "expected_events":expected, "observed_events":observed,
        "focused":window.is_focused(), "pressed_snapshot":pressed_snapshot,
        "final_states":final_states, "injector":env::var("VMNL_PLATFORM_INPUT_INJECTOR").ok(),
        "failure_reason":if qualified {Value::Null} else {
            json!("missing/extra/wrong events, deadline, focus or released-state mismatch")}})
}

fn states(window: &glfw::PWindow, spec: &Value) -> Value {
    let mut values = Vec::new();
    if let Some(keys) = spec["keys"].as_array() {
        for key in keys {
            // The case fixture contains only named GLFW keys. Check both range and integer
            // representation before the FFI call; GLFW accepts every key in this inclusive range.
            if let Some(raw) = key
                .as_i64()
                .and_then(|n| i32::try_from(n).ok())
                .filter(|n| (32..=348).contains(n))
            {
                // SAFETY: The live window belongs to this GLFW thread; `raw` is a valid key range.
                let state = unsafe { glfw::ffi::glfwGetKey(window.window_ptr(), raw) };
                values.push(json!({"key":raw,"state":if state==0 {"Release"} else {"Press"}}));
            }
        }
    }
    if spec["mouse"] == true {
        let raw = spec["mouse_button"]
            .as_i64()
            .and_then(|n| i32::try_from(n).ok())
            .unwrap_or(0);
        if let Some(button) = glfw::MouseButton::from_i32(raw) {
            values.push(
                json!({"button":raw,"state":format!("{:?}",window.get_mouse_button(button))}),
            );
        }
    }
    json!(values)
}
