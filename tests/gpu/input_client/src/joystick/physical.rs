// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Operator-driven observations through the public VMNL API only.

use super::{emit, invalid, require, BUTTONS};
use std::{
    io::{self, BufRead as _},
    thread,
    time::{Duration, Instant},
};
use vmnl::{
    Context, EventKind, InputRuntime, JoystickButtonState, JoystickId, JoystickSample,
    JoystickStatus, VMNLResult, Window,
};

const OBSERVATION_TIMEOUT: Duration = Duration::from_secs(5);
const STABLE_TIME: Duration = Duration::from_millis(150);

#[derive(Debug)]
enum Action {
    Select(usize),
    Button(usize, bool, Option<usize>),
    Axis(usize, f32, f32, Option<(usize, f32)>),
    Hat(usize, u8),
    Disconnect,
    Empty,
}

fn parse(parts: &[&str]) -> VMNLResult<Action> {
    let index = |s: &str| s.parse::<usize>().map_err(invalid);
    match parts {
        ["SELECT", slot] if index(slot)? < JoystickId::COUNT => Ok(Action::Select(index(slot)?)),
        ["BUTTON", raw, down @ ("0" | "1"), mapped] => {
            let mapped = if *mapped == "-" {
                None
            } else {
                Some(index(mapped)?)
            };
            require(
                mapped.is_none_or(|i| i < BUTTONS.len()),
                "invalid mapped button",
            )?;
            Ok(Action::Button(index(raw)?, *down == "1", mapped))
        }
        ["AXIS", raw, expected, tolerance, mapped @ ..] => {
            let value = expected.parse::<f32>().map_err(invalid)?;
            let tolerance = tolerance.parse::<f32>().map_err(invalid)?;
            require(
                value.is_finite()
                    && (-1.0..=1.0).contains(&value)
                    && tolerance.is_finite()
                    && tolerance > 0.0
                    && tolerance <= 0.25,
                "invalid axis expectation/tolerance",
            )?;
            let mapped = match mapped {
                [] => None,
                [mapped, expected] => {
                    let value = expected.parse::<f32>().map_err(invalid)?;
                    require(
                        index(mapped)? < 6 && value.is_finite() && (-1.0..=1.0).contains(&value),
                        "invalid mapped axis expectation",
                    )?;
                    Some((index(mapped)?, value))
                }
                _ => return Err(invalid("invalid mapped axis fields")),
            };
            Ok(Action::Axis(index(raw)?, value, tolerance, mapped))
        }
        ["HAT", raw, mask] => {
            let mask = mask.parse::<u8>().map_err(invalid)?;
            require(
                [0, 1, 2, 4, 8, 3, 6, 9, 12].contains(&mask),
                "invalid hat direction",
            )?;
            Ok(Action::Hat(index(raw)?, mask))
        }
        ["DISCONNECT"] => Ok(Action::Disconnect),
        ["EMPTY"] => Ok(Action::Empty),
        _ => Err(invalid("invalid physical action")),
    }
}

fn matches(action: &Action, sample: &JoystickSample) -> bool {
    match *action {
        Action::Button(index, down, mapped) => {
            sample.buttons().get(index).is_some_and(|b| {
                *b == if down {
                    JoystickButtonState::Pressed
                } else {
                    JoystickButtonState::Released
                }
            }) && mapped.is_none_or(|i| {
                sample.gamepad().is_some_and(|g| {
                    g.buttons()[i]
                        == if down {
                            JoystickButtonState::Pressed
                        } else {
                            JoystickButtonState::Released
                        }
                })
            })
        }
        Action::Axis(index, expected, tolerance, mapped) => {
            sample
                .axes()
                .get(index)
                .is_some_and(|actual| axis_matches(*actual, expected, tolerance))
                && mapped.is_none_or(|(i, expected)| {
                    sample
                        .gamepad()
                        .is_some_and(|g| axis_matches(g.axes()[i], expected, tolerance))
                })
        }
        Action::Hat(index, mask) => sample.hats().get(index).is_some_and(|h| h.as_raw() == mask),
        _ => false,
    }
}

fn axis_matches(actual: f32, expected: f32, tolerance: f32) -> bool {
    actual.is_finite() && (-1.0..=1.0).contains(&actual) && (actual - expected).abs() <= tolerance
}

fn hex(value: Option<&str>) -> String {
    value
        .unwrap_or_default()
        .bytes()
        .map(|b| format!("{b:02x}"))
        .collect()
}

