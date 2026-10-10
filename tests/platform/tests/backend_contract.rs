// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Display-server contracts selected explicitly by the platform recipes and CI.

#![allow(clippy::expect_used, clippy::panic)]

#[cfg(target_os = "linux")]
use serde_json::json;
use serde_json::Value;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

const NATIVE_INPUT_TIMEOUT: Duration = Duration::from_secs(7);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);
static READY_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(target_os = "linux")]
fn is_mouse_native_input(operation: &str) -> bool {
    matches!(
        operation,
        "mouse-native-input"
            | "mouse-buttons-input"
            | "mouse-motion-input"
            | "mouse-hover-boundary-input"
            | "mouse-scroll-input"
    )
}

fn probe(backend: &str, operation: &str) -> Value {
    let output = if matches!(
        operation,
        "keyboard-native-input"
            | "keyboard-modifier-input"
            | "mouse-native-input"
            | "mouse-buttons-input"
            | "mouse-motion-input"
            | "mouse-hover-boundary-input"
            | "mouse-scroll-input"
    ) {
        native_input_probe(backend, operation).expect("native input probe should complete")
    } else {
        Command::new(env!("CARGO_BIN_EXE_platform_probe"))
            .args([backend, operation])
            .output()
            .expect("platform probe should start")
    };
    assert!(
        !output.stdout.is_empty(),
        "{backend}/{operation} emitted no JSON"
    );
    if let Some(directory) = env::var_os("VMNL_PLATFORM_ARTIFACT_DIR") {
        let directory = PathBuf::from(directory);
        fs::create_dir_all(&directory).expect("platform artifact directory should be created");
        let mut artifact = OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join(format!("{backend}.jsonl")))
            .expect("platform artifact should open");
        artifact
            .write_all(&output.stdout)
            .expect("platform artifact should be written");
    }
    assert!(
        output.status.success(),
        "{backend}/{operation} failed or aborted: status={:?}, stdout={}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("platform probe should emit one JSON record")
}

fn native_input_probe(backend: &str, operation: &str) -> Result<Output, String> {
    let sequence = READY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let max_server_button =
        if operation == "mouse-buttons-input" && matches!(backend, "x11" | "wayland") {
            Some(x11_pointer_button_count()?)
        } else {
            None
        };
    let ready_file = env::temp_dir().join(format!(
        "vmnl-platform-ready-{}-{sequence}",
        std::process::id()
    ));
    let injector = if backend == "win32" && operation == "mouse-hover-boundary-input" {
        "set-cursor-pos"
    } else {
        input_injector_name(backend)?
    };
    let result = (|| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_platform_probe"));
        command
            .args([backend, operation])
            .env("VMNL_PLATFORM_READY_FILE", &ready_file)
            .env("VMNL_PLATFORM_INPUT_INJECTOR", injector)
            .env(
                "VMNL_PLATFORM_X11_MAX_BUTTON",
                max_server_button.map_or_else(String::new, |value| value.to_string()),
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if backend == "wayland" && operation == "mouse-hover-boundary-input" {
            // Expose whether the XTEST-to-Weston path sends wl_pointer enter/leave events.
            command.env("WAYLAND_DEBUG", "1");
        }
        let mut child = command
            .spawn()
            .map_err(|error| format!("platform probe should start: {error}"))?;

        if let Err(reason) = wait_until_ready(&mut child, &ready_file, backend, operation) {
            let reason = append_external_x11_focus_diagnostic(backend, &reason);
            return Err(terminate_with_diagnostics(child, &reason));
        }
        log_external_x11_focus_at_ready(backend);
        let injection = match operation {
            "keyboard-native-input" => inject_key_a(),
            "keyboard-modifier-input" => inject_key_shift_a(),
            "mouse-native-input" => inject_mouse_left(),
            "mouse-buttons-input" => {
                let button_limit = match backend {
                    "x11" | "wayland" => usize::from(max_server_button.unwrap_or_default()),
                    "win32" => 5,
                    "cocoa" => 8,
                    value => {
                        return Err(format!("mouse-button injection is unsupported for {value}"))
                    }
                };
                inject_mouse_buttons(button_limit)
            }
            "mouse-motion-input" => inject_mouse_motion(),
            "mouse-hover-boundary-input" => inject_mouse_hover_boundary(backend),
            "mouse-scroll-input" => inject_mouse_scroll(),
            value => Err(format!("unsupported native input operation: {value}")),
        };
        if let Err(reason) = injection {
            return Err(terminate_with_diagnostics(child, &reason));
        }

        wait_for_probe(child)
    })();
    let _ = fs::remove_file(ready_file);
    result
}

#[cfg(target_os = "linux")]
fn append_external_x11_focus_diagnostic(backend: &str, reason: &str) -> String {
    if backend != "wayland" {
        return reason.to_owned();
    }

    let diagnostic = measure_external_x11_focus().unwrap_or_else(|error| {
        json!({
            "measurement_error": error,
        })
    });
    format!("{reason}; external_x11_focus={diagnostic}")
}

#[cfg(not(target_os = "linux"))]
fn append_external_x11_focus_diagnostic(_backend: &str, reason: &str) -> String {
    reason.to_owned()
}

#[cfg(target_os = "linux")]
#[allow(clippy::print_stderr)]
fn log_external_x11_focus_at_ready(backend: &str) {
    if backend == "wayland" {
        eprintln!(
            "focus_at_ready={}",
            append_external_x11_focus_diagnostic(backend, "glfw_focus_confirmed_by_ready=true")
        );
    }
}

#[cfg(not(target_os = "linux"))]
fn log_external_x11_focus_at_ready(_backend: &str) {}

#[cfg(target_os = "linux")]
fn measure_external_x11_focus() -> Result<Value, String> {
    use x11rb::{connection::Connection as _, protocol::xproto::ConnectionExt as _};

    let (connection, screen_number) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    let screen = connection
        .setup()
        .roots
        .get(screen_number)
        .ok_or_else(|| format!("parent X server has no screen {screen_number}"))?;
    let root = screen.root;
    let focus = connection
        .get_input_focus()
        .map_err(|error| format!("failed to request parent X11 input focus: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read parent X11 input focus: {error}"))?;
    let pointer = connection
        .query_pointer(root)
        .map_err(|error| format!("failed to request parent X11 pointer position: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read parent X11 pointer position: {error}"))?;
    let tree = connection
        .query_tree(root)
        .map_err(|error| format!("failed to request parent X11 root tree: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read parent X11 root tree: {error}"))?;

    let root_children: Vec<Value> = tree
        .children
        .iter()
        .take(16)
        .map(|&window| {
            json!({
                "id": x11_window_id(window),
                "wm_name": x11_text_property(
                    &connection,
                    window,
                    x11rb::protocol::xproto::AtomEnum::WM_NAME,
                ),
                "wm_class": x11_text_property(
                    &connection,
                    window,
                    x11rb::protocol::xproto::AtomEnum::WM_CLASS,
                ),
            })
        })
        .collect();

    Ok(json!({
        "display": env::var("DISPLAY").ok(),
        "screen": screen_number,
        "root": x11_window_id(root),
        "focus": {
            "id": x11_window_id(focus.focus),
            "kind": x11_focus_target_kind(focus.focus, root),
            "revert_to": x11_revert_to_name(focus.revert_to),
            "revert_to_raw": u8::from(focus.revert_to),
        },
        "pointer": {
            "child": x11_window_id(pointer.child),
            "root_x": pointer.root_x,
            "root_y": pointer.root_y,
            "same_screen": pointer.same_screen,
        },
        "root_child_count": tree.children.len(),
        "root_children": root_children,
        "root_children_truncated": tree.children.len() > 16,
    }))
}

#[cfg(target_os = "linux")]
fn x11_text_property(
    connection: &x11rb::rust_connection::RustConnection,
    window: u32,
    property: x11rb::protocol::xproto::AtomEnum,
) -> Value {
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _};

    let reply = match connection.get_property(false, window, property, AtomEnum::STRING, 0, 256) {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply,
            Err(error) => return json!({ "error": error.to_string() }),
        },
        Err(error) => return json!({ "error": error.to_string() }),
    };
    if reply.value.is_empty() {
        return Value::Null;
    }

    let fields: Vec<String> = reply
        .value
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .map(|field| String::from_utf8_lossy(field).into_owned())
        .collect();
    json!(fields)
}

#[cfg(target_os = "linux")]
fn x11_window_id(window: u32) -> String {
    format!("0x{window:08x}")
}

#[cfg(target_os = "linux")]
fn x11_focus_target_kind(focus: u32, root: u32) -> &'static str {
    match focus {
        0 => "none",
        1 => "pointer-root",
        value if value == root => "root",
        _ => "window",
    }
}

#[cfg(target_os = "linux")]
fn x11_revert_to_name(revert_to: x11rb::protocol::xproto::InputFocus) -> &'static str {
    use x11rb::protocol::xproto::InputFocus;

    match revert_to {
        InputFocus::NONE => "none",
        InputFocus::POINTER_ROOT => "pointer-root",
        InputFocus::PARENT => "parent",
        InputFocus::FOLLOW_KEYBOARD => "follow-keyboard",
        _ => "unknown",
    }
}

#[cfg(target_os = "linux")]
fn weston_x11_window(
    connection: &x11rb::rust_connection::RustConnection,
    root: u32,
) -> Result<Option<u32>, String> {
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _};

    let mut pending = vec![root];
    let mut weston = None;
    while let Some(parent) = pending.pop() {
        let cookie = match connection.query_tree(parent) {
            Ok(cookie) => cookie,
            Err(error) if parent == root => {
                return Err(format!("failed to query parent X11 windows: {error}"));
            }
            Err(_) => continue,
        };
        let tree = match cookie.reply() {
            Ok(tree) => tree,
            Err(error) if parent == root => {
                return Err(format!("failed to read parent X11 windows: {error}"));
            }
            Err(_) => continue,
        };
        for window in tree.children {
            let class = x11_text_property(connection, window, AtomEnum::WM_CLASS);
            let is_weston = class.as_array().is_some_and(|fields| {
                fields
                    .iter()
                    .any(|field| field.as_str() == Some("Weston Compositor"))
            });
            if is_weston && weston.replace(window).is_some() {
                return Err("multiple Weston X11 windows found".to_owned());
            }
            pending.push(window);
        }
    }
    Ok(weston)
}

