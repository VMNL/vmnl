// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Linux physical-controller operator protocol over the existing #91 transport.

use super::{
    joystick_driver, native_input_process::ClientProcess, native_input_report::InputTestReport,
    physical_joystick_profile,
};
use serde_json::{json, Value};
use std::{
    fs::File,
    io::{self, BufRead as _, IsTerminal as _, Read as _},
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    process::Command,
    sync::mpsc,
    thread,
    time::Duration,
};

struct Console(mpsc::Receiver<Result<String, String>>);

impl Console {
    fn new() -> Result<Self, String> {
        if !io::stdin().is_terminal() {
            return Err("physical qualification requires an operator terminal".into());
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut input = stdin.lock();
            loop {
                let mut line = String::new();
                let result = match input.by_ref().take(1025).read_line(&mut line) {
                    Ok(n) if n > 0 && n <= 1024 && line.ends_with('\n') => {
                        Ok(line.trim().to_owned())
                    }
                    Ok(_) => Err("operator input ended or exceeded 1024 bytes".into()),
                    Err(error) => Err(error.to_string()),
                };
                let failed = result.is_err();
                if sender.send(result).is_err() || failed {
                    break;
                }
            }
        });
        Ok(Self(receiver))
    }

    #[allow(clippy::print_stderr)]
    fn ask(&self, prompt: &str) -> Result<String, String> {
        eprintln!("{prompt}\nEntrée pour confirmer ; q pour arrêter. Délai : 45 s.");
        self.receive(Duration::from_secs(45))
    }

    fn receive(&self, timeout: Duration) -> Result<String, String> {
        let response = self
            .0
            .recv_timeout(timeout)
            .map_err(|error| format!("operator confirmation missing: {error}"))??;
        if response == "q" {
            return Err("operator cancelled physical qualification".into());
        }
        Ok(response)
    }
}

struct Device {
    slot: usize,
    name: String,
    guid: String,
    counts: [usize; 3],
    mapped: bool,
}

