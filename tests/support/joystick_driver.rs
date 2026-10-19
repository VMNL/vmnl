// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Reuse the #91 bounded transport/report with an owned Linux uinput fixture.

use super::{
    native_input_process::ClientProcess, native_input_report::InputTestReport,
    virtual_joystick::VirtualJoystick,
};
use std::{
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    process::Command,
    time::{Duration, Instant},
};

fn wayland_roundtrip(trace: &str) -> Option<Vec<String>> {
    [
        ("wl_display", ".get_registry(", true),
        ("wl_registry", ".global(", false),
        ("wl_callback", ".done(", false),
    ]
    .into_iter()
    .map(|(interface, operation, outgoing)| {
        trace.lines().find_map(|line| {
            let (head, _) = line.split_once(operation)?;
            let object = head.split_whitespace().last()?.strip_prefix(interface)?;
            let id = object
                .strip_prefix('#')
                .or_else(|| object.strip_prefix('@'))?;
            (id.parse::<u32>().is_ok() && head.contains("->") == outgoing).then(|| line.to_owned())
        })
    })
    .collect()
}

fn observe_wayland(
    client: &ClientProcess,
    profile: &str,
    timeout: Duration,
) -> Result<serde_json::Value, String> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(evidence) = wayland_roundtrip(&client.diagnostics()?) {
            return Ok(serde_json::json!({
                "profile": profile,
                "client_pid": client.id(),
                "observed_backend": "wayland",
                "method": "client stderr WAYLAND_DEBUG=client, DISPLAY unset",
                "protocol": evidence,
            }));
        }
        if Instant::now() >= deadline {
            return Err(
                "client supplied no complete Wayland protocol roundtrip before deadline".into(),
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

enum Injection {
    None,
    Connect,
    Disconnect,
    Button(u16, bool),
    Axes([i32; 6]),
    Hat(i32, i32),
    Neutral,
}

struct Case {
    name: String,
    command: String,
    injection: Injection,
}

#[allow(clippy::too_many_lines)] // Declarative scenario inventory in injection order.
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    let mut add = |name: &str, command: &str, injection| {
        cases.push(Case {
            name: name.into(),
            command: command.into(),
            injection,
        });
    };
    add(
        "Connect and identify unmapped device",
        "CONNECT",
        Injection::Connect,
    );
    add("Controlled mapping", "MAP", Injection::None);
    add(
        "Late subscriber does not replay connection",
        "LATE",
        Injection::None,
    );
    for index in 0..15 {
        add(
            &format!("Button {index} press"),
            &format!("BUTTON\t{index}\t1"),
            Injection::Button(index, true),
        );
        add(
            &format!("Button {index} release"),
            &format!("BUTTON\t{index}\t0"),
            Injection::Button(index, false),
        );
    }
    add(
        "Six distinct axis values",
        "AXES",
        Injection::Axes([-1000, -500, 0, 250, 750, 1000]),
    );
    add(
        "Six alternate axis values",
        "AXES_ALT",
        Injection::Axes([1000, 500, -750, -250, -1000, 750]),
    );
    add(
        "All axes at minimum",
        "AXES_MIN",
        Injection::Axes([-1000; 6]),
    );
    add(
        "All axes at maximum",
        "AXES_MAX",
        Injection::Axes([1000; 6]),
    );
    add("Map hat to gamepad Dpad", "MAP_HAT", Injection::None);
    // Literal Linux coordinates and independent public hat bitmasks.
    for (name, x, y, mask) in [
        ("Centered", 0, 0, 0),
        ("Up", 0, -1, 1),
        ("Right", 1, 0, 2),
        ("Down", 0, 1, 4),
        ("Left", -1, 0, 8),
        ("Right up", 1, -1, 3),
        ("Right down", 1, 1, 6),
        ("Left up", -1, -1, 9),
        ("Left down", -1, 1, 12),
    ] {
        add(
            &format!("Hat {name}"),
            &format!("HAT\t{mask}"),
            Injection::Hat(x, y),
        );
    }
    add("Restore button Dpad mapping", "MAP", Injection::None);
    add("Neutral raw state", "NEUTRAL", Injection::Neutral);
    add("Hold mapped A", "BUTTON\t0\t1", Injection::Button(0, true));
    add(
        "Replace mapping with A/B permutation",
        "SWAP",
        Injection::None,
    );
    add(
        "Mapping unavailable with raw button held",
        "LOSS",
        Injection::None,
    );
    add(
        "Restore mapping while raw button held",
        "MAP",
        Injection::None,
    );
    add("Disable tracking immediately", "TRACK_OFF", Injection::None);
    add(
        "Re-enable tracking without invented press",
        "TRACK_ON",
        Injection::None,
    );
    add(
        "Disable event delivery independently",
        "EVENTS_OFF",
        Injection::None,
    );
    add("Disconnect while held", "DISCONNECT", Injection::Disconnect);
    add("Next absent batch clears release", "EMPTY", Injection::None);
    add(
        "Enable delivery without replay",
        "EVENTS_ON",
        Injection::None,
    );
    add(
        "Reconnect by identity without slot assumption",
        "RECONNECT",
        Injection::Connect,
    );
    add(
        "Reconnected mapped press",
        "BUTTON\t0\t1",
        Injection::Button(0, true),
    );
    add(
        "Reconnected mapped release",
        "BUTTON\t0\t0",
        Injection::Button(0, false),
    );
    add(
        "Disconnect released fixture",
        "DISCONNECT",
        Injection::Disconnect,
    );
    add(
        "Window-first connection fan-out",
        "WINDOW_CONNECT",
        Injection::Connect,
    );
    add(
        "Window-first disconnection fan-out",
        "WINDOW_DISCONNECT",
        Injection::Disconnect,
    );
    cases
}

fn start_client(
    mode: &str,
    hats: bool,
    order: &str,
    wayland: bool,
) -> Result<(VirtualJoystick, ClientProcess, String), String> {
    let fixture = VirtualJoystick::open()?;
    let path = std::env::var_os("VMNL_JOYSTICK_CLIENT")
        .ok_or("VMNL_JOYSTICK_CLIENT is unset; use just joystick-test <backend> <mode>")?;
    let name = format!("VMNL virtual joystick {}", std::process::id());
    let mut command = Command::new(path);
    command.args([mode, if hats { "true" } else { "false" }, order, &name]);
    command.env_remove("SDL_GAMECONTROLLERCONFIG");
    if wayland {
        command.env_remove("DISPLAY");
        command.env_remove("WAYLAND_SOCKET");
        command.env_remove("FORCE_COLOR");
        command.env("NO_COLOR", "1");
        command.env("XDG_SESSION_TYPE", "wayland");
        command.env("WAYLAND_DEBUG", "client");
        command.env(
            "WAYLAND_DISPLAY",
            std::env::var_os("WAYLAND_DISPLAY").unwrap_or_else(|| "wayland-0".into()),
        );
    }
    let mut client = ClientProcess::spawn(&mut command)?;
    client.expect("HELLO\tJOYSTICK\t1", Duration::from_secs(15))?;
    Ok((fixture, client, name))
}

fn write_report(
    report: &InputTestReport,
    mode: &str,
    requested_backend: Option<&str>,
    observations: &[serde_json::Value],
) -> Result<(), String> {
    if let Some(path) = std::env::var_os("VMNL_JOYSTICK_REPORT") {
        let mut record = report.json(mode);
        record["requested_backend"] = serde_json::json!(requested_backend);
        record["backend_observations"] = serde_json::json!(observations);
        record["fixture_guid"] = serde_json::json!("030000000df000009400000091000000");
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?,
        )
        .map_err(|e| format!("could not write joystick report: {e}"))?;
    }
    Ok(())
}