#[cfg(target_os = "linux")]
fn weston_has_x11_focus() -> Result<bool, String> {
    use x11rb::{connection::Connection as _, protocol::xproto::ConnectionExt as _};

    let (connection, screen_number) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    let root = connection.setup().roots[screen_number].root;
    let Some(weston) = weston_x11_window(&connection, root)? else {
        return Ok(false);
    };
    let focus = connection
        .get_input_focus()
        .map_err(|error| format!("failed to query parent X11 focus: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read parent X11 focus: {error}"))?;
    Ok(focus.focus == weston)
}

#[cfg(target_os = "linux")]
fn activate_wayland_window() -> Result<(), String> {
    use x11rb::{
        connection::Connection as _,
        protocol::{
            xproto::{ConnectionExt as _, BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT},
            xtest::ConnectionExt as _,
        },
    };

    let (connection, screen_number) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    let root = connection.setup().roots[screen_number].root;
    let Some(weston) = weston_x11_window(&connection, root)? else {
        return Ok(());
    };

    let geometry = connection
        .get_geometry(weston)
        .map_err(|error| format!("failed to query Weston geometry: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read Weston geometry: {error}"))?;
    let center_x = i16::try_from(geometry.width / 2)
        .map_err(|_| "Weston window width is too large for X11 pointer coordinates".to_owned())?;
    let center_y = i16::try_from(geometry.height / 2)
        .map_err(|_| "Weston window height is too large for X11 pointer coordinates".to_owned())?;
    let position = connection
        .translate_coordinates(weston, root, center_x, center_y)
        .map_err(|error| format!("failed to translate Weston center: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read Weston center: {error}"))?;
    connection
        .xtest_get_version(2, 2)
        .map_err(|error| format!("failed to query XTEST: {error}"))?
        .reply()
        .map_err(|error| format!("XTEST is unavailable: {error}"))?;
    connection
        .warp_pointer(0u32, root, 0, 0, 0, 0, position.dst_x, position.dst_y)
        .map_err(|error| format!("failed to move pointer into Weston: {error}"))?
        .check()
        .map_err(|error| format!("X11 pointer move into Weston failed: {error}"))?;
    for event in [BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT] {
        connection
            .xtest_fake_input(event, 1, 0, root, position.dst_x, position.dst_y, 0)
            .map_err(|error| format!("failed to enqueue Weston activation click: {error}"))?
            .check()
            .map_err(|error| format!("Weston activation click failed: {error}"))?;
    }
    connection
        .flush()
        .map_err(|error| format!("failed to flush Weston activation click: {error}"))
}

fn wait_until_ready(
    child: &mut Child,
    ready_file: &Path,
    backend: &str,
    operation: &str,
) -> Result<(), String> {
    let deadline = Instant::now() + NATIVE_INPUT_TIMEOUT;
    #[cfg(not(target_os = "linux"))]
    let _ = (backend, operation);
    #[cfg(target_os = "linux")]
    let mut next_activation = Instant::now();
    #[cfg(target_os = "linux")]
    let mut mouse_activation_sent = false;
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to inspect platform probe: {error}"))?
        {
            return Err(format!("platform probe exited before READY with {status}"));
        }
        let signal = match fs::read_to_string(ready_file) {
            Ok(signal) => signal,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(format!("failed to read platform probe signal: {error}")),
        };
        if signal == "READY\n" {
            #[cfg(target_os = "linux")]
            if backend == "wayland" {
                if weston_has_x11_focus()? {
                    return Ok(());
                }
                if is_mouse_native_input(operation) {
                    return Err(
                        "Wayland probe is ready but Weston lacks parent X11 focus".to_owned()
                    );
                }
            } else {
                return Ok(());
            }
            #[cfg(not(target_os = "linux"))]
            return Ok(());
        }
        #[cfg(target_os = "linux")]
        if backend == "wayland" && Instant::now() >= next_activation {
            let activation_needed = match signal.as_str() {
                "MAPPED\n" if is_mouse_native_input(operation) => {
                    if mouse_activation_sent {
                        !weston_has_x11_focus()?
                    } else {
                        mouse_activation_sent = true;
                        true
                    }
                }
                "MAPPED\n" => true,
                "READY\n" => !is_mouse_native_input(operation),
                _ => false,
            };
            if activation_needed {
                activate_wayland_window()?;
            }
            next_activation = Instant::now() + Duration::from_millis(250);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "platform probe did not become injection-ready within 7 seconds; signal={signal:?}"
            ));
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