fn unhex(s: &str) -> Result<String, String> {
    if s.len() & 1 != 0 || !s.is_ascii() {
        return Err("invalid device string".into());
    }
    let bytes = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

fn device(record: &str) -> Result<Device, String> {
    let fields: Vec<_> = record.split('\t').collect();
    if fields.len() != 8 || fields[0] != "DEVICE" {
        return Err("invalid device catalog record".into());
    }
    let number = |s: &str| s.parse::<usize>().map_err(|e| e.to_string());
    let slot = number(fields[1])?;
    if slot >= 16 || !["true", "false"].contains(&fields[7]) {
        return Err("invalid catalog slot/mapping".into());
    }
    Ok(Device {
        slot,
        name: unhex(fields[2])?,
        guid: unhex(fields[3])?,
        counts: [number(fields[4])?, number(fields[5])?, number(fields[6])?],
        mapped: fields[7] == "true",
    })
}

fn catalog(client: &mut ClientProcess) -> Result<Vec<Device>, String> {
    let mut devices = Vec::new();
    for _ in 0..=16 {
        let record = client.receive(Duration::from_secs(5))?;
        if record == "CATALOG_END" {
            return if devices.is_empty() {
                Err("no joystick detected; physical qualification is unavailable".into())
            } else {
                Ok(devices)
            };
        }
        let device = device(&record)?;
        if devices.iter().any(|d: &Device| d.slot == device.slot) {
            return Err("duplicate catalog slot".into());
        }
        devices.push(device);
    }
    Err("device catalog exceeds 16 slots".into())
}

fn profile() -> Result<Value, String> {
    let path = std::env::var_os("VMNL_JOYSTICK_PHYSICAL_PROFILE")
        .ok_or("physical profile path missing")?;
    let mut text = String::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(65537)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() > 65536 {
        return Err("physical profile exceeds 64 KiB".into());
    }
    let profile = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    physical_joystick_profile::validate_identity(&profile)?;
    Ok(profile)
}

fn start(mode: &str) -> Result<ClientProcess, String> {
    let path = std::env::var_os("VMNL_JOYSTICK_CLIENT")
        .ok_or("use just joystick-physical-test <backend> <profile> <mode>")?;
    let mut command = Command::new(path);
    command
        .args(["physical", mode])
        .env_remove("SDL_GAMECONTROLLERCONFIG");
    if std::env::var("VMNL_JOYSTICK_BACKEND").as_deref() == Ok("wayland") {
        joystick_driver::configure_wayland(&mut command);
    }
    let mut client = ClientProcess::spawn(&mut command)?;
    client.expect("HELLO\tPHYSICAL\t1", Duration::from_secs(15))?;
    Ok(client)
}

#[allow(clippy::print_stderr)]
fn execute(mode: &str, report: &mut InputTestReport, metadata: &mut Value) -> Result<(), String> {
    let (profile, mut client, devices, console) =
        report.run("Physical", "Environment", "Readiness", || {
            let profile = profile()?;
            let mut client = start(mode)?;
            let devices = catalog(&mut client)?;
            Ok::<_, String>((profile, client, devices, Console::new()?))
        })?;
    metadata["profile"] = profile.clone();
    for d in &devices {
        eprintln!(
            "slot={} name={:?} GUID={} buttons={} axes={} hats={} mapped={}",
            d.slot, d.name, d.guid, d.counts[0], d.counts[1], d.counts[2], d.mapped
        );
    }
    let (selected, plan) = report.run("Physical", "Environment", "Selection/profile", || {
        if console.ask(&format!(
            "Confirme une manette PHYSIQUE {} via {} : tape PHYSICAL.",
            profile["model"], profile["connection"]
        ))? != "PHYSICAL"
        {
            return Err("physical device was not operator-confirmed".into());
        }
        metadata["operator_confirmed"] = json!(true);
        let slot = console
            .ask("Choisis le numéro de slot affiché (0..15).")?
            .parse::<usize>()
            .map_err(|e| e.to_string())?;
        let selected = devices
            .iter()
            .find(|d| d.slot == slot)
            .ok_or("slot not in catalog")?;
        if selected.name.starts_with("VMNL virtual joystick ") {
            return Err("uinput fixture cannot qualify a physical controller".into());
        }
        let plan = physical_joystick_profile::plan(&profile, selected.counts, selected.mapped)?;
        Ok::<_, String>((selected, plan))
    })?;
    metadata["device"] = json!({"slot":selected.slot,"name":selected.name,"guid":selected.guid,"raw_counts":selected.counts,"mapped":selected.mapped,"hat_buttons":false,"client_pid":client.id()});
    for step in &plan.steps {
        report.plan("Physical", &step.name, &["Observation"]);
    }
    for (control, reason) in &plan.exclusions {
        report.plan("Exclusions", control, &["Coverage"]);
        report.exclude("Exclusions", control, "Coverage", reason);
    }
    if std::env::var("VMNL_JOYSTICK_BACKEND").as_deref() == Ok("wayland") {
        report.plan("Physical", "Environment", &["Wayland roundtrip"]);
        metadata["backend_observations"] =
            json!([
                report.run("Physical", "Environment", "Wayland roundtrip", || {
                    joystick_driver::observe_wayland(&client, "physical", Duration::from_secs(5))
                })?
            ]);
    }
    client.observe(
        report,
        "Physical",
        "Selection",
        &["Observation"],
        &format!("SELECT\t{}", selected.slot),
        || Ok(()),
    )?;
    for step in plan.steps {
        client.observe(
            report,
            "Physical",
            &step.name,
            &["Observation"],
            &step.command,
            || {
                console.ask(&format!(
                    "{}\nMaintiens cet état après Entrée jusqu'au PASS affiché.",
                    step.prompt
                ))?;
                Ok(())
            },
        )?;
        eprintln!("{}: PASS", step.name);
    }
    report.run("Physical", "Environment", "Client shutdown", || {
        client.finish(Duration::from_secs(5))
    })?;
    Ok(())
}

#[allow(clippy::print_stderr)]
pub(crate) fn run(mode: &str) -> Result<(), String> {
    let mut report = InputTestReport::default();
    report.plan(
        "Physical",
        "Environment",
        &["Readiness", "Selection/profile", "Client shutdown"],
    );
    report.plan("Physical", "Selection", &["Observation"]);
    let mut metadata = json!({"operator_confirmed":false});
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        execute(mode, &mut report, &mut metadata)
    }));
    eprintln!(
        "{}",
        report.render_for(
            "VMNL PHYSICAL JOYSTICK TEST — Linux",
            "operator → physical controller → public VMNL client"
        )
    );
    eprintln!("Relâche toutes les commandes de la manette. Aucun périphérique physique ni droit hôte n'a été modifié.");
    if let Some(path) = std::env::var_os("VMNL_JOYSTICK_REPORT") {
        let mut record = report.json(mode);
        record["evidence"] = json!("physical-operator-declared");
        record["requested_backend"] = json!(std::env::var("VMNL_JOYSTICK_BACKEND").ok());
        record["physical"] = metadata;
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    match outcome {
        Ok(result) => result,
        Err(panic) => resume_unwind(panic),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    #[test]
    fn operator_cancellation_eof_and_timeout_fail() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let console = Console(receiver);
        assert!(console.receive(Duration::from_millis(1)).is_err());
        sender.send(Ok("q".into())).expect("cancel");
        assert!(console.receive(Duration::from_millis(1)).is_err());
        drop(sender);
        assert!(console.receive(Duration::from_millis(1)).is_err());
    }
    #[test]
    fn catalog_uses_reported_slot_and_rejects_malformed_identity() {
        let d = device("DEVICE\t7\t4162\t3031\t15\t6\t1\ttrue").expect("valid catalog");
        assert_eq!(d.slot, 7);
        assert_eq!(d.name, "Ab");
        for record in [
            "DEVICE\t16\t\t\t1\t1\t1\tfalse",
            "DEVICE\t0\tzz\t\t1\t1\t1\tfalse",
            "DEVICE\t0\t\t\t1\t1\t1\tmaybe",
            "OTHER",
        ] {
            assert!(device(record).is_err());
        }
    }
    #[test]
    fn absent_hardware_fails_instead_of_qualifying() {
        let mut command = Command::new("python3");
        command.args(["-c", "print('CATALOG_END', flush=True)"]);
        let mut client = ClientProcess::spawn(&mut command).expect("fake catalog starts");
        let mut report = InputTestReport::default();
        report.plan("Physical", "Environment", &["Readiness"]);
        assert!(report
            .run("Physical", "Environment", "Readiness", || catalog(
                &mut client
            ))
            .is_err());
        assert_eq!(report.json("runtime")["qualified"], false);
    }
}
