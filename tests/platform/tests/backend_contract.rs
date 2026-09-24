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

fn probe(backend: &str, operation: &str) -> Value {
    let output = if operation == "keyboard-native-input" {
        native_keyboard_probe(backend).expect("native keyboard probe should complete")
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

fn native_keyboard_probe(backend: &str) -> Result<Output, String> {
    let sequence = READY_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let ready_file = env::temp_dir().join(format!(
        "vmnl-platform-ready-{}-{sequence}",
        std::process::id()
    ));
    let injector = input_injector_name(backend)?;
    let result = (|| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_platform_probe"))
            .args([backend, "keyboard-native-input"])
            .env("VMNL_PLATFORM_READY_FILE", &ready_file)
            .env("VMNL_PLATFORM_INPUT_INJECTOR", injector)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("platform probe should start: {error}"))?;

        if let Err(reason) = wait_until_ready(&mut child, &ready_file) {
            let reason = append_external_x11_focus_diagnostic(backend, &reason);
            return Err(terminate_with_diagnostics(child, &reason));
        }
        log_external_x11_focus_at_ready(backend);
        if let Err(reason) = inject_key_a() {
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

fn wait_until_ready(child: &mut Child, ready_file: &Path) -> Result<(), String> {
    let deadline = Instant::now() + NATIVE_INPUT_TIMEOUT;
    loop {
        if ready_file.is_file() {
            return Ok(());
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to inspect platform probe: {error}"))?
        {
            return Err(format!("platform probe exited before READY with {status}"));
        }
        if Instant::now() >= deadline {
            return Err("platform probe did not emit READY within 7 seconds".to_owned());
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

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn inject_key_a() -> Result<(), String> {
    Err("native input injection is unsupported on this OS".to_owned())
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