fn wait_for_probe(mut child: Child) -> Result<Output, String> {
    let deadline = Instant::now() + NATIVE_INPUT_TIMEOUT;
    loop {
        if child
            .try_wait()
            .map_err(|error| format!("failed to inspect platform probe: {error}"))?
            .is_some()
        {
            return child
                .wait_with_output()
                .map_err(|error| format!("failed to collect platform probe output: {error}"));
        }
        if Instant::now() >= deadline {
            return Err(terminate_with_diagnostics(
                child,
                "platform probe did not exit within 7 seconds after input injection",
            ));
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

fn terminate_with_diagnostics(mut child: Child, reason: &str) -> String {
    let _ = child.kill();
    match child.wait_with_output() {
        Ok(output) => format!(
            "{reason}; status={:?}, stdout={}, stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
        Err(error) => format!("{reason}; failed to collect platform probe output: {error}"),
    }
}

#[cfg(target_os = "linux")]
fn input_injector_name(backend: &str) -> Result<&'static str, String> {
    match backend {
        "x11" => Ok("x11-xtest"),
        "wayland" => Ok("x11-xtest-parent"),
        value => Err(format!("XTEST cannot inject the {value} backend")),
    }
}

#[cfg(target_os = "windows")]
fn input_injector_name(backend: &str) -> Result<&'static str, String> {
    (backend == "win32")
        .then_some("send-input")
        .ok_or_else(|| format!("SendInput cannot inject the {backend} backend"))
}

#[cfg(target_os = "macos")]
fn input_injector_name(backend: &str) -> Result<&'static str, String> {
    (backend == "cocoa")
        .then_some("cg-event-post")
        .ok_or_else(|| format!("CGEventPost cannot inject the {backend} backend"))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn input_injector_name(backend: &str) -> Result<&'static str, String> {
    Err(format!(
        "native input injection is unsupported for {backend} on this OS"
    ))
}

#[cfg(target_os = "linux")]
fn inject_key_sequence(shifted: bool) -> Result<(), String> {
    use x11rb::{
        connection::Connection as _,
        protocol::{
            xproto::{ConnectionExt as _, KEY_PRESS_EVENT, KEY_RELEASE_EVENT},
            xtest::ConnectionExt as _,
        },
    };

    const XK_A: u32 = 0x0041;
    const XK_A_LOWER: u32 = 0x0061;
    const XK_SHIFT_L: u32 = 0xffe1;

    let (connection, _) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    connection
        .xtest_get_version(2, 2)
        .map_err(|error| format!("failed to query XTEST: {error}"))?
        .reply()
        .map_err(|error| format!("XTEST is unavailable: {error}"))?;

    let setup = connection.setup();
    let keycode_count = u16::from(setup.max_keycode) - u16::from(setup.min_keycode) + 1;
    let keycode_count = u8::try_from(keycode_count)
        .map_err(|_| "X11 keycode range does not fit the protocol request".to_owned())?;
    let mapping = connection
        .get_keyboard_mapping(setup.min_keycode, keycode_count)
        .map_err(|error| format!("failed to request the X11 keyboard mapping: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read the X11 keyboard mapping: {error}"))?;
    let keysyms_per_keycode = usize::from(mapping.keysyms_per_keycode);
    if keysyms_per_keycode == 0 {
        return Err("X11 returned an empty keyboard mapping".to_owned());
    }
    let keycode_for_keysym = |keysym: u32, name: &str| -> Result<u8, String> {
        let keycode_offset = mapping
            .keysyms
            .chunks(keysyms_per_keycode)
            .position(|keysyms| keysyms.contains(&keysym))
            .ok_or_else(|| format!("X11 keyboard mapping has no {name} keysym"))?;
        let keycode = u16::from(setup.min_keycode)
            + u16::try_from(keycode_offset)
                .map_err(|_| format!("X11 {name} keycode offset is too large"))?;
        u8::try_from(keycode).map_err(|_| format!("X11 {name} keycode is invalid"))
    };
    let a_keycode = mapping
        .keysyms
        .chunks(keysyms_per_keycode)
        .position(|keysyms| keysyms.contains(&XK_A) || keysyms.contains(&XK_A_LOWER))
        .ok_or_else(|| "X11 keyboard mapping has no A keysym".to_owned())?;
    let a_keycode = u16::from(setup.min_keycode)
        + u16::try_from(a_keycode).map_err(|_| "X11 A keycode offset is too large".to_owned())?;
    let a_keycode = u8::try_from(a_keycode).map_err(|_| "X11 A keycode is invalid".to_owned())?;
    let shift_keycode = if shifted {
        Some(keycode_for_keysym(XK_SHIFT_L, "left Shift")?)
    } else {
        None
    };
    let mut sequence = Vec::with_capacity(if shifted { 4 } else { 2 });
    if let Some(shift_keycode) = shift_keycode {
        sequence.push((KEY_PRESS_EVENT, shift_keycode, "left Shift press"));
    }
    sequence.extend([
        (KEY_PRESS_EVENT, a_keycode, "A press"),
        (KEY_RELEASE_EVENT, a_keycode, "A release"),
    ]);
    if let Some(shift_keycode) = shift_keycode {
        sequence.push((KEY_RELEASE_EVENT, shift_keycode, "left Shift release"));
    }

    let fake_key = |event_type, keycode, name: &str| {
        connection
            .xtest_fake_input(event_type, keycode, 0, 0, 0, 0, 0)
            .map_err(|error| format!("failed to enqueue XTEST {name}: {error}"))?
            .check()
            .map_err(|error| format!("XTEST {name} failed: {error}"))
    };
    let mut pressed_keys = Vec::new();
    for (event_type, keycode, name) in sequence {
        if let Err(error) = fake_key(event_type, keycode, name) {
            let mut cleanup_errors = Vec::new();
            for pressed_key in pressed_keys.iter().rev().copied() {
                if let Err(cleanup_error) =
                    fake_key(KEY_RELEASE_EVENT, pressed_key, "held-key cleanup release")
                {
                    cleanup_errors.push(cleanup_error);
                }
            }
            let _ = connection.flush();
            return Err(if cleanup_errors.is_empty() {
                format!("{error}; held-key cleanup succeeded")
            } else {
                format!("{error}; held-key cleanup failed: {cleanup_errors:?}")
            });
        }
        if event_type == KEY_PRESS_EVENT {
            pressed_keys.push(keycode);
        } else {
            pressed_keys.retain(|pressed_key| *pressed_key != keycode);
        }
    }
    connection
        .flush()
        .map_err(|error| format!("failed to flush XTEST input: {error}"))
}

#[cfg(target_os = "linux")]
fn inject_key_a() -> Result<(), String> {
    inject_key_sequence(false)
}

#[cfg(target_os = "linux")]
fn inject_key_shift_a() -> Result<(), String> {
    inject_key_sequence(true)
}

#[cfg(target_os = "linux")]
fn inject_mouse_left() -> Result<(), String> {
    use x11rb::{
        connection::Connection as _,
        protocol::{xproto::BUTTON_PRESS_EVENT, xtest::ConnectionExt as _},
    };

    const BUTTON1: u8 = 1;

    let (connection, _) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    connection
        .xtest_get_version(2, 2)
        .map_err(|error| format!("failed to query XTEST: {error}"))?
        .reply()
        .map_err(|error| format!("XTEST is unavailable: {error}"))?;

    let press = connection
        .xtest_fake_input(BUTTON_PRESS_EVENT, BUTTON1, 0, 0, 0, 0, 0)
        .map_err(|error| format!("failed to enqueue XTEST left-button press: {error}"))?
        .check()
        .map_err(|error| format!("XTEST left-button press failed: {error}"));
    if let Err(error) = press {
        let _ = inject_x11_button_release(&connection, BUTTON1, "left-button");
        return Err(error);
    }

    if let Err(error) = inject_x11_button_release(&connection, BUTTON1, "left-button") {
        let cleanup = match inject_x11_button_release(&connection, BUTTON1, "left-button") {
            Ok(()) => "release retry succeeded".to_owned(),
            Err(retry) => retry,
        };
        return Err(format!("{error}; cleanup: {cleanup}"));
    }

    connection
        .flush()
        .map_err(|error| format!("failed to flush XTEST input: {error}"))
}

#[cfg(target_os = "linux")]
fn inject_mouse_buttons(max_server_button: usize) -> Result<(), String> {
    use x11rb::{
        connection::Connection as _,
        protocol::{xproto::BUTTON_PRESS_EVENT, xtest::ConnectionExt as _},
    };

    let (connection, _) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    connection
        .xtest_get_version(2, 2)
        .map_err(|error| format!("failed to query XTEST: {error}"))?
        .reply()
        .map_err(|error| format!("XTEST is unavailable: {error}"))?;

    for (button, name) in [
        (1, "GLFW button 1 (left)"),
        (2, "X11 button 2 / GLFW button 3 (middle)"),
        (3, "X11 button 3 / GLFW button 2 (right)"),
        (8, "GLFW button 4"),
        (9, "GLFW button 5"),
        (10, "GLFW button 6"),
        (11, "GLFW button 7"),
        (12, "GLFW button 8"),
    ]
    .into_iter()
    .filter(|(button, _)| usize::from(*button) <= max_server_button)
    {
        let press = connection
            .xtest_fake_input(BUTTON_PRESS_EVENT, button, 0, 0, 0, 0, 0)
            .map_err(|error| format!("failed to enqueue XTEST {name} press: {error}"))?
            .check()
            .map_err(|error| format!("XTEST {name} press failed: {error}"));
        if let Err(error) = press {
            let cleanup = inject_x11_button_release(&connection, button, name);
            return Err(match cleanup {
                Ok(()) => format!("{error}; cleanup release succeeded"),
                Err(cleanup) => format!("{error}; cleanup release failed: {cleanup}"),
            });
        }

        if let Err(error) = inject_x11_button_release(&connection, button, name) {
            let cleanup = inject_x11_button_release(&connection, button, name);
            return Err(match cleanup {
                Ok(()) => format!("{error}; cleanup release retry succeeded"),
                Err(cleanup) => format!("{error}; cleanup release retry failed: {cleanup}"),
            });
        }
    }

    connection
        .flush()
        .map_err(|error| format!("failed to flush XTEST mouse buttons: {error}"))
}

#[cfg(target_os = "linux")]
fn x11_pointer_button_count() -> Result<u8, String> {
    use x11rb::protocol::xproto::ConnectionExt as _;

    let (connection, _) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    let mapping = connection
        .get_pointer_mapping()
        .map_err(|error| format!("failed to request the X11 pointer mapping: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read the X11 pointer mapping: {error}"))?;
    u8::try_from(mapping.map.len())
        .map_err(|_| "X11 pointer button count does not fit u8".to_owned())
}

#[cfg(not(target_os = "linux"))]
fn x11_pointer_button_count() -> Result<u8, String> {
    Err("X11 pointer button count is available only on Linux".to_owned())
}

#[cfg(target_os = "linux")]
fn inject_mouse_scroll() -> Result<(), String> {
    use x11rb::{
        connection::Connection as _,
        protocol::{xproto::BUTTON_PRESS_EVENT, xtest::ConnectionExt as _},
    };

    let (connection, _) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    connection
        .xtest_get_version(2, 2)
        .map_err(|error| format!("failed to query XTEST: {error}"))?
        .reply()
        .map_err(|error| format!("XTEST is unavailable: {error}"))?;

    for (button, direction) in [
        (4, "vertical scroll up"),
        (5, "vertical scroll down"),
        (6, "horizontal scroll positive"),
        (7, "horizontal scroll negative"),
    ] {
        let name = direction;
        let press = connection
            .xtest_fake_input(BUTTON_PRESS_EVENT, button, 0, 0, 0, 0, 0)
            .map_err(|error| format!("failed to enqueue XTEST {name}: {error}"))?
            .check()
            .map_err(|error| format!("XTEST {name} failed: {error}"));
        if let Err(error) = press {
            let cleanup = inject_x11_button_release(&connection, button, name);
            return Err(match cleanup {
                Ok(()) => format!("{error}; cleanup release succeeded"),
                Err(cleanup) => format!("{error}; cleanup release failed: {cleanup}"),
            });
        }

        if let Err(error) = inject_x11_button_release(&connection, button, name) {
            let cleanup = inject_x11_button_release(&connection, button, name);
            return Err(match cleanup {
                Ok(()) => format!("{error}; cleanup release retry succeeded"),
                Err(cleanup) => format!("{error}; cleanup release retry failed: {cleanup}"),
            });
        }
    }

    connection
        .flush()
        .map_err(|error| format!("failed to flush XTEST mouse scroll: {error}"))
}

#[cfg(target_os = "linux")]
fn inject_mouse_motion() -> Result<(), String> {
    use x11rb::{
        connection::Connection as _,
        protocol::{
            xproto::{ConnectionExt as _, MOTION_NOTIFY_EVENT},
            xtest::ConnectionExt as _,
        },
    };

    const DELTA_X: i16 = 6;
    const DELTA_Y: i16 = 4;

    let (connection, screen_number) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    connection
        .xtest_get_version(2, 2)
        .map_err(|error| format!("failed to query XTEST: {error}"))?
        .reply()
        .map_err(|error| format!("XTEST is unavailable: {error}"))?;

    let root = connection
        .setup()
        .roots
        .get(screen_number)
        .ok_or_else(|| format!("parent X server has no screen {screen_number}"))?
        .root;
    let pointer = connection
        .query_pointer(root)
        .map_err(|error| format!("failed to request the parent pointer position: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read the parent pointer position: {error}"))?;
    let x = pointer
        .root_x
        .checked_add(DELTA_X)
        .ok_or_else(|| "target pointer X coordinate overflowed".to_owned())?;
    let y = pointer
        .root_y
        .checked_add(DELTA_Y)
        .ok_or_else(|| "target pointer Y coordinate overflowed".to_owned())?;
    connection
        .xtest_fake_input(MOTION_NOTIFY_EVENT, 0, 0, root, x, y, 0)
        .map_err(|error| format!("failed to enqueue XTEST pointer movement: {error}"))?
        .check()
        .map_err(|error| format!("XTEST pointer movement failed: {error}"))?;
    connection
        .flush()
        .map_err(|error| format!("failed to flush XTEST pointer movement: {error}"))
}

#[cfg(target_os = "linux")]
#[allow(clippy::print_stderr)]
fn inject_mouse_hover_boundary(backend: &str) -> Result<(), String> {
    use x11rb::{
        connection::Connection as _,
        protocol::{
            xproto::{ConnectionExt as _, MOTION_NOTIFY_EVENT},
            xtest::ConnectionExt as _,
        },
    };

    let (connection, screen_number) = x11rb::connect(None)
        .map_err(|error| format!("failed to connect to the parent X server: {error}"))?;
    connection
        .xtest_get_version(2, 2)
        .map_err(|error| format!("failed to query XTEST: {error}"))?
        .reply()
        .map_err(|error| format!("XTEST is unavailable: {error}"))?;

    let screen = connection
        .setup()
        .roots
        .get(screen_number)
        .ok_or_else(|| format!("parent X server has no screen {screen_number}"))?;
    let root = screen.root;
    let max_x = i16::try_from(screen.width_in_pixels.saturating_sub(1))
        .map_err(|_| "X11 screen width does not fit an XTEST coordinate".to_owned())?;
    let max_y = i16::try_from(screen.height_in_pixels.saturating_sub(1))
        .map_err(|_| "X11 screen height does not fit an XTEST coordinate".to_owned())?;
    let (outside_weston_bounds, weston_window) = if backend == "wayland" {
        let weston = weston_x11_bounds(&connection, root)?;
        (Some(weston.bounds), Some(weston.window))
    } else {
        (None, None)
    };
    let weston_ancestry = weston_window
        .map(|window| x11_window_ancestry(&connection, root, window))
        .transpose()?;
    let pointer = connection
        .query_pointer(root)
        .map_err(|error| format!("failed to request the parent pointer position: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read the parent pointer position: {error}"))?;
    let original = (pointer.root_x, pointer.root_y);
    let target = [(0, 0), (max_x, 0), (0, max_y), (max_x, max_y)]
        .into_iter()
        .filter(|(x, y)| match outside_weston_bounds {
            Some((left, top, right, bottom)) => {
                let (x, y) = (i32::from(*x), i32::from(*y));
                x < left || x > right || y < top || y > bottom
            }
            None => true,
        })
        .max_by_key(|(x, y)| {
            let dx = i64::from(*x) - i64::from(original.0);
            let dy = i64::from(*y) - i64::from(original.1);
            dx * dx + dy * dy
        })
        .ok_or_else(|| match outside_weston_bounds {
            Some(bounds) => {
                format!("no X11 screen corner lies outside the parent Weston window {bounds:?}")
            }
            None => "X11 screen has no pointer boundary target".to_owned(),
        })?;
    let move_to = |(x, y), label: &str| {
        connection
            .xtest_fake_input(MOTION_NOTIFY_EVENT, 0, 0, root, x, y, 0)
            .map_err(|error| format!("failed to enqueue XTEST {label}: {error}"))?
            .check()
            .map_err(|error| format!("XTEST {label} failed: {error}"))
    };
    move_to(target, "pointer leave")?;
    let leave_check = if backend == "wayland" {
        confirm_x11_pointer_target(&connection, root, target, "leave").map(Some)
    } else {
        Ok(None)
    };
    if backend == "wayland" {
        eprintln!(
            "wayland_hover_boundary target={target:?} original={original:?} weston_window={weston_window:?} weston_ancestry={weston_ancestry:?} weston_bounds={outside_weston_bounds:?} pointer_child_after_leave={:?}",
            leave_check.as_ref().ok().copied().flatten()
        );
        thread::sleep(Duration::from_millis(100));
    }
    if let Err(error) = move_to(original, "pointer re-entry") {
        let cleanup = move_to(original, "pointer re-entry cleanup");
        return Err(match cleanup {
            Ok(()) => format!("{error}; cleanup re-entry succeeded"),
            Err(cleanup) => format!("{error}; cleanup re-entry failed: {cleanup}"),
        });
    }
    if backend == "wayland" {
        let pointer_child = confirm_x11_pointer_target(&connection, root, original, "re-entry")?;
        let pointer_ancestry = x11_window_ancestry(&connection, root, pointer_child)?;
        eprintln!(
            "wayland_hover_boundary pointer_after_reentry={original:?} pointer_child_after_reentry={pointer_child:#x} pointer_ancestry={pointer_ancestry:?}"
        );
        leave_check?;
    }
    connection
        .flush()
        .map_err(|error| format!("failed to flush XTEST hover boundary movement: {error}"))
}

#[cfg(target_os = "linux")]
struct WestonWindowGeometry {
    window: u32,
    bounds: (i32, i32, i32, i32),
}

#[cfg(target_os = "linux")]
fn weston_x11_bounds(
    connection: &x11rb::rust_connection::RustConnection,
    root: u32,
) -> Result<WestonWindowGeometry, String> {
    use x11rb::protocol::xproto::ConnectionExt as _;

    let weston = weston_x11_window(connection, root)?
        .ok_or_else(|| "could not locate the parent Weston X11 window".to_owned())?;
    let geometry = connection
        .get_geometry(weston)
        .map_err(|error| format!("failed to request Weston window geometry: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read Weston window geometry: {error}"))?;
    let origin = connection
        .translate_coordinates(weston, root, 0, 0)
        .map_err(|error| format!("failed to request Weston screen position: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read Weston screen position: {error}"))?;
    let left = i32::from(origin.dst_x);
    let top = i32::from(origin.dst_y);
    Ok(WestonWindowGeometry {
        window: weston,
        bounds: (
            left,
            top,
            left + i32::from(geometry.width.saturating_sub(1)),
            top + i32::from(geometry.height.saturating_sub(1)),
        ),
    })
}

#[cfg(target_os = "linux")]
fn confirm_x11_pointer_target(
    connection: &x11rb::rust_connection::RustConnection,
    root: u32,
    expected: (i16, i16),
    phase: &str,
) -> Result<u32, String> {
    use x11rb::{connection::Connection as _, protocol::xproto::ConnectionExt as _};

    connection
        .flush()
        .map_err(|error| format!("failed to flush XTEST pointer {phase}: {error}"))?;
    let pointer = connection
        .query_pointer(root)
        .map_err(|error| format!("failed to request pointer after {phase}: {error}"))?
        .reply()
        .map_err(|error| format!("failed to read pointer after {phase}: {error}"))?;
    let actual = (pointer.root_x, pointer.root_y);
    if actual != expected {
        return Err(format!(
            "XTEST pointer {phase} targeted {expected:?}, but the parent pointer is at {actual:?}"
        ));
    }
    Ok(pointer.child)
}

#[cfg(target_os = "linux")]
fn x11_window_ancestry(
    connection: &x11rb::rust_connection::RustConnection,
    root: u32,
    window: u32,
) -> Result<Vec<u32>, String> {
    use x11rb::protocol::xproto::ConnectionExt as _;

    let mut ancestry = Vec::new();
    let mut current = window;
    for _ in 0..64 {
        ancestry.push(current);
        if current == root {
            return Ok(ancestry);
        }
        let tree = connection
            .query_tree(current)
            .map_err(|error| {
                format!("failed to request parent of X11 window {current:#x}: {error}")
            })?
            .reply()
            .map_err(|error| {
                format!("failed to read parent of X11 window {current:#x}: {error}")
            })?;
        if tree.parent == current {
            return Err(format!(
                "X11 window ancestry stopped at self-parented window {current:#x} before root {root:#x}"
            ));
        }
        current = tree.parent;
    }
    Err(format!(
        "X11 window ancestry from {window:#x} exceeded 64 levels before root {root:#x}"
    ))
}

#[cfg(target_os = "linux")]
fn inject_x11_button_release(
    connection: &x11rb::rust_connection::RustConnection,
    button: u8,
    name: &str,
) -> Result<(), String> {
    use x11rb::protocol::{xproto::BUTTON_RELEASE_EVENT, xtest::ConnectionExt as _};

    connection
        .xtest_fake_input(BUTTON_RELEASE_EVENT, button, 0, 0, 0, 0, 0)
        .map_err(|error| format!("failed to enqueue XTEST {name} release: {error}"))?
        .check()
        .map_err(|error| format!("XTEST {name} release failed: {error}"))
}

#[cfg(target_os = "windows")]
fn inject_key_a() -> Result<(), String> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_A,
    };

    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_A,
                    ..KEYBDINPUT::default()
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_A,
                    dwFlags: KEYEVENTF_KEYUP,
                    ..KEYBDINPUT::default()
                },
            },
        },
    ];
    let input_size = i32::try_from(size_of::<INPUT>())
        .map_err(|_| "Win32 INPUT size does not fit i32".to_owned())?;
    // SAFETY: `inputs` contains two initialized keyboard INPUT records and remains alive for the
    // duration of the call. `input_size` is the exact size of one INPUT record.
    let sent = unsafe { SendInput(2, inputs.as_ptr(), input_size) };
    if sent == 2 {
        Ok(())
    } else {
        Err(format!(
            "SendInput inserted {sent}/2 events: {}",
            std::io::Error::last_os_error()
        ))
    }
}