#[allow(clippy::print_stderr)] // Terminal reporting is the purpose of this test driver.
pub(crate) fn run(mode: &str) -> Result<(), String> {
    let mut report = InputTestReport::default();
    let requested_backend = std::env::var("VMNL_JOYSTICK_BACKEND").ok();
    let wayland = requested_backend.as_deref() == Some("wayland");
    let mut backend_observations = Vec::new();
    let cases: Vec<_> = cases()
        .into_iter()
        .filter(|case| {
            mode == "window"
                || ![
                    "TRACK_OFF",
                    "TRACK_ON",
                    "EVENTS_OFF",
                    "EVENTS_ON",
                    "WINDOW_CONNECT",
                    "WINDOW_DISCONNECT",
                ]
                .contains(&case.command.as_str())
        })
        .collect();
    for hats in [false, true] {
        for order in ["ab", "ba"] {
            let profile = format!("{mode} / hat_buttons={hats} / order={order}");
            report.plan(&profile, "Environment", &["Readiness", "Client shutdown"]);
            if wayland {
                report.plan(&profile, "Environment", &["Wayland roundtrip"]);
            }
            for case in &cases {
                report.plan(&profile, &case.name, &["Observation"]);
            }
        }
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        for hats in [false, true] {
            for order in ["ab", "ba"] {
                let profile = format!("{mode} / hat_buttons={hats} / order={order}");
                let (mut fixture, mut client, name) =
                    report.run(&profile, "Environment", "Readiness", || {
                        start_client(mode, hats, order, wayland)
                    })?;
                if wayland {
                    backend_observations.push(report.run(
                        &profile,
                        "Environment",
                        "Wayland roundtrip",
                        || observe_wayland(&client, &profile, Duration::from_secs(5)),
                    )?);
                }
                for case in &cases {
                    client.observe(
                        &mut report,
                        &profile,
                        &case.name,
                        &["Observation"],
                        &case.command,
                        || match case.injection {
                            Injection::None => Ok(()),
                            Injection::Connect => {
                                let path = fixture.create(&name)?;
                                eprintln!("owned uinput fixture event node: {}", path.display());
                                Ok(())
                            }
                            Injection::Disconnect => fixture.destroy(),
                            Injection::Button(index, pressed) => fixture.button(index, pressed),
                            Injection::Axes(values) => fixture.axes(values),
                            Injection::Hat(x, y) => fixture.hat(x, y),
                            Injection::Neutral => fixture.neutral(),
                        },
                    )?;
                }
                report.run(&profile, "Environment", "Client shutdown", || {
                    client.finish(Duration::from_secs(5))
                })?;
            }
        }
        if report.is_complete() {
            Ok(())
        } else {
            Err("joystick report is incomplete".into())
        }
    }));
    eprintln!(
        "{}",
        report.render_for(
            "VMNL JOYSTICK TEST — Linux",
            "uinput → evdev → public VMNL client"
        )
    );
    write_report(
        &report,
        mode,
        requested_backend.as_deref(),
        &backend_observations,
    )?;
    match outcome {
        Ok(result) => result,
        Err(payload) => resume_unwind(payload),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn backend_evidence_requires_outgoing_request_and_incoming_responses() {
        for separator in ['#', '@'] {
            let request = format!(
                "[0.1] -> wl_display{separator}1.get_registry(new id wl_registry{separator}2)"
            );
            let global = format!("[0.2] wl_registry{separator}2.global(1, \"wl_compositor\", 6)");
            let done = format!("[0.3] wl_callback{separator}3.done(9)");
            let trace = format!("{request}\n{global}\n{done}\n");
            assert_eq!(
                wayland_roundtrip(&trace),
                Some(vec![request.clone(), global.clone(), done.clone()])
            );
            for incomplete in [
                String::new(),
                format!("{request}\n{global}"),
                format!("{global}\n{done}"),
                trace.replace("->", ""),
                trace.replace("wl_registry", "other_registry"),
            ] {
                assert!(wayland_roundtrip(&incomplete).is_none());
            }
        }
    }

    #[test]
    fn missing_backend_evidence_fails_instead_of_qualifying() {
        let mut command = Command::new("python3");
        command.args(["-c", "import sys\nprint('unrelated diagnostics', file=sys.stderr)\nprint('HELLO', flush=True)\ninput()"]);
        let mut client = ClientProcess::spawn(&mut command).expect("fixture starts");
        client
            .expect("HELLO", Duration::from_secs(5))
            .expect("fixture ready");
        let mut report = InputTestReport::default();
        report.plan("Environment", "Backend", &["Wayland roundtrip"]);
        assert!(report
            .run("Environment", "Backend", "Wayland roundtrip", || {
                observe_wayland(&client, "test", Duration::from_millis(20))
            })
            .is_err());
        assert_eq!(report.json("runtime")["qualified"], false);
    }

    #[test]
    fn plan_covers_every_button_and_holds_until_acknowledgement() {
        let cases = cases();
        for index in 0..15 {
            let press = cases
                .iter()
                .position(|c| c.command == format!("BUTTON\t{index}\t1"))
                .expect("press planned");
            assert_eq!(cases[press + 1].command, format!("BUTTON\t{index}\t0"));
        }
        assert_eq!(
            cases
                .iter()
                .filter(|c| matches!(c.injection, Injection::Hat(_, _)))
                .count(),
            9
        );
        assert!(
            cases
                .iter()
                .position(|c| c.command == "LOSS")
                .expect("loss")
                < cases
                    .iter()
                    .position(|c| c.command == "DISCONNECT")
                    .expect("disconnect")
        );
    }
}
