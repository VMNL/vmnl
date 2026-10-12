// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Per-row eligibility and independent native input sequences.

#[path = "input_injection.rs"]
mod input_injection;
#[path = "../../src/input_keys.rs"]
mod input_keys;

use glfw::Key;
use input_keys::KEYS;
use serde_json::{json, Value};

#[derive(Clone, Copy)]
struct NativeKey {
    key: Key,
    code: u16,
    scancode: u16,
}

pub(super) fn inject(spec: &Value, ready: &std::path::Path) -> Result<(), String> {
    input_injection::inject(spec, ready)
}

pub(super) fn run(backend: &str) {
    let (keys, mut rows) =
        available_keys(backend).expect("native key capabilities must be measurable");
    let mut cases = Vec::new();
    key_cases(&keys, backend, &mut cases);
    let find = |key| keys.iter().find(|n| n.key == key).copied();
    let a = find(Key::A).expect("A must be injectable");
    modifier_cases(&keys, backend, &mut cases, &mut rows);
    lock_cases(&keys, backend, &mut cases, &mut rows);
    if matches!(backend, "x11" | "wayland") {
        let spec = case(
            "bounded-a-repeat",
            "repeat",
            vec![
                key_action(a, true, 0),
                json!({"kind":"wait","ms":1500}),
                key_action(a, false, 0),
            ],
            vec![
                key_event(a, "Press", Some(0)),
                key_event(a, "Release", Some(0)),
            ],
            vec![Key::A],
            false,
        );
        cases.push(spec);
    } else {
        rows.push(json!({"case":"bounded-a-repeat","status":"not injectable","reason":"SendInput/CGEventPost key-down does not establish a hardware auto-repeat source; explicit repeat events would not qualify a held-key test"}));
    }
    cases.push(case(
        "focus-loss-held-key-and-button",
        "focus-loss",
        vec![
            key_action(a, true, 0),
            mouse_action(true, 0, backend),
            json!({"kind":"focus-wait"}),
            key_action(a, false, 0),
            mouse_action(false, 0, backend),
        ],
        vec![
            key_event(a, "Press", Some(0)),
            mouse_event("Press", 0),
            json!({"kind":"focus","focused":false}),
            key_event(a, "Release", Some(0)),
            mouse_event("Release", 0),
        ],
        vec![Key::A],
        true,
    ));
    text_cases(backend, &keys, &mut cases, &mut rows).expect("text eligibility must be measurable");
    rows.extend(
        cases
            .iter()
            .filter(|c| c["family"] != "key")
            .map(|c| json!({"case":c["case"],"status":"eligible","reason":Value::Null})),
    );
    super::retain_record(
        backend,
        &json!({"schema":vmnl_platform_tests::PROBE_SCHEMA_VERSION,"backend_requested":backend,"backend_actual":backend,
        "operation":"native-input-eligibility","result":"recorded","value":{"rows":rows,"native_case_count":cases.len()}}),
    );
    for spec in cases {
        let record = super::probe_with_case(backend, "native-input-case", Some(&spec));
        assert_eq!(record["schema"], vmnl_platform_tests::PROBE_SCHEMA_VERSION);
        assert_eq!(record["backend_actual"], backend);
        assert_eq!(record["value"]["case"], spec["case"]);
        assert_eq!(record["value"]["qualified"], true, "{}", spec["case"]);
        assert_eq!(record["value"]["expected_events"], spec["expected"]);
    }
}

fn key_cases(keys: &[NativeKey], backend: &str, cases: &mut Vec<Value>) {
    for key in keys {
        let mut actions = vec![
            key_action(*key, true, modifier(key.key)),
            key_action(*key, false, 0),
        ];
        let mut expected = vec![
            key_event(*key, "Press", (modifier(key.key) == 0).then_some(0)),
            key_event(*key, "Release", (modifier(key.key) == 0).then_some(0)),
        ];
        let lock =
            matches!(key.key, Key::CapsLock | Key::NumLock | Key::ScrollLock) && backend != "cocoa";
        if lock {
            actions.extend(actions.clone());
            expected.extend(expected.clone());
        }
        cases.push(case(
            &format!("key-{:?}", key.key),
            "key",
            actions,
            expected,
            vec![key.key],
            false,
        ));
    }
}