#[cfg(target_os = "windows")]
fn inject_key_shift_a() -> Result<(), String> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_A, VK_LSHIFT,
    };

    let key = |virtual_key, key_up| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: virtual_key,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { 0 },
                ..KEYBDINPUT::default()
            },
        },
    };
    let inputs = [
        key(VK_LSHIFT, false),
        key(VK_A, false),
        key(VK_A, true),
        key(VK_LSHIFT, true),
    ];
    let input_size = i32::try_from(size_of::<INPUT>())
        .map_err(|_| "Win32 INPUT size does not fit i32".to_owned())?;
    // SAFETY: `inputs` contains four initialized keyboard INPUT records and remains alive for the
    // duration of the call. `input_size` is the exact size of one INPUT record.
    let sent = unsafe { SendInput(4, inputs.as_ptr(), input_size) };
    if sent == 4 {
        return Ok(());
    }

    let input_error = std::io::Error::last_os_error();
    let cleanup: Vec<INPUT> = match sent {
        1 => vec![key(VK_LSHIFT, true)],
        2 => vec![key(VK_A, true), key(VK_LSHIFT, true)],
        3 => vec![key(VK_LSHIFT, true)],
        _ => Vec::new(),
    };
    let cleanup_sent = if cleanup.is_empty() {
        0
    } else {
        // SAFETY: `cleanup` contains initialized key-up records and remains alive for the call.
        // Its length is bounded by two, which fits the Win32 `u32` record count.
        unsafe { SendInput(cleanup.len() as u32, cleanup.as_ptr(), input_size) }
    };
    Err(format!(
        "SendInput inserted {sent}/4 Shift+A events; cleanup inserted {cleanup_sent}/{} key-up events: {input_error}",
        cleanup.len()
    ))
}

#[cfg(target_os = "windows")]
fn inject_mouse_left() -> Result<(), String> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
        MOUSEINPUT,
    };

    let inputs = [
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: MOUSEEVENTF_LEFTDOWN,
                    ..MOUSEINPUT::default()
                },
            },
        },
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: MOUSEEVENTF_LEFTUP,
                    ..MOUSEINPUT::default()
                },
            },
        },
    ];
    let input_size = i32::try_from(size_of::<INPUT>())
        .map_err(|_| "Win32 INPUT size does not fit i32".to_owned())?;
    // SAFETY: `inputs` contains two initialized mouse INPUT records and remains alive for the
    // duration of the call. `input_size` is the exact size of one INPUT record.
    let sent = unsafe { SendInput(2, inputs.as_ptr(), input_size) };
    if sent == 2 {
        return Ok(());
    }

    let release = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: MOUSEEVENTF_LEFTUP,
                ..MOUSEINPUT::default()
            },
        },
    };
    // SAFETY: `release` is one initialized mouse INPUT record that remains alive for the call.
    let cleanup_sent = unsafe { SendInput(1, &release, input_size) };
    Err(format!(
        "SendInput inserted {sent}/2 mouse events; cleanup release inserted {cleanup_sent}/1: {}",
        std::io::Error::last_os_error()
    ))
}

#[cfg(target_os = "windows")]
fn inject_mouse_buttons(button_limit: usize) -> Result<(), String> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
        MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
        MOUSEINPUT,
    };

    const XBUTTON_DOWN: u32 = 0x0080;
    const XBUTTON_UP: u32 = 0x0100;
    let mappings = [
        (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, 0, "Button1"),
        (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, 0, "Button2"),
        (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, 0, "Button3"),
        (XBUTTON_DOWN, XBUTTON_UP, 1, "Button4 (XBUTTON1)"),
        (XBUTTON_DOWN, XBUTTON_UP, 2, "Button5 (XBUTTON2)"),
    ];
    let input_size = i32::try_from(size_of::<INPUT>())
        .map_err(|_| "Win32 INPUT size does not fit i32".to_owned())?;

    for (down_flags, up_flags, mouse_data, name) in mappings.into_iter().take(button_limit) {
        let inputs = [
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        mouseData: mouse_data,
                        dwFlags: down_flags,
                        ..MOUSEINPUT::default()
                    },
                },
            },
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        mouseData: mouse_data,
                        dwFlags: up_flags,
                        ..MOUSEINPUT::default()
                    },
                },
            },
        ];
        // SAFETY: `inputs` contains two initialized mouse INPUT records and remains alive for the
        // duration of the call. `input_size` is the exact size of one INPUT record.
        let sent = unsafe { SendInput(2, inputs.as_ptr(), input_size) };
        if sent != 2 {
            let injection_error = std::io::Error::last_os_error();
            let release = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        mouseData: mouse_data,
                        dwFlags: up_flags,
                        ..MOUSEINPUT::default()
                    },
                },
            };
            // SAFETY: `release` is one initialized mouse INPUT record that remains alive for the call.
            let cleanup_sent = unsafe { SendInput(1, &release, input_size) };
            return Err(format!(
                "SendInput inserted {sent}/2 {name} events; cleanup release inserted {cleanup_sent}/1: {injection_error}"
            ));
        }
    }

    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_mouse_buttons(_button_limit: usize) -> Result<(), String> {
    Err("native mouse-button injection is unsupported on this OS".to_owned())
}

