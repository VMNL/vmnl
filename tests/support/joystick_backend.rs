// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shared joystick backend validation and bounded Wayland evidence.

use super::native_input_process::ClientProcess;
use std::{
    process::Command,
    time::{Duration, Instant},
};

pub(super) fn validate_backend(backend: &str, host: &str) -> Result<(), String> {
    match (backend, host) {
        ("x11" | "wayland", "linux") | ("win32", "windows") | ("cocoa", "macos") => Ok(()),
        _ => Err(format!(
            "joystick backend {backend:?} does not match host {host:?}"
        )),
    }
}

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

pub(super) fn observe_wayland(
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

pub(super) fn configure_wayland(command: &mut Command) {
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::super::native_input_report::InputTestReport;
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
    #[cfg(target_os = "linux")]
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
    fn backend_requires_matching_native_host() {
        for host in ["linux", "windows", "macos", "unknown"] {
            for backend in ["x11", "wayland", "win32", "cocoa", "null"] {
                let expected = matches!(
                    (backend, host),
                    ("x11" | "wayland", "linux") | ("win32", "windows") | ("cocoa", "macos")
                );
                assert_eq!(validate_backend(backend, host).is_ok(), expected);
            }
        }
    }
}