struct Session {
    runtime: InputRuntime,
    window: Option<Window>,
    _context: Option<Context>,
    catalog: Vec<JoystickSample>,
    selected: Option<JoystickSample>,
    disconnected: bool,
}

impl Session {
    fn new(windowed: bool) -> VMNLResult<Self> {
        let mut runtime = InputRuntime::builder().hat_buttons(false).build()?;
        let _ = runtime.poll_events();
        let context = if windowed {
            Some(Context::new()?)
        } else {
            None
        };
        let mut window = context
            .as_ref()
            .map(|context| {
                Window::builder()
                    .title("VMNL physical joystick")
                    .size(320, 180)
                    .joystick_tracking(true)
                    .joystick_event_delivery(true)
                    .build(context)
            })
            .transpose()?;
        if let Some(window) = &mut window {
            window.hide();
            let _ = window.poll_events();
        }
        let mut catalog = Vec::new();
        for index in 0..JoystickId::COUNT {
            if let Some(sample) = runtime.sample_joystick(
                JoystickId::from_index(index).ok_or_else(|| invalid("invalid slot"))?,
            )? {
                catalog.push(sample);
            }
        }
        Ok(Self {
            runtime,
            window,
            _context: context,
            catalog,
            selected: None,
            disconnected: false,
        })
    }

    fn action(&mut self, action: &Action) -> VMNLResult<()> {
        if let Action::Select(slot) = *action {
            require(self.selected.is_none(), "device already selected")?;
            let sample = self
                .catalog
                .iter()
                .find(|s| s.id().index() == slot)
                .ok_or_else(|| invalid("slot not listed in catalog"))?;
            let current = self
                .runtime
                .sample_joystick(sample.id())?
                .ok_or_else(|| invalid("selected device disappeared"))?;
            require(identity(sample, &current), "selected slot changed identity")?;
            self.selected = Some(current);
            return Ok(());
        }
        let original = self
            .selected
            .as_ref()
            .ok_or_else(|| invalid("select a device first"))?
            .clone();
        require(
            !self.disconnected || matches!(action, Action::Empty),
            "device already disconnected",
        )?;
        let id = original.id();
        let held = self
            .window
            .as_ref()
            .map(|w| BUTTONS.map(|b| w.input().joystick(id).is_down(b)))
            .unwrap_or([false; 15]);
        let changed = match *action {
            Action::Button(_, down, Some(i)) => Some((i, held[i] != down, down)),
            _ => None,
        };
        let mut edge_seen = false;
        let mut releases = [false; 15];
        let mut runtime_events = 0;
        let mut window_events = 0;
        let mut stable_since = None;
        let deadline = Instant::now() + OBSERVATION_TIMEOUT;
        while Instant::now() < deadline {
            runtime_events += self.runtime.poll_events().iter().filter(|e|
                matches!(e.kind(), EventKind::JoystickDisconnected { id: slot } if *slot == id)).count();
            if let Some(window) = &mut self.window {
                window_events += window.poll_events().iter().filter(|e|
                    matches!(e.kind(), EventKind::JoystickDisconnected { id: slot } if *slot == id)).count();
                let state = window.input().joystick(id);
                for (i, b) in BUTTONS.into_iter().enumerate() {
                    releases[i] |= state.is_released(b);
                }
                if let Some((i, _, down)) = changed {
                    edge_seen |= if down {
                        state.is_pressed(BUTTONS[i])
                    } else {
                        state.is_released(BUTTONS[i])
                    };
                }
            }
            let sample = self.runtime.sample_joystick(id)?;
            if let Some(sample) = &sample {
                require(identity(&original, sample), "selected slot was replaced")?;
            }
            let absent = matches!(action, Action::Disconnect | Action::Empty);
            let mut observed = if absent {
                sample.is_none()
            } else {
                sample.as_ref().is_some_and(|s| matches(action, s))
            };
            if let Some(window) = &self.window {
                let state = window.input().joystick(id);
                observed &= if absent {
                    state.status() == JoystickStatus::Absent
                        && state.sample().is_none()
                        && BUTTONS.into_iter().all(|b| !state.is_down(b))
                } else {
                    state
                        .sample()
                        .is_some_and(|s| identity(&original, s) && matches(action, s))
                };
                if changed.is_some_and(|(_, needs_edge, _)| needs_edge) {
                    observed &= edge_seen;
                }
                if matches!(action, Action::Disconnect) {
                    observed &= held
                        .iter()
                        .zip(releases)
                        .all(|(held, released)| !*held || released);
                }
                if matches!(action, Action::Empty) {
                    observed &= BUTTONS
                        .into_iter()
                        .all(|b| !state.is_pressed(b) && !state.is_released(b));
                }
            }
            if matches!(action, Action::Disconnect) {
                require(
                    runtime_events <= 1 && window_events <= 1,
                    "duplicate disconnect event",
                )?;
                observed &= runtime_events == 1 && (self.window.is_none() || window_events == 1);
            }
            if matches!(action, Action::Empty) {
                require(
                    runtime_events == 0 && window_events == 0,
                    "disconnect event replayed in next batch",
                )?;
            }
            if observed {
                let since = stable_since.get_or_insert_with(Instant::now);
                if since.elapsed() >= STABLE_TIME {
                    self.disconnected |= matches!(action, Action::Disconnect);
                    return Ok(());
                }
            } else {
                stable_since = None;
            }
            thread::sleep(Duration::from_millis(5));
        }
        Err(invalid(format!(
            "physical observation timed out: {action:?}"
        )))
    }
}