#[cfg(target_os = "windows")]
fn send_mouse_input(
    dx: i32,
    dy: i32,
    mouse_data: u32,
    flags: u32,
    name: &str,
) -> Result<(), String> {
    use std::mem::size_of;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEINPUT,
    };

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: mouse_data,
                dwFlags: flags,
                ..MOUSEINPUT::default()
            },
        },
    };
    let input_size = i32::try_from(size_of::<INPUT>())
        .map_err(|_| "Win32 INPUT size does not fit i32".to_owned())?;
    // SAFETY: `input` is one initialized mouse INPUT record and remains alive for the call.
    let sent = unsafe { SendInput(1, &input, input_size) };
    if sent == 1 {
        Ok(())
    } else {
        Err(format!(
            "SendInput inserted {sent}/1 {name} event: {}",
            std::io::Error::last_os_error()
        ))
    }
}

#[cfg(target_os = "windows")]
fn inject_mouse_scroll() -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MOUSEEVENTF_HWHEEL, MOUSEEVENTF_WHEEL};

    let inputs = [
        (MOUSEEVENTF_WHEEL, 120, "vertical scroll up"),
        (MOUSEEVENTF_WHEEL, -120, "vertical scroll down"),
        // GLFW inverts WM_MOUSEHWHEEL's horizontal delta to match X11 and Cocoa.
        (MOUSEEVENTF_HWHEEL, -120, "horizontal scroll positive"),
        (MOUSEEVENTF_HWHEEL, 120, "horizontal scroll negative"),
    ];
    let input_count = inputs.len();
    for (index, (flags, delta, name)) in inputs.into_iter().enumerate() {
        send_mouse_input(0, 0, delta as u32, flags, name)?;
        if index + 1 < input_count {
            // Let GLFW's message pump handle each wheel message before the next reversal.
            thread::sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
#[allow(clippy::print_stderr)]
fn inject_mouse_motion() -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        MOUSEEVENTF_MOVE, MOUSEEVENTF_MOVE_NOCOALESCE,
    };

    let original = win32_cursor_position("before pointer movement")?;
    send_mouse_input(
        6,
        4,
        0,
        MOUSEEVENTF_MOVE | MOUSEEVENTF_MOVE_NOCOALESCE,
        "pointer movement",
    )?;

    let deadline = Instant::now() + Duration::from_millis(250);
    let mut actual = original;
    while Instant::now() < deadline {
        actual = win32_cursor_position("after pointer movement")?;
        if actual != original {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    eprintln!("win32_mouse_motion original={original:?} actual={actual:?}");
    Ok(())
}

#[cfg(target_os = "windows")]
fn inject_mouse_hover_boundary(_backend: &str) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::POINT,
        UI::WindowsAndMessaging::{
            GetCursorPos, GetSystemMetrics, SetCursorPos, SM_CXSCREEN, SM_CYSCREEN,
        },
    };

    let mut original = POINT { x: 0, y: 0 };
    // SAFETY: `original` is writable storage for one Win32 POINT record.
    if unsafe { GetCursorPos(&raw mut original) } == 0 {
        return Err(format!(
            "GetCursorPos failed before hover-boundary injection: {}",
            std::io::Error::last_os_error()
        ));
    }
    // SAFETY: These indices request the primary screen dimensions and take no pointers.
    let (width, height) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    if width <= 0 || height <= 0 {
        return Err(format!(
            "invalid Win32 primary screen size: {width}x{height}"
        ));
    }
    let target = [
        (0, 0),
        (width - 1, 0),
        (0, height - 1),
        (width - 1, height - 1),
    ]
    .into_iter()
    .max_by_key(|(x, y)| {
        let dx = i64::from(*x) - i64::from(original.x);
        let dy = i64::from(*y) - i64::from(original.y);
        dx * dx + dy * dy
    })
    .ok_or_else(|| "Win32 screen has no pointer boundary target".to_owned())?;
    // SAFETY: The target is inside the primary display bounds queried above.
    if unsafe { SetCursorPos(target.0, target.1) } == 0 {
        return Err(format!(
            "SetCursorPos failed while leaving the probe window: {}",
            std::io::Error::last_os_error()
        ));
    }
    let leave_check = confirm_win32_pointer_position(target, "leave");
    thread::sleep(Duration::from_millis(100));
    // SAFETY: `original` was returned by GetCursorPos and restores the pointer to the probe.
    if unsafe { SetCursorPos(original.x, original.y) } == 0 {
        return Err(format!(
            "SetCursorPos failed while re-entering the probe window: {}",
            std::io::Error::last_os_error()
        ));
    }
    confirm_win32_pointer_position((original.x, original.y), "re-entry")?;
    leave_check
}