fn modifier_cases(
    keys: &[NativeKey],
    backend: &str,
    cases: &mut Vec<Value>,
    rows: &mut Vec<Value>,
) {
    let find = |key| keys.iter().find(|n| n.key == key).copied();
    let a = find(Key::A).expect("A injectable");
    // Every nonempty subset of the four standard left modifiers, plus right-side variants.
    let left = [
        Key::LeftShift,
        Key::LeftControl,
        Key::LeftAlt,
        Key::LeftSuper,
    ];
    let mut combinations: Vec<Vec<Key>> = (1..16)
        .map(|mask| {
            left.iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, k)| *k)
                .collect()
        })
        .collect();
    combinations.extend(
        [
            Key::RightShift,
            Key::RightControl,
            Key::RightAlt,
            Key::RightSuper,
        ]
        .iter()
        .map(|k| vec![*k]),
    );
    for group in combinations {
        let name = format!("modifiers-{group:?}");
        let Some(native): Option<Vec<_>> = group.iter().map(|k| find(*k)).collect() else {
            rows.push(json!({"case":name,"status":"not injectable","reason":"modifier key unavailable in this backend/map"}));
            continue;
        };
        let mask = group.iter().fold(0, |m, k| m | modifier(*k));
        // Cocoa command shortcuts suppress key-up, while desktop Alt/Super shortcuts may grab
        // keys. Use a bare modifier chord, not an application shortcut, for these combinations.
        let shortcut = mask & 0b1100 != 0;
        let mut actions = Vec::new();
        let mut expected = Vec::new();
        let mut active = 0;
        for key in &native {
            active |= modifier(key.key);
            actions.push(key_action(*key, true, active));
            expected.push(key_event(*key, "Press", None));
        }
        if !shortcut {
            actions.extend([key_action(a, true, mask), key_action(a, false, mask)]);
            expected.extend([
                key_event(a, "Press", Some(mask)),
                key_event(a, "Release", Some(mask)),
            ]);
        }
        for key in native.iter().rev() {
            active &= !modifier(key.key);
            actions.push(key_action(*key, false, active));
            expected.push(key_event(*key, "Release", None));
        }
        let mut held = group.clone();
        if !shortcut {
            held.push(Key::A);
        }
        cases.push(case(&name, "modifiers", actions, expected, held, false));
        // Mouse callbacks preserve the same modifier combination without key shortcuts.
        let mut actions = Vec::new();
        let mut expected = Vec::new();
        let mut active = 0;
        for key in &native {
            active |= modifier(key.key);
            actions.push(key_action(*key, true, active));
            expected.push(key_event(*key, "Press", None));
        }
        actions.extend([
            mouse_action(true, mask, backend),
            mouse_action(false, mask, backend),
        ]);
        expected.extend([mouse_event("Press", mask), mouse_event("Release", mask)]);
        for key in native.iter().rev() {
            active &= !modifier(key.key);
            actions.push(key_action(*key, false, active));
            expected.push(key_event(*key, "Release", None));
        }
        cases.push(case(
            &format!("mouse-{name}"),
            "mouse-modifiers",
            actions,
            expected,
            group,
            true,
        ));
    }
}

fn lock_cases(keys: &[NativeKey], backend: &str, cases: &mut Vec<Value>, rows: &mut Vec<Value>) {
    let find = |key| keys.iter().find(|n| n.key == key).copied();
    let a = find(Key::A).expect("A injectable");
    for lock in [Key::CapsLock, Key::NumLock] {
        let name = format!("lock-modifiers-{lock:?}");
        if backend == "cocoa" {
            rows.push(json!({"case":name,"status":"not yet qualified","reason":"Core Graphics flags/lock restoration require a dedicated qualified lock-state injector"}));
            continue;
        }
        let Some(key) = find(lock) else {
            rows.push(json!({"case":name,"status":"not injectable","reason":"lock key absent from active map"}));
            continue;
        };
        let actions = vec![
            key_action(key, true, 0),
            key_action(key, false, 0),
            key_action(a, true, 0),
            key_action(a, false, 0),
            mouse_action(true, 0, backend),
            mouse_action(false, 0, backend),
            key_action(key, true, 0),
            key_action(key, false, 0),
            key_action(a, true, 0),
            key_action(a, false, 0),
            mouse_action(true, 0, backend),
            mouse_action(false, 0, backend),
        ];
        let expected = vec![
            key_event(key, "Press", None),
            key_event(key, "Release", None),
            key_event(a, "Press", Some(0)),
            key_event(a, "Release", Some(0)),
            mouse_event("Press", 0),
            mouse_event("Release", 0),
            key_event(key, "Press", None),
            key_event(key, "Release", None),
            key_event(a, "Press", Some(0)),
            key_event(a, "Release", Some(0)),
            mouse_event("Press", 0),
            mouse_event("Release", 0),
        ];
        let mut spec = case(
            &name,
            "lock-modifiers",
            actions,
            expected,
            vec![lock, Key::A],
            true,
        );
        spec["lock_mods"] = json!(true);
        spec["lock_bit"] = json!(if lock == Key::CapsLock { 16 } else { 32 });
        cases.push(spec);
    }
}