fn identity(first: &JoystickSample, current: &JoystickSample) -> bool {
    first.id() == current.id()
        && first.name() == current.name()
        && first.guid() == current.guid()
        && first.axes().len() == current.axes().len()
        && first.buttons().len() == current.buttons().len()
        && first.hats().len() == current.hats().len()
}

pub(super) fn run(args: &[String]) -> VMNLResult<()> {
    require(
        args.len() == 1 && matches!(args[0].as_str(), "runtime" | "window"),
        "expected physical runtime|window",
    )?;
    let mut session = Session::new(args[0] == "window")?;
    emit("HELLO\tPHYSICAL\t1")?;
    for sample in &session.catalog {
        emit(&format!(
            "DEVICE\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sample.id().index(),
            hex(sample.name()),
            hex(sample.guid()),
            sample.buttons().len(),
            sample.axes().len(),
            sample.hats().len(),
            sample.gamepad().is_some()
        ))?;
    }
    emit("CATALOG_END")?;
    let stdin = io::stdin();
    let mut commands = stdin.lock().lines();
    let mut sequence = 0_u64;
    while let Some(line) = commands.next() {
        let line = line.map_err(invalid)?;
        if line == "FINISH" {
            require(
                session.disconnected,
                "finish requires an observed physical disconnection",
            )?;
            emit("DONE")?;
            return Ok(());
        }
        sequence += 1;
        let parts: Vec<_> = line.split('\t').collect();
        require(
            parts.len() >= 3 && parts[0] == "ARM" && parts[1] == sequence.to_string(),
            "invalid physical ARM",
        )?;
        let action = parse(&parts[2..])?;
        emit(&format!("READY\t{sequence}"))?;
        let barrier = commands
            .next()
            .ok_or_else(|| invalid("driver disconnected"))?
            .map_err(invalid)?;
        require(
            barrier == format!("INJECTED\t{sequence}"),
            "invalid physical barrier",
        )?;
        if let Err(error) = session.action(&action) {
            emit(&format!("FAIL\t{sequence}\tObservation\t{error}"))?;
            return Err(error);
        }
        emit(&format!("PASS\t{sequence}\tObservation"))?;
        emit(&format!("OBSERVED\t{sequence}"))?;
    }
    Err(invalid("driver disconnected before FINISH"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_physical_actions_and_nonfinite_tolerances() {
        for command in [
            "SELECT\t16",
            "BUTTON\t0\t2\t-",
            "BUTTON\t0\t1\t15",
            "AXIS\t0\tNaN\t0.1",
            "AXIS\t0\t0\t0",
            "AXIS\t0\t0\tinf",
            "AXIS\t0\t0\t0.1\t6\t1",
            "AXIS\t0\t0\t0.1\t0\tNaN",
            "HAT\t0\t15",
            "UNKNOWN",
        ] {
            assert!(
                parse(&command.split('\t').collect::<Vec<_>>()).is_err(),
                "{command}"
            );
        }
        assert!(parse(&["SELECT", "7"]).is_ok());
        assert!(parse(&["BUTTON", "2", "1", "0"]).is_ok());
        assert!(parse(&["AXIS", "0", "-1", "0.1", "2", "1"]).is_ok());
    }
    #[test]
    fn axis_oracle_is_bounded_and_uses_explicit_tolerance() {
        assert!(axis_matches(0.04, 0.0, 0.1));
        for actual in [0.2, f32::NAN, f32::INFINITY, 1.01] {
            assert!(!axis_matches(actual, 0.0, 0.1));
        }
        assert!(!axis_matches(0.0, 1.0, 0.1));
    }
}
