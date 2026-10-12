// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Native event posting with bounded holds and release-on-error cleanup.

use serde_json::Value;
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub(super) fn inject(spec: &Value, ready: &Path) -> Result<(), String> {
    inject_with(spec, ready, post)
}

pub(crate) fn inject_with(
    spec: &Value,
    ready: &Path,
    mut post: impl FnMut(&Value) -> Result<(), String>,
) -> Result<(), String> {
    let actions = spec["actions"].as_array().ok_or("case actions missing")?;
    let mut held: Vec<Value> = Vec::new();
    // Lock cases tap twice to restore the initial toggle state. Track successful presses so
    // an injector error between those taps still performs the missing inverse toggle.
    let mut toggles: Vec<Value> = Vec::new();
    let paired_locks: Vec<_> = actions
        .iter()
        .filter(|a| {
            a["lock"] == true
                && a["down"] == true
                && actions
                    .iter()
                    .filter(|b| b["lock"] == true && b["down"] == true && b["code"] == a["code"])
                    .count()
                    == 2
        })
        .map(|a| a["code"].clone())
        .collect();
    let result = (|| {
        for action in actions {
            match action["kind"].as_str() {
                Some("wait") => std::thread::sleep(Duration::from_millis(
                    action["ms"].as_u64().ok_or("hold duration missing")?,
                )),
                Some("focus-wait") => {
                    let deadline = Instant::now() + Duration::from_secs(5);
                    while !std::fs::read_to_string(ready.with_extension("observed"))
                        .is_ok_and(|v| v == "OBSERVED\n")
                    {
                        if Instant::now() >= deadline {
                            return Err("focus loss not observed before deadline".to_owned());
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
                Some("key" | "mouse" | "text") => {
                    post(action)?;
                    if action["lock"] == true
                        && action["down"] == true
                        && paired_locks.contains(&action["code"])
                    {
                        if toggles.iter().any(|a| a["code"] == action["code"]) {
                            toggles.retain(|a| a["code"] != action["code"]);
                        } else {
                            toggles.push(action.clone());
                        }
                    }
                    if action["down"] == true {
                        held.push(action.clone());
                    } else {
                        held.retain(|e| {
                            !(e["kind"] == action["kind"] && e["code"] == action["code"])
                        });
                    }
                }
                _ => return Err("invalid injection action".to_owned()),
            }
        }
        Ok(())
    })();
    let mut cleanup_errors = Vec::new();
    for mut action in held.into_iter().rev() {
        action["down"] = Value::Bool(false);
        action["modifiers"] = Value::from(0);
        if let Err(error) = post(&action) {
            cleanup_errors.push(error);
        }
    }
    for mut action in toggles {
        action["modifiers"] = Value::from(0);
        if let Err(error) = post(&action) {
            cleanup_errors.push(error);
        }
        action["down"] = Value::Bool(false);
        if let Err(error) = post(&action) {
            cleanup_errors.push(error);
        }
    }
    if !cleanup_errors.is_empty() {
        return Err(format!(
            "injection={result:?}; key/button cleanup failed: {cleanup_errors:?}"
        ));
    }
    result?;
    std::fs::write(ready.with_extension("done"), b"DONE\n")
        .map_err(|e| format!("completion marker: {e}"))
}

#[cfg(target_os = "linux")]
fn post(action: &Value) -> Result<(), String> {
    use x11rb::protocol::{
        xproto::{BUTTON_PRESS_EVENT, BUTTON_RELEASE_EVENT, KEY_PRESS_EVENT, KEY_RELEASE_EVENT},
        xtest::ConnectionExt as _,
    };
    let (connection, _) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let down = action["down"] == true;
    let kind = if action["kind"] == "mouse" {
        if down {
            BUTTON_PRESS_EVENT
        } else {
            BUTTON_RELEASE_EVENT
        }
    } else {
        if down {
            KEY_PRESS_EVENT
        } else {
            KEY_RELEASE_EVENT
        }
    };
    let code = action["code"]
        .as_u64()
        .and_then(|n| u8::try_from(n).ok())
        .ok_or("XTEST code out of range")?;
    let result = connection
        .xtest_fake_input(kind, code, 0, 0, 0, 0, 0)
        .map_err(|e| e.to_string())?
        .check()
        .map_err(|e| format!("XTEST {action}: {e}"));
    result
}

#[cfg(target_os = "windows")]
fn post(action: &Value) -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
        KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEINPUT,
    };
    let code = action["code"]
        .as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .ok_or("Win32 code out of range")?;
    let down = action["down"] == true;
    let input = if action["kind"] == "mouse" {
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: if down {
                        MOUSEEVENTF_LEFTDOWN
                    } else {
                        MOUSEEVENTF_LEFTUP
                    },
                    ..MOUSEINPUT::default()
                },
            },
        }
    } else {
        let unicode = action["kind"] == "text";
        let pause = action["pause"] == true;
        let flags = if unicode {
            KEYEVENTF_UNICODE
        } else if pause {
            0
        } else {
            KEYEVENTF_SCANCODE
        } | if !unicode && !pause && code & 0x100 != 0 {
            KEYEVENTF_EXTENDEDKEY
        } else {
            0
        } | if down { 0 } else { KEYEVENTF_KEYUP };
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: if pause { 0x13 } else { 0 },
                    wScan: if unicode { code } else { code & 0xff },
                    dwFlags: flags,
                    ..KEYBDINPUT::default()
                },
            },
        }
    };
    let size = i32::try_from(std::mem::size_of::<INPUT>()).map_err(|e| e.to_string())?;
    // SAFETY: One initialized INPUT is live for the call and `size` is its exact ABI size.
    if unsafe { SendInput(1, &raw const input, size) } == 1 {
        Ok(())
    } else {
        Err(format!("SendInput: {}", std::io::Error::last_os_error()))
    }
}