#[cfg(target_os = "windows")]
fn confirm_win32_pointer_position(expected: (i32, i32), phase: &str) -> Result<(), String> {
    let actual = win32_cursor_position(phase)?;
    if actual != expected {
        return Err(format!(
            "pointer {phase} targeted {expected:?}, but Win32 reports {actual:?}"
        ));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn win32_cursor_position(phase: &str) -> Result<(i32, i32), String> {
    use windows_sys::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

    let mut actual = POINT { x: 0, y: 0 };
    // SAFETY: `actual` is writable storage for one Win32 POINT record.
    if unsafe { GetCursorPos(&raw mut actual) } == 0 {
        return Err(format!(
            "GetCursorPos failed {phase}: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok((actual.x, actual.y))
}

#[cfg(target_os = "macos")]
fn inject_mouse_scroll() -> Result<(), String> {
    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    const CG_HID_EVENT_TAP: u32 = 0;
    const CG_SCROLL_EVENT_UNIT_PIXEL: u32 = 0;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreateScrollWheelEvent(
            source: *mut c_void,
            units: u32,
            wheel_count: u32,
            wheel1: i32,
            wheel2: i32,
            wheel3: i32,
        ) -> CGEventRef;
        fn CGEventPost(tap: u32, event: CGEventRef);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }

    let inputs = [
        (1, 10, 0, "vertical scroll up"),
        (1, -10, 0, "vertical scroll down"),
        (2, 0, 10, "horizontal scroll positive"),
        (2, 0, -10, "horizontal scroll negative"),
    ];
    let input_count = inputs.len();
    for (index, (wheel_count, vertical, horizontal, name)) in inputs.into_iter().enumerate() {
        // SAFETY: Axis 1 is vertical and axis 2 is horizontal. Ten pixels normalize to one GLFW
        // scroll unit for precise events; the retained event is checked before posting or release.
        let event = unsafe {
            CGEventCreateScrollWheelEvent(
                std::ptr::null_mut(),
                CG_SCROLL_EVENT_UNIT_PIXEL,
                wheel_count,
                vertical,
                horizontal,
                0,
            )
        };
        if event.is_null() {
            return Err(format!(
                "CGEventCreateScrollWheelEvent returned null for {name}"
            ));
        }
        // SAFETY: `event` is a live retained CoreGraphics object created above.
        unsafe {
            CGEventPost(CG_HID_EVENT_TAP, event);
            CFRelease(event);
        }
        if index + 1 < input_count {
            thread::sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn inject_mouse_motion() -> Result<(), String> {
    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    const CG_HID_EVENT_TAP: u32 = 0;
    const MOUSE_MOVED: u32 = 5;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreate(source: *mut c_void) -> CGEventRef;
        fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
        fn CGEventCreateMouseEvent(
            source: *mut c_void,
            mouse_type: u32,
            mouse_cursor_position: CGPoint,
            mouse_button: u32,
        ) -> CGEventRef;
        fn CGEventPost(tap: u32, event: CGEventRef);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }

    // SAFETY: A null source requests the default event source. The returned event is checked
    // before its location is queried or its retained CoreFoundation reference is released.
    let position_event = unsafe { CGEventCreate(std::ptr::null_mut()) };
    if position_event.is_null() {
        return Err("CGEventCreate returned null while reading cursor position".to_owned());
    }
    // SAFETY: `position_event` is a live CoreGraphics event created above.
    let position = unsafe { CGEventGetLocation(position_event) };
    // SAFETY: `position_event` is a live retained CoreFoundation object.
    unsafe { CFRelease(position_event) };

    let target = CGPoint {
        x: position.x + 6.0,
        y: position.y + 4.0,
    };
    // SAFETY: MOUSE_MOVED is a valid mouse event type and target is a finite point. The returned
    // retained reference is checked before posting or release.
    let event = unsafe { CGEventCreateMouseEvent(std::ptr::null_mut(), MOUSE_MOVED, target, 0) };
    if event.is_null() {
        return Err("CGEventCreateMouseEvent returned null for pointer movement".to_owned());
    }
    // SAFETY: `event` is a live retained CoreGraphics object created above.
    unsafe {
        CGEventPost(CG_HID_EVENT_TAP, event);
        CFRelease(event);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn inject_mouse_hover_boundary(_backend: &str) -> Result<(), String> {
    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    const CG_HID_EVENT_TAP: u32 = 0;
    const MOUSE_MOVED: u32 = 5;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreate(source: *mut c_void) -> CGEventRef;
        fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
        fn CGEventCreateMouseEvent(
            source: *mut c_void,
            mouse_type: u32,
            mouse_cursor_position: CGPoint,
            mouse_button: u32,
        ) -> CGEventRef;
        fn CGEventPost(tap: u32, event: CGEventRef);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }

    // SAFETY: A null source requests the default event source. The returned event is checked
    // before its location is queried or its retained CoreFoundation reference is released.
    let position_event = unsafe { CGEventCreate(std::ptr::null_mut()) };
    if position_event.is_null() {
        return Err("CGEventCreate returned null while reading cursor position".to_owned());
    }
    // SAFETY: `position_event` is a live CoreGraphics event created above.
    let original = unsafe { CGEventGetLocation(position_event) };
    // SAFETY: `position_event` is a live retained CoreFoundation object.
    unsafe { CFRelease(position_event) };
    let outside = CGPoint {
        x: original.x + 300.0,
        y: original.y + 300.0,
    };

    // SAFETY: MOUSE_MOVED is a valid event type; both target points are finite. The returned
    // retained references are checked before either event is posted.
    let (leave, reenter) = unsafe {
        (
            CGEventCreateMouseEvent(std::ptr::null_mut(), MOUSE_MOVED, outside, 0),
            CGEventCreateMouseEvent(std::ptr::null_mut(), MOUSE_MOVED, original, 0),
        )
    };
    if leave.is_null() || reenter.is_null() {
        // SAFETY: Any non-null event was created above with a retained CoreFoundation reference.
        unsafe {
            if !leave.is_null() {
                CFRelease(leave);
            }
            if !reenter.is_null() {
                CFRelease(reenter);
            }
        }
        return Err("CGEventCreateMouseEvent returned null for hover boundary".to_owned());
    }
    // SAFETY: Both mouse-move references are live and retained until after they are posted.
    unsafe {
        CGEventPost(CG_HID_EVENT_TAP, leave);
    }
    thread::sleep(Duration::from_millis(100));
    // SAFETY: Both mouse-move references remain live and retained until after they are posted.
    unsafe {
        CGEventPost(CG_HID_EVENT_TAP, reenter);
        CFRelease(leave);
        CFRelease(reenter);
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_mouse_scroll() -> Result<(), String> {
    Err("native mouse scroll injection is unsupported on this OS".to_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_mouse_motion() -> Result<(), String> {
    Err("native pointer movement injection is unsupported on this OS".to_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_mouse_hover_boundary(_backend: &str) -> Result<(), String> {
    Err("native pointer hover-boundary injection is unsupported on this OS".to_owned())
}

#[cfg(target_os = "macos")]
fn inject_key_a() -> Result<(), String> {
    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    const CG_HID_EVENT_TAP: u32 = 0;
    const ANSI_A_KEYCODE: u16 = 0;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreateKeyboardEvent(
            source: *mut c_void,
            virtual_key: u16,
            key_down: bool,
        ) -> CGEventRef;
        fn CGEventPost(tap: u32, event: CGEventRef);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }

    // SAFETY: A null source requests the default CoreGraphics event source. Keycode zero is the
    // documented ANSI A hardware keycode, and both returned references are checked before use.
    let (press, release) = unsafe {
        (
            CGEventCreateKeyboardEvent(std::ptr::null_mut(), ANSI_A_KEYCODE, true),
            CGEventCreateKeyboardEvent(std::ptr::null_mut(), ANSI_A_KEYCODE, false),
        )
    };
    if press.is_null() || release.is_null() {
        // SAFETY: Any non-null value was created above with a retained CoreFoundation reference.
        unsafe {
            if !press.is_null() {
                CFRelease(press);
            }
            if !release.is_null() {
                CFRelease(release);
            }
        }
        return Err("CGEventCreateKeyboardEvent returned null".to_owned());
    }

    // SAFETY: Both event references are valid and retained until after they are posted.
    unsafe {
        CGEventPost(CG_HID_EVENT_TAP, press);
        CGEventPost(CG_HID_EVENT_TAP, release);
        CFRelease(press);
        CFRelease(release);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn inject_key_shift_a() -> Result<(), String> {
    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    const CG_HID_EVENT_TAP: u32 = 0;
    const ANSI_A_KEYCODE: u16 = 0;
    const LEFT_SHIFT_KEYCODE: u16 = 56;
    // Core Graphics kCGEventFlagMaskShift.
    const CG_EVENT_FLAG_MASK_SHIFT: u64 = 0x0002_0000;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreateKeyboardEvent(
            source: *mut c_void,
            virtual_key: u16,
            key_down: bool,
        ) -> CGEventRef;
        fn CGEventSetFlags(event: CGEventRef, flags: u64);
        fn CGEventPost(tap: u32, event: CGEventRef);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }

    let sequence = [
        (
            LEFT_SHIFT_KEYCODE,
            true,
            CG_EVENT_FLAG_MASK_SHIFT,
            "left Shift press",
        ),
        (ANSI_A_KEYCODE, true, CG_EVENT_FLAG_MASK_SHIFT, "A press"),
        (ANSI_A_KEYCODE, false, CG_EVENT_FLAG_MASK_SHIFT, "A release"),
        (LEFT_SHIFT_KEYCODE, false, 0, "left Shift release"),
    ];
    let mut events: Vec<CGEventRef> = Vec::with_capacity(sequence.len());
    for (keycode, key_down, flags, name) in sequence {
        // SAFETY: A null source requests the default event source; keycodes and down states are
        // valid CoreGraphics keyboard event parameters.
        let event = unsafe { CGEventCreateKeyboardEvent(std::ptr::null_mut(), keycode, key_down) };
        if event.is_null() {
            // SAFETY: Previously created events are live retained CoreFoundation references.
            unsafe {
                for event in events {
                    CFRelease(event);
                }
            }
            return Err(format!(
                "CGEventCreateKeyboardEvent returned null for {name}"
            ));
        }
        // SAFETY: `event` is a live keyboard event; the flag mask represents its modifier state.
        unsafe { CGEventSetFlags(event, flags) };
        events.push(event);
    }

    // SAFETY: Each event is a valid retained keyboard event and stays live until after posting.
    unsafe {
        for event in events {
            CGEventPost(CG_HID_EVENT_TAP, event);
            CFRelease(event);
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn inject_mouse_left() -> Result<(), String> {
    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    const CG_HID_EVENT_TAP: u32 = 0;
    const CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
    const CG_EVENT_LEFT_MOUSE_UP: u32 = 2;
    const CG_MOUSE_BUTTON_LEFT: u32 = 0;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreate(source: *mut c_void) -> CGEventRef;
        fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
        fn CGEventCreateMouseEvent(
            source: *mut c_void,
            mouse_type: u32,
            mouse_cursor_position: CGPoint,
            mouse_button: u32,
        ) -> CGEventRef;
        fn CGEventPost(tap: u32, event: CGEventRef);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }

    // SAFETY: A null source requests the default event source. The returned event is checked
    // before its location is queried or its retained CoreFoundation reference is released.
    let position_event = unsafe { CGEventCreate(std::ptr::null_mut()) };
    if position_event.is_null() {
        return Err("CGEventCreate returned null while reading cursor position".to_owned());
    }
    // SAFETY: `position_event` is a live CoreGraphics event created above.
    let position = unsafe { CGEventGetLocation(position_event) };
    // SAFETY: `position_event` is a live retained CoreFoundation object.
    unsafe { CFRelease(position_event) };

    // SAFETY: The location is the current global cursor position. Both returned references are
    // checked before posting and released after use.
    let (press, release) = unsafe {
        (
            CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                CG_EVENT_LEFT_MOUSE_DOWN,
                position,
                CG_MOUSE_BUTTON_LEFT,
            ),
            CGEventCreateMouseEvent(
                std::ptr::null_mut(),
                CG_EVENT_LEFT_MOUSE_UP,
                position,
                CG_MOUSE_BUTTON_LEFT,
            ),
        )
    };
    if press.is_null() || release.is_null() {
        // SAFETY: Any non-null value was created above with a retained CoreFoundation reference.
        unsafe {
            if !press.is_null() {
                CFRelease(press);
            }
            if !release.is_null() {
                CFRelease(release);
            }
        }
        return Err("CGEventCreateMouseEvent returned null".to_owned());
    }

    // SAFETY: Both mouse-event references are valid and retained until after they are posted.
    unsafe {
        CGEventPost(CG_HID_EVENT_TAP, press);
        CGEventPost(CG_HID_EVENT_TAP, release);
        CFRelease(press);
        CFRelease(release);
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn inject_mouse_buttons(button_limit: usize) -> Result<(), String> {
    use std::ffi::c_void;

    type CGEventRef = *mut c_void;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    const CG_HID_EVENT_TAP: u32 = 0;
    const LEFT_MOUSE_DOWN: u32 = 1;
    const LEFT_MOUSE_UP: u32 = 2;
    const RIGHT_MOUSE_DOWN: u32 = 3;
    const RIGHT_MOUSE_UP: u32 = 4;
    const OTHER_MOUSE_DOWN: u32 = 25;
    const OTHER_MOUSE_UP: u32 = 26;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreate(source: *mut c_void) -> CGEventRef;
        fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
        fn CGEventCreateMouseEvent(
            source: *mut c_void,
            mouse_type: u32,
            mouse_cursor_position: CGPoint,
            mouse_button: u32,
        ) -> CGEventRef;
        fn CGEventPost(tap: u32, event: CGEventRef);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }

    // SAFETY: A null source requests the default event source. The returned event is checked
    // before its location is queried or its retained CoreFoundation reference is released.
    let position_event = unsafe { CGEventCreate(std::ptr::null_mut()) };
    if position_event.is_null() {
        return Err("CGEventCreate returned null while reading cursor position".to_owned());
    }
    // SAFETY: `position_event` is a live CoreGraphics event created above.
    let position = unsafe { CGEventGetLocation(position_event) };
    // SAFETY: `position_event` is a live retained CoreFoundation object.
    unsafe { CFRelease(position_event) };

    let mappings = [
        (0, LEFT_MOUSE_DOWN, LEFT_MOUSE_UP, "Button1 (left)"),
        (1, RIGHT_MOUSE_DOWN, RIGHT_MOUSE_UP, "Button2 (right)"),
        (2, OTHER_MOUSE_DOWN, OTHER_MOUSE_UP, "Button3 (middle)"),
        (3, OTHER_MOUSE_DOWN, OTHER_MOUSE_UP, "Button4"),
        (4, OTHER_MOUSE_DOWN, OTHER_MOUSE_UP, "Button5"),
        (5, OTHER_MOUSE_DOWN, OTHER_MOUSE_UP, "Button6"),
        (6, OTHER_MOUSE_DOWN, OTHER_MOUSE_UP, "Button7"),
        (7, OTHER_MOUSE_DOWN, OTHER_MOUSE_UP, "Button8"),
    ];
    for (button, down_type, up_type, name) in mappings.into_iter().take(button_limit) {
        // SAFETY: The mouse button number and event types are valid CoreGraphics values. Both
        // returned references are checked before posting and released after use.
        let (press, release) = unsafe {
            (
                CGEventCreateMouseEvent(std::ptr::null_mut(), down_type, position, button),
                CGEventCreateMouseEvent(std::ptr::null_mut(), up_type, position, button),
            )
        };
        if press.is_null() || release.is_null() {
            // SAFETY: Any non-null value was created above with a retained CoreFoundation reference.
            unsafe {
                if !press.is_null() {
                    CFRelease(press);
                }
                if !release.is_null() {
                    CFRelease(release);
                }
            }
            return Err(format!("CGEventCreateMouseEvent returned null for {name}"));
        }

        // SAFETY: Both mouse-event references are valid and retained until after they are posted.
        unsafe {
            CGEventPost(CG_HID_EVENT_TAP, press);
            CGEventPost(CG_HID_EVENT_TAP, release);
            CFRelease(press);
            CFRelease(release);
        }
    }

    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_key_a() -> Result<(), String> {
    Err("native input injection is unsupported on this OS".to_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_key_shift_a() -> Result<(), String> {
    Err("native modifier-key injection is unsupported on this OS".to_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_mouse_left() -> Result<(), String> {
    Err("native mouse input injection is unsupported on this OS".to_owned())
}

#[test]
#[ignore = "requires VMNL_PLATFORM_TEST_BACKEND and a qualified native display server"]
fn selected_backend_contract() {
    let backend = env::var("VMNL_PLATFORM_TEST_BACKEND")
        .expect("VMNL_PLATFORM_TEST_BACKEND must name wayland or x11");
    let operations: &[&str] = match backend.as_str() {
        "wayland" => &[
            "keyboard-metadata",
            "keyboard-input-modes",
            "keyboard-wait-events-then-poll",
            "keyboard-wait-events-timeout-then-poll",
            "mouse-input-modes",
            "set-position",
            "get-position",
            "set-opacity",
            "get-opacity",
            "iconify",
            "keyboard-native-input",
            "keyboard-modifier-input",
            "mouse-native-input",
            "mouse-motion-input",
            "mouse-hover-boundary-input",
            "mouse-buttons-input",
            "mouse-scroll-input",
        ],
        "x11" => &[
            "keyboard-metadata",
            "keyboard-input-modes",
            "keyboard-wait-events-then-poll",
            "keyboard-wait-events-timeout-then-poll",
            "mouse-input-modes",
            "set-position",
            "get-position",
            "set-opacity",
            "get-opacity",
            "iconify",
            "maximize",
            "focus",
            "keyboard-native-input",
            "keyboard-modifier-input",
            "mouse-native-input",
            "mouse-motion-input",
            "mouse-hover-boundary-input",
            "mouse-buttons-input",
            "mouse-scroll-input",
        ],
        "win32" | "cocoa" => &[
            "create",
            "keyboard-metadata",
            "keyboard-input-modes",
            "keyboard-wait-events-then-poll",
            "keyboard-wait-events-timeout-then-poll",
            "mouse-input-modes",
            "set-position",
            "get-position",
            "focus",
            "keyboard-native-input",
            "keyboard-modifier-input",
            "mouse-native-input",
            "mouse-buttons-input",
            "mouse-motion-input",
            "mouse-hover-boundary-input",
            "mouse-scroll-input",
        ],
        value => panic!("unsupported qualified backend: {value}"),
    };

    for operation in operations {
        let record = probe(&backend, operation);
        assert_eq!(record["backend_requested"], backend);
        assert_eq!(record["backend_actual"], backend);
        assert_eq!(record["operation"], *operation);
        assert_eq!(record["result"], "ok");
        assert_operation_contract(&backend, operation, &record);
    }
}

fn assert_operation_contract(backend: &str, operation: &str, record: &Value) {
    match operation {
        "mouse-input-modes" => assert_mouse_input_modes(record),
        "keyboard-input-modes" => assert_keyboard_input_modes(record),
        "keyboard-metadata" => assert_keyboard_metadata(record),
        "keyboard-wait-events-then-poll" | "keyboard-wait-events-timeout-then-poll" => {
            assert_keyboard_wait(record);
        }
        "keyboard-native-input" => assert_native_keyboard_input(record),
        "keyboard-modifier-input" => assert_native_keyboard_modifier_input(record),
        "mouse-native-input" => assert_native_mouse_input(record),
        "mouse-motion-input" => assert_native_mouse_motion(record),
        "mouse-hover-boundary-input" => assert_native_mouse_hover_boundary(record),
        "mouse-buttons-input" => assert_native_mouse_buttons_input(record),
        "mouse-scroll-input" => assert_native_mouse_scroll_input(record),
        _ => {}
    }

    if backend == "wayland" && matches!(operation, "set-position" | "set-opacity") {
        let callbacks = record["callbacks"]
            .as_array()
            .expect("callbacks should be a JSON array");
        assert!(
            callbacks.iter().any(|callback| {
                matches!(
                    callback["code"].as_i64(),
                    Some(code) if code == i64::from(glfw::ffi::GLFW_FEATURE_UNAVAILABLE)
                )
            }),
            "Wayland {operation} must report GLFW_FEATURE_UNAVAILABLE: {record}"
        );
    }
    if backend == "wayland" && operation == "get-position" {
        assert_eq!(record["value"], serde_json::json!([0, 0]));
    }
    if backend == "wayland" && operation == "get-opacity" {
        assert_eq!(record["value"], serde_json::json!(1.0));
    }
}

fn assert_mouse_input_modes(record: &Value) {
    assert_eq!(
        record["value"],
        serde_json::json!({
            "defaults": {
                "sticky_mouse_buttons": false,
                "lock_key_modifier_reporting": false,
            },
            "enabled": {
                "sticky_mouse_buttons": true,
                "lock_key_modifier_reporting": true,
            },
            "disabled_after_reset": {
                "sticky_mouse_buttons": true,
                "lock_key_modifier_reporting": true,
            },
        })
    );
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

fn assert_keyboard_input_modes(record: &Value) {
    assert_eq!(record["value"]["default_sticky_keys"], false);
    assert_eq!(record["value"]["enabled_sticky_keys"], true);
    assert_eq!(record["value"]["disabled_after_reset"], true);
    assert_eq!(record["value"]["callback_installed"], true);
    assert_eq!(
        record["value"]["actions"],
        serde_json::json!(["Press", "Release"])
    );
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

fn assert_keyboard_metadata(record: &Value) {
    assert!(record["value"]["a_scancode"].is_number());
    if record["backend_actual"] == "wayland" {
        assert_eq!(record["value"]["names_deferred_until_keyboard_event"], true);
        assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
        return;
    }

    assert!(record["value"]["a_name_by_key"].is_string());
    assert_eq!(
        record["value"]["a_name_by_scancode"],
        record["value"]["a_name_by_key"]
    );
    assert!(record["value"]["escape_name"].is_null());
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

fn assert_keyboard_wait(record: &Value) {
    assert_eq!(record["value"]["callback_installed"], true);
    assert_eq!(
        record["value"]["actions_after_poll"],
        serde_json::json!(["Press", "Release"])
    );
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

fn assert_native_keyboard_input(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["case"], "a-press-release");
    assert_eq!(
        record["value"]["actions"],
        serde_json::json!(["Press", "Release"])
    );
    let events = record["value"]["observed_events"]
        .as_array()
        .expect("native keyboard events should be an array");
    assert_eq!(events.len(), 2);
    for (event, action) in events.iter().zip(["Press", "Release"]) {
        assert_eq!(event["key"], "A");
        assert_eq!(event["action"], action);
        assert_eq!(event["modifiers"], 0);
    }
    assert_eq!(record["value"]["final_state"], "Release");
    let expected_injector = match record["backend_actual"].as_str() {
        Some("x11") => "x11-xtest",
        Some("wayland") => "x11-xtest-parent",
        Some("win32") => "send-input",
        Some("cocoa") => "cg-event-post",
        value => panic!("unexpected native-input backend: {value:?}"),
    };
    assert_eq!(record["value"]["injector"], expected_injector);
    let scancodes = record["value"]["scancodes"]
        .as_array()
        .expect("native input scancodes should be an array");
    assert_eq!(scancodes.len(), 2);
    assert_eq!(scancodes[0], scancodes[1]);
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

fn assert_native_keyboard_modifier_input(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["case"], "shift-a-press-release");
    let events = record["value"]["observed_events"]
        .as_array()
        .expect("native modifier events should be an array");
    assert_eq!(events.len(), 4);
    for (event, (key, action)) in events.iter().zip([
        ("LeftShift", "Press"),
        ("A", "Press"),
        ("A", "Release"),
        ("LeftShift", "Release"),
    ]) {
        assert_eq!(event["key"], key);
        assert_eq!(event["action"], action);
    }
    let shift_mask = i64::from(glfw::Modifiers::Shift.bits());
    let standard_modifier_mask = i64::from(
        (glfw::Modifiers::Shift
            | glfw::Modifiers::Control
            | glfw::Modifiers::Alt
            | glfw::Modifiers::Super)
            .bits(),
    );
    for event in [&events[1], &events[2]] {
        let modifiers = event["modifiers"]
            .as_i64()
            .expect("modified A event should report a modifier mask");
        assert_eq!(modifiers & standard_modifier_mask, shift_mask);
    }
    let scancodes: Vec<_> = events
        .iter()
        .map(|event| {
            event["scancode"]
                .as_i64()
                .expect("native key events should report scancodes")
        })
        .collect();
    assert_eq!(scancodes[0], scancodes[3], "Shift scancode must be stable");
    assert_eq!(scancodes[1], scancodes[2], "A scancode must be stable");
    assert_eq!(record["value"]["final_state"], "Release");
    assert_eq!(record["value"]["left_shift_final_state"], "Release");
    assert_eq!(record["value"]["failure_reason"], Value::Null);
    let expected_injector = match record["backend_actual"].as_str() {
        Some("x11") => "x11-xtest",
        Some("wayland") => "x11-xtest-parent",
        Some("win32") => "send-input",
        Some("cocoa") => "cg-event-post",
        value => panic!("unexpected native modifier backend: {value:?}"),
    };
    assert_eq!(record["value"]["injector"], expected_injector);
    assert!(record["callbacks"].as_array().is_some_and(Vec::is_empty));
}

fn assert_native_mouse_input(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["hovered"], true);
    assert_eq!(
        record["value"]["observed_events"],
        serde_json::json!([
            {"button": "Button1", "action": "Press", "modifiers": 0},
            {"button": "Button1", "action": "Release", "modifiers": 0},
        ])
    );
    assert_eq!(record["value"]["case"], "left-button-press-release");
    assert_eq!(record["value"]["final_state"], "Release");
    assert_eq!(record["value"]["failure_reason"], Value::Null);
    let expected_injector = match record["backend_actual"].as_str() {
        Some("x11") => "x11-xtest",
        Some("wayland") => "x11-xtest-parent",
        Some("win32") => "send-input",
        Some("cocoa") => "cg-event-post",
        value => panic!("unexpected mouse input backend: {value:?}"),
    };
    assert_eq!(record["value"]["injector"], expected_injector);
}

fn assert_native_mouse_motion(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["hovered"], true);

    let initial = record["value"]["initial_cursor_position"]
        .as_array()
        .expect("initial cursor position should be an array");
    let final_position = record["value"]["cursor_position"]
        .as_array()
        .expect("final cursor position should be an array");
    let observed = record["value"]["observed_events"]
        .as_array()
        .expect("cursor movement events should be an array");
    assert_eq!(observed.len(), 1);
    assert_eq!(
        record["value"]["injected_root_delta"],
        serde_json::json!([6, 4])
    );
    for (index, delta) in [6.0, 4.0].into_iter().enumerate() {
        let initial = initial[index]
            .as_f64()
            .expect("initial cursor coordinate should be a number");
        let final_coordinate = final_position[index]
            .as_f64()
            .expect("final cursor coordinate should be a number");
        let event_coordinate = observed[0][if index == 0 { "x" } else { "y" }]
            .as_f64()
            .expect("cursor event coordinate should be a number");
        assert!((event_coordinate - final_coordinate).abs() <= 0.01);
        if matches!(record["backend_actual"].as_str(), Some("wayland" | "win32")) {
            assert!(event_coordinate > initial);
        } else {
            assert!((final_coordinate - initial - delta).abs() <= 0.01);
        }
    }

    assert_eq!(record["value"]["case"], "pointer-movement");
    assert_eq!(record["value"]["final_state"], Value::Null);
    assert_eq!(record["value"]["failure_reason"], Value::Null);
    let expected_injector = match record["backend_actual"].as_str() {
        Some("x11") => "x11-xtest",
        Some("wayland") => "x11-xtest-parent",
        Some("win32") => "send-input",
        Some("cocoa") => "cg-event-post",
        value => panic!("unexpected native pointer-motion backend: {value:?}"),
    };
    assert_eq!(record["value"]["injector"], expected_injector);
}

fn assert_native_mouse_hover_boundary(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["hovered"], true);
    assert_eq!(
        record["value"]["observed_events"],
        serde_json::json!([{"entered": false}, {"entered": true}])
    );
    assert_eq!(record["value"]["case"], "pointer-leave-enter");
    assert_eq!(record["value"]["final_state"], Value::Null);
    assert_eq!(record["value"]["failure_reason"], Value::Null);
    let expected_injector = match record["backend_actual"].as_str() {
        Some("x11") => "x11-xtest",
        Some("wayland") => "x11-xtest-parent",
        Some("win32") => "set-cursor-pos",
        Some("cocoa") => "cg-event-post",
        value => panic!("unexpected native pointer-boundary backend: {value:?}"),
    };
    assert_eq!(record["value"]["injector"], expected_injector);
}

struct MouseButtonExpectations {
    events: Vec<Value>,
    tested: Vec<Value>,
    unsupported: Vec<Value>,
    all_eight: bool,
    basis: &'static str,
    injector: &'static str,
}

fn mouse_button_press_release(button: &str) -> [Value; 2] {
    [
        serde_json::json!({"button": button, "action": "Press", "modifiers": 0}),
        serde_json::json!({"button": button, "action": "Release", "modifiers": 0}),
    ]
}

fn assert_native_mouse_buttons_input(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["hovered"], true);
    let expected = match record["backend_actual"].as_str() {
        Some("x11" | "wayland") => x11_mouse_button_expectations(record),
        Some("win32") => win32_mouse_button_expectations(),
        Some("cocoa") => cocoa_mouse_button_expectations(),
        value => panic!("unexpected native mouse-button backend: {value:?}"),
    };
    assert_eq!(
        record["value"]["observed_events"],
        serde_json::json!(expected.events)
    );
    assert_eq!(
        record["value"]["button_coverage"]["tested_mappings"],
        serde_json::json!(expected.tested)
    );
    assert_eq!(
        record["value"]["button_coverage"]["mapping_basis"],
        expected.basis
    );
    assert_eq!(
        record["value"]["button_coverage"]["all_eight_glfw_buttons_tested"],
        expected.all_eight
    );
    assert_eq!(
        record["value"]["button_coverage"]["unsupported_mappings"],
        serde_json::json!(expected.unsupported)
    );
    assert_eq!(
        record["value"]["case"],
        "eligible-mouse-buttons-press-release"
    );
    assert_eq!(
        record["value"]["final_state"],
        serde_json::json!(vec!["Release"; 8])
    );
    assert_eq!(record["value"]["failure_reason"], Value::Null);
    assert_eq!(record["value"]["injector"], expected.injector);
}

fn x11_mouse_button_expectations(record: &Value) -> MouseButtonExpectations {
    let maximum = record["value"]["button_coverage"]["maximum_server_button"]
        .as_u64()
        .expect("X11 maximum server button should be recorded");
    let mappings: [(u8, &str); 8] = [
        (1, "Button1"),
        (2, "Button3"),
        (3, "Button2"),
        (8, "Button4"),
        (9, "Button5"),
        (10, "Button6"),
        (11, "Button7"),
        (12, "Button8"),
    ];
    let tested: Vec<_> = mappings
        .into_iter()
        .filter(|(button, _)| u64::from(*button) <= maximum)
        .collect();
    let unsupported: Vec<_> = mappings
        .into_iter()
        .filter(|(button, _)| u64::from(*button) > maximum)
        .collect();
    MouseButtonExpectations {
        events: tested
            .iter()
            .flat_map(|(_, button)| mouse_button_press_release(button))
            .collect(),
        tested: tested
            .iter()
            .map(|(button, glfw_button)| {
                serde_json::json!({"server_button": button, "glfw_button": glfw_button})
            })
            .collect(),
        unsupported: unsupported
            .iter()
            .map(|(button, glfw_button)| {
                serde_json::json!({
                    "server_button": button,
                    "glfw_button": glfw_button,
                    "reason": "server_button_exceeds_x11_pointer_mapping",
                })
            })
            .collect(),
        all_eight: unsupported.is_empty(),
        basis: "x11_pointer_mapping_length",
        injector: if record["backend_actual"] == "x11" {
            "x11-xtest"
        } else {
            "x11-xtest-parent"
        },
    }
}

fn win32_mouse_button_expectations() -> MouseButtonExpectations {
    let mappings = [
        ("left", "Button1"),
        ("right", "Button2"),
        ("middle", "Button3"),
        ("XBUTTON1", "Button4"),
        ("XBUTTON2", "Button5"),
    ];
    MouseButtonExpectations {
        events: mappings
            .iter()
            .flat_map(|(_, button)| mouse_button_press_release(button))
            .collect(),
        tested: mappings
            .iter()
            .map(|(native_button, glfw_button)| {
                serde_json::json!({"native_button": native_button, "glfw_button": glfw_button})
            })
            .collect(),
        unsupported: ["Button6", "Button7", "Button8"]
            .into_iter()
            .map(|glfw_button| {
                serde_json::json!({
                    "native_button": Value::Null,
                    "glfw_button": glfw_button,
                    "reason": "win32_sendinput_has_no_glfw_mapping",
                })
            })
            .collect(),
        all_eight: false,
        basis: "win32_sendinput",
        injector: "send-input",
    }
}

fn cocoa_mouse_button_expectations() -> MouseButtonExpectations {
    let mappings: [(u8, &str); 8] = [
        (0, "Button1"),
        (1, "Button2"),
        (2, "Button3"),
        (3, "Button4"),
        (4, "Button5"),
        (5, "Button6"),
        (6, "Button7"),
        (7, "Button8"),
    ];
    MouseButtonExpectations {
        events: mappings
            .iter()
            .flat_map(|(_, button)| mouse_button_press_release(button))
            .collect(),
        tested: mappings
            .iter()
            .map(|(native_button, glfw_button)| {
                serde_json::json!({"native_button": native_button, "glfw_button": glfw_button})
            })
            .collect(),
        unsupported: Vec::new(),
        all_eight: true,
        basis: "cocoa_cgevent_button_number",
        injector: "cg-event-post",
    }
}

fn assert_native_mouse_scroll_input(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["hovered"], true);
    let expected = [
        (0.0_f64, 1.0_f64),
        (0.0_f64, -1.0_f64),
        (1.0_f64, 0.0_f64),
        (-1.0_f64, 0.0_f64),
    ];
    let observed = record["value"]["observed_events"]
        .as_array()
        .expect("scroll events should be an array");
    if record["backend_actual"] == "cocoa" {
        assert_eq!(observed.len(), expected.len());
        for (event, (expected_horizontal, expected_vertical)) in observed.iter().zip(expected) {
            let actual_horizontal = event["dx"]
                .as_f64()
                .expect("horizontal scroll delta should be a number");
            let actual_vertical = event["dy"]
                .as_f64()
                .expect("vertical scroll delta should be a number");
            if expected_horizontal.abs() <= f64::EPSILON {
                assert!(actual_horizontal.abs() <= f64::EPSILON);
            } else {
                assert!(actual_horizontal.abs() > f64::EPSILON);
                assert_eq!(
                    actual_horizontal.is_sign_positive(),
                    expected_horizontal.is_sign_positive()
                );
            }
            if expected_vertical.abs() <= f64::EPSILON {
                assert!(actual_vertical.abs() <= f64::EPSILON);
            } else {
                assert!(actual_vertical.abs() > f64::EPSILON);
                assert_eq!(
                    actual_vertical.is_sign_positive(),
                    expected_vertical.is_sign_positive()
                );
            }
        }
    } else {
        assert_eq!(
            record["value"]["observed_events"],
            serde_json::json!([
                {"dx": 0.0, "dy": 1.0},
                {"dx": 0.0, "dy": -1.0},
                {"dx": 1.0, "dy": 0.0},
                {"dx": -1.0, "dy": 0.0},
            ])
        );
    }
    assert_eq!(
        record["value"]["case"],
        "vertical-horizontal-scroll-directions"
    );
    assert_eq!(record["value"]["final_state"], Value::Null);
    assert_eq!(record["value"]["failure_reason"], Value::Null);
    let expected_injector = match record["backend_actual"].as_str() {
        Some("x11") => "x11-xtest",
        Some("wayland") => "x11-xtest-parent",
        Some("win32") => "send-input",
        Some("cocoa") => "cg-event-post",
        value => panic!("unexpected native scroll backend: {value:?}"),
    };
    assert_eq!(record["value"]["injector"], expected_injector);
}