fn modifier(key: Key) -> i32 {
    match key {
        Key::LeftShift | Key::RightShift => 1,
        Key::LeftControl | Key::RightControl => 2,
        Key::LeftAlt | Key::RightAlt => 4,
        Key::LeftSuper | Key::RightSuper => 8,
        _ => 0,
    }
}
fn key_action(key: NativeKey, down: bool, modifiers: i32) -> Value {
    json!({"kind":"key","code":key.code,"down":down,"modifiers":modifiers,"pause":key.key==Key::Pause,"lock":matches!(key.key,Key::CapsLock|Key::NumLock|Key::ScrollLock)})
}
fn key_event(key: NativeKey, action: &str, modifiers: Option<i32>) -> Value {
    let mut value =
        json!({"kind":"key","key":key.key as i32,"scancode":key.scancode,"action":action});
    if let Some(mods) = modifiers {
        value["modifiers"] = json!(mods);
    }
    value
}
fn mouse_action(down: bool, modifiers: i32, backend: &str) -> Value {
    json!({"kind":"mouse","code":i32::from(matches!(backend,"x11"|"wayland")),"down":down,"modifiers":modifiers})
}
fn mouse_event(action: &str, modifiers: i32) -> Value {
    json!({"kind":"mouse","button":0,"action":action,"modifiers":modifiers})
}
fn case(
    name: &str,
    family: &str,
    actions: Vec<Value>,
    expected: Vec<Value>,
    keys: Vec<Key>,
    mouse: bool,
) -> Value {
    json!({"case":name,"family":family,"actions":Value::Array(actions),"expected":Value::Array(expected),"keys":keys.into_iter().map(|k|k as i32).collect::<Vec<_>>(),"mouse":mouse,"modifier_mask":15})
}

fn available_keys(backend: &str) -> Result<(Vec<NativeKey>, Vec<Value>), String> {
    #[cfg(target_os = "linux")]
    let names = xkb_names()?;
    let mut keys = Vec::new();
    let mut rows = Vec::new();
    for mapping in KEYS {
        let found = match backend {
            "win32" => mapping.win32.map(|c| (c, c)),
            "cocoa" => mapping.cocoa.map(|c| (c, c)),
            #[cfg(target_os = "linux")]
            "x11" | "wayland" => input_keys::linux_key(mapping, backend, &names),
            _ => None,
        };
        let mut reason = if found.is_none() {
            Some("physical key has no backend mapping or is absent from the active XKB map")
        } else {
            None
        };
        if backend == "win32" && matches!(mapping.key, Key::LeftSuper | Key::RightSuper) {
            reason=Some("Windows opens Start on bare Win-key release; this profile cannot deliver the release to the focused probe reliably");
        }
        if backend == "cocoa" && mapping.key == Key::CapsLock {
            reason=Some("Caps Lock uses flagsChanged and persistent OS lock state; dedicated restoration is not yet qualified");
        }
        let status = if backend == "cocoa" && mapping.key == Key::CapsLock {
            "not yet qualified"
        } else if reason.is_some() {
            "not injectable"
        } else {
            "eligible"
        };
        rows.push(json!({"case":format!("key-{:?}",mapping.key),"key":mapping.key as i32,"status":status,"reason":reason,
            "native_code":found.map(|n|n.0),"scancode":found.map(|n|n.1)}));
        if let Some((code, scancode)) = found.filter(|_| reason.is_none()) {
            keys.push(NativeKey {
                key: mapping.key,
                code,
                scancode,
            });
        }
    }
    Ok((keys, rows))
}

