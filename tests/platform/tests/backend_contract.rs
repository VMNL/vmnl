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

fn is_mouse_native_input(operation: &str) -> bool {
    matches!(
        operation,
        "mouse-native-input" | "mouse-buttons-input" | "mouse-motion-input" | "mouse-scroll-input"
    )
}

fn probe(backend: &str, operation: &str) -> Value {
    let output = if matches!(
        operation,
        "keyboard-native-input"
            | "mouse-native-input"
            | "mouse-buttons-input"
            | "mouse-motion-input"
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
    let max_server_button = if operation == "mouse-buttons-input" {
        Some(x11_pointer_button_count()?)
    } else {
        None
    };
    let ready_file = env::temp_dir().join(format!(
        "vmnl-platform-ready-{}-{sequence}",
        std::process::id()
    ));
    let injector = input_injector_name(backend)?;
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
            "mouse-native-input" => inject_mouse_left(),
            "mouse-buttons-input" => {
                inject_mouse_buttons(usize::from(max_server_button.unwrap_or_default()))
            }
            "mouse-motion-input" => inject_mouse_motion(),
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
fn inject_key_a() -> Result<(), String> {
    use x11rb::{
        connection::Connection as _,
        protocol::{
            xproto::{ConnectionExt as _, KEY_PRESS_EVENT, KEY_RELEASE_EVENT},
            xtest::ConnectionExt as _,
        },
    };

    const XK_A: u32 = 0x0041;
    const XK_A_LOWER: u32 = 0x0061;

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
    let keycode_offset = mapping
        .keysyms
        .chunks(keysyms_per_keycode)
        .position(|keysyms| keysyms.contains(&XK_A) || keysyms.contains(&XK_A_LOWER))
        .ok_or_else(|| "X11 keyboard mapping has no A keysym".to_owned())?;
    let keycode = u16::from(setup.min_keycode)
        + u16::try_from(keycode_offset)
            .map_err(|_| "X11 A keycode offset is too large".to_owned())?;
    let keycode = u8::try_from(keycode).map_err(|_| "X11 A keycode is invalid".to_owned())?;

    connection
        .xtest_fake_input(KEY_PRESS_EVENT, keycode, 0, 0, 0, 0, 0)
        .map_err(|error| format!("failed to enqueue XTEST A press: {error}"))?
        .check()
        .map_err(|error| format!("XTEST A press failed: {error}"))?;
    connection
        .xtest_fake_input(KEY_RELEASE_EVENT, keycode, 0, 0, 0, 0, 0)
        .map_err(|error| format!("failed to enqueue XTEST A release: {error}"))?
        .check()
        .map_err(|error| format!("XTEST A release failed: {error}"))?;
    connection
        .flush()
        .map_err(|error| format!("failed to flush XTEST input: {error}"))
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

#[cfg(not(target_os = "linux"))]
fn inject_mouse_scroll() -> Result<(), String> {
    Err("mouse scroll injection is implemented only with XTEST on X11".to_owned())
}

#[cfg(not(target_os = "linux"))]
fn inject_mouse_motion() -> Result<(), String> {
    Err("pointer movement injection is currently implemented only with XTEST".to_owned())
}

#[cfg(not(target_os = "linux"))]
fn inject_mouse_buttons(_max_server_button: usize) -> Result<(), String> {
    Err(
        "eligible mouse-button injection is currently implemented only with XTEST on X11"
            .to_owned(),
    )
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

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_key_a() -> Result<(), String> {
    Err("native input injection is unsupported on this OS".to_owned())
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
            "mouse-native-input",
            "mouse-motion-input",
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
            "mouse-native-input",
            "mouse-motion-input",
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
            "mouse-native-input",
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
        "mouse-native-input" => assert_native_mouse_input(record),
        "mouse-motion-input" => assert_native_mouse_motion(record),
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
    assert_eq!(
        record["value"]["actions"],
        serde_json::json!(["Press", "Release"])
    );
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
        if record["backend_actual"] == "wayland" {
            assert!(event_coordinate > initial);
        } else {
            assert!((final_coordinate - initial - delta).abs() <= 0.01);
        }
    }

    assert_eq!(record["value"]["case"], "pointer-movement");
    assert_eq!(record["value"]["final_state"], Value::Null);
    assert_eq!(record["value"]["failure_reason"], Value::Null);
    let expected_injector = if record["backend_actual"] == "x11" {
        "x11-xtest"
    } else {
        "x11-xtest-parent"
    };
    assert_eq!(record["value"]["injector"], expected_injector);
}

fn assert_native_mouse_buttons_input(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["hovered"], true);
    let maximum_server_button = record["value"]["button_coverage"]["maximum_server_button"]
        .as_u64()
        .expect("X11 maximum server button should be recorded");
    let expected_button_mappings = [
        (1, "Button1"),
        (2, "Button3"),
        (3, "Button2"),
        (8, "Button4"),
        (9, "Button5"),
        (10, "Button6"),
        (11, "Button7"),
        (12, "Button8"),
    ];
    let expected_events: Vec<Value> = expected_button_mappings
        .into_iter()
        .filter(|(server_button, _)| *server_button <= maximum_server_button)
        .flat_map(|(_, button)| {
            [
                serde_json::json!({"button": button, "action": "Press", "modifiers": 0}),
                serde_json::json!({"button": button, "action": "Release", "modifiers": 0}),
            ]
        })
        .collect();
    assert_eq!(
        record["value"]["observed_events"],
        serde_json::json!(expected_events)
    );

    let expected_tested_mappings: Vec<Value> = expected_button_mappings
        .into_iter()
        .filter(|(server_button, _)| *server_button <= maximum_server_button)
        .map(|(server_button, glfw_button)| {
            serde_json::json!({"server_button": server_button, "glfw_button": glfw_button})
        })
        .collect();
    let expected_unsupported_mappings: Vec<Value> = expected_button_mappings
        .into_iter()
        .filter(|(server_button, _)| *server_button > maximum_server_button)
        .map(|(server_button, glfw_button)| {
            serde_json::json!({
                "server_button": server_button,
                "glfw_button": glfw_button,
                "reason": "server_button_exceeds_x11_pointer_mapping",
            })
        })
        .collect();
    let all_eight_glfw_buttons_tested = expected_unsupported_mappings.is_empty();
    assert_eq!(
        record["value"]["button_coverage"]["tested_mappings"],
        serde_json::json!(expected_tested_mappings)
    );
    assert_eq!(
        record["value"]["button_coverage"]["all_eight_glfw_buttons_tested"],
        all_eight_glfw_buttons_tested
    );
    assert_eq!(
        record["value"]["button_coverage"]["unsupported_mappings"],
        serde_json::json!(expected_unsupported_mappings)
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
    let expected_injector = match record["backend_actual"].as_str() {
        Some("x11") => "x11-xtest",
        Some("wayland") => "x11-xtest-parent",
        value => panic!("unexpected native mouse-button backend: {value:?}"),
    };
    assert_eq!(record["value"]["injector"], expected_injector);
}

fn assert_native_mouse_scroll_input(record: &Value) {
    assert_eq!(record["value"]["qualified"], true);
    assert_eq!(record["value"]["stage"], "input");
    assert_eq!(record["value"]["focused"], true);
    assert_eq!(record["value"]["hovered"], true);
    assert_eq!(
        record["value"]["observed_events"],
        serde_json::json!([
            {"dx": 0.0, "dy": 1.0},
            {"dx": 0.0, "dy": -1.0},
            {"dx": 1.0, "dy": 0.0},
            {"dx": -1.0, "dy": 0.0},
        ])
    );
    assert_eq!(
        record["value"]["case"],
        "vertical-horizontal-scroll-directions"
    );
    assert_eq!(record["value"]["final_state"], Value::Null);
    assert_eq!(record["value"]["failure_reason"], Value::Null);
    let expected_injector = match record["backend_actual"].as_str() {
        Some("x11") => "x11-xtest",
        Some("wayland") => "x11-xtest-parent",
        value => panic!("unexpected native scroll backend: {value:?}"),
    };
    assert_eq!(record["value"]["injector"], expected_injector);
}