#[cfg(target_os = "macos")]
fn post(action: &Value) -> Result<(), String> {
    use std::ffi::c_void;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreateKeyboardEvent(source: *mut c_void, code: u16, down: bool) -> *mut c_void;
        fn CGEventKeyboardSetUnicodeString(event: *mut c_void, length: usize, text: *const u16);
        fn CGEventCreateMouseEvent(
            source: *mut c_void,
            kind: u32,
            point: CGPoint,
            button: u32,
        ) -> *mut c_void;
        fn CGEventCreate(source: *mut c_void) -> *mut c_void;
        fn CGEventGetLocation(event: *mut c_void) -> CGPoint;
        fn CGEventSetFlags(event: *mut c_void, flags: u64);
        fn CGEventPost(tap: u32, event: *mut c_void);
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }
    let code = action["code"]
        .as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .ok_or("Cocoa code out of range")?;
    let down = action["down"] == true;
    let mask = action["modifiers"].as_u64().unwrap_or(0);
    let flags = (if mask & 1 != 0 { 1 << 17 } else { 0 })
        | (if mask & 2 != 0 { 1 << 18 } else { 0 })
        | (if mask & 4 != 0 { 1 << 19 } else { 0 })
        | (if mask & 8 != 0 { 1 << 20 } else { 0 })
        | (if mask & 16 != 0 { 1 << 16 } else { 0 });
    let event = if action["kind"] == "mouse" {
        // SAFETY: Null source selects the default; the returned reference is checked before use.
        let pointer = unsafe { CGEventCreate(std::ptr::null_mut()) };
        if pointer.is_null() {
            return Err("CGEventCreate returned null".to_owned());
        }
        // SAFETY: `pointer` is a live retained event until released after the location query.
        let point = unsafe {
            let p = CGEventGetLocation(pointer);
            CFRelease(pointer);
            p
        };
        // SAFETY: Finite OS pointer location and documented left down/up event types.
        unsafe { CGEventCreateMouseEvent(std::ptr::null_mut(), if down { 1 } else { 2 }, point, 0) }
    } else {
        // SAFETY: Fixture supplies a physical hardware code; null selects the default source.
        unsafe { CGEventCreateKeyboardEvent(std::ptr::null_mut(), code, down) }
    };
    if event.is_null() {
        return Err("Core Graphics event allocation failed".to_owned());
    }
    let text: Vec<u16> = action["text"]
        .as_str()
        .unwrap_or("")
        .encode_utf16()
        .collect();
    // SAFETY: Retained event and UTF-16 buffer stay live until after posting. HID tap is zero.
    unsafe {
        CGEventSetFlags(event, flags);
        if action["kind"] == "text" {
            CGEventKeyboardSetUnicodeString(event, text.len(), text.as_ptr());
        }
        CGEventPost(0, event);
        CFRelease(event);
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn post(_action: &Value) -> Result<(), String> {
    Err("no native injector on this OS".to_owned())
}