#[cfg(target_os = "linux")]
fn xkb_names() -> Result<Vec<(String, u16)>, String> {
    use x11rb::protocol::xkb::{ConnectionExt as _, NameDetail, ID};
    let (connection, _) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let version = connection
        .xkb_use_extension(1, 0)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    if !version.supported {
        return Err("XKB 1.0 required to resolve physical key names".to_owned());
    }
    let names = connection
        .xkb_get_names(
            ID::USE_CORE_KBD.into(),
            NameDetail::KEY_NAMES | NameDetail::KEY_ALIASES,
        )
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    let raw = names
        .value_list
        .key_names
        .ok_or("XKB omitted physical key names")?;
    let mut entries = Vec::new();
    for (index, name) in raw.into_iter().enumerate() {
        let code = u16::from(names.first_key) + u16::try_from(index).map_err(|e| e.to_string())?;
        entries.push((
            String::from_utf8_lossy(&name.name)
                .trim_end_matches('\0')
                .to_owned(),
            code,
        ));
    }
    if let Some(aliases) = names.value_list.key_aliases {
        for alias in aliases {
            let real = String::from_utf8_lossy(&alias.real)
                .trim_end_matches('\0')
                .to_owned();
            if let Some((_, code)) = entries.iter().find(|(name, _)| *name == real) {
                entries.push((
                    String::from_utf8_lossy(&alias.alias)
                        .trim_end_matches('\0')
                        .to_owned(),
                    *code,
                ));
            }
        }
    }
    Ok(entries)
}

fn text_cases(
    backend: &str,
    keys: &[NativeKey],
    cases: &mut Vec<Value>,
    rows: &mut Vec<Value>,
) -> Result<(), String> {
    for text in ["a", "é", "🦀"] {
        let expected = vec![
            json!({"kind":"char-mods","text":text,"modifiers":0}),
            json!({"kind":"char","text":text}),
        ];
        let actions = match backend {
            "win32" => text
                .encode_utf16()
                .flat_map(|unit| {
                    [
                        json!({"kind":"text","code":unit,"down":true}),
                        json!({"kind":"text","code":unit,"down":false}),
                    ]
                })
                .collect(),
            "cocoa" => vec![
                json!({"kind":"text","code":0,"text":text,"down":true}),
                json!({"kind":"text","code":0,"text":text,"down":false}),
            ],
            #[cfg(target_os = "linux")]
            "x11" | "wayland" => {
                let scalar = text.chars().next().ok_or("text fixture empty")? as u32;
                let Some(code) = text_keycode(scalar)? else {
                    rows.push(json!({"case":format!("text-{text}"),"status":"not injectable","reason":"scalar absent at unmodified group-1/level-1 in active parent X11 map; no layout mutation or input method is configured"}));
                    continue;
                };
                vec![
                    json!({"kind":"key","code":code,"down":true}),
                    json!({"kind":"key","code":code,"down":false}),
                ]
            }
            _ => return Err("no text injector".to_owned()),
        };
        let tracked = if backend == "cocoa" {
            vec![Key::A]
        } else if matches!(backend, "x11" | "wayland") {
            keys.iter()
                .filter(|k| actions.first().is_some_and(|a| a["code"] == k.code))
                .map(|k| k.key)
                .collect()
        } else {
            Vec::new()
        };
        let spec = case(
            &format!("text-{text}"),
            "text",
            actions,
            expected,
            tracked,
            false,
        );
        cases.push(spec);
    }
    Ok(())
}
#[cfg(target_os = "linux")]
fn text_keycode(scalar: u32) -> Result<Option<u16>, String> {
    use x11rb::{connection::Connection as _, protocol::xproto::ConnectionExt as _};
    let (connection, _) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let setup = connection.setup();
    let count = setup.max_keycode - setup.min_keycode + 1;
    let map = connection
        .get_keyboard_mapping(setup.min_keycode, count)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    if map.keysyms_per_keycode == 0 {
        return Err("X11 keysym mapping empty".to_owned());
    }
    let symbol = if scalar <= 0xff {
        scalar
    } else {
        0x0100_0000 | scalar
    };
    map.keysyms
        .chunks(usize::from(map.keysyms_per_keycode))
        .position(|row| row.first() == Some(&symbol))
        .map(|i| {
            u16::try_from(i)
                .map(|offset| u16::from(setup.min_keycode) + offset)
                .map_err(|e| e.to_string())
        })
        .transpose()
}
