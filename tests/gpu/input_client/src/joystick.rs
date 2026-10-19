// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public VMNL observations; native device creation and injection belong to the driver.

mod physical;

use std::{
    cell::RefCell,
    io::{self, BufRead as _, Write as _},
    rc::Rc,
    thread,
    time::{Duration, Instant},
};
use vmnl::{
    Context, Event, EventKind, GamepadButton, InputRuntime, JoystickButtonState, JoystickId,
    JoystickSample, JoystickStatus, VMNLError, VMNLErrorKind, VMNLResult, Window,
};

const TIMEOUT: Duration = Duration::from_secs(5);
const GUID: &str = "030000000df000009400000091000000";
const BUTTONS: [GamepadButton; 15] = [
    GamepadButton::A,
    GamepadButton::B,
    GamepadButton::X,
    GamepadButton::Y,
    GamepadButton::LeftBumper,
    GamepadButton::RightBumper,
    GamepadButton::Back,
    GamepadButton::Start,
    GamepadButton::Guide,
    GamepadButton::LeftThumb,
    GamepadButton::RightThumb,
    GamepadButton::DpadUp,
    GamepadButton::DpadRight,
    GamepadButton::DpadDown,
    GamepadButton::DpadLeft,
];
const MAPPING: &str = "x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,back:b6,start:b7,guide:b8,leftstick:b9,rightstick:b10,dpup:b11,dpright:b12,dpdown:b13,dpleft:b14,leftx:a0,lefty:a1,rightx:a2,righty:a3,lefttrigger:a4,righttrigger:a5,platform:Linux,";

type CallbackTrace = Rc<RefCell<Vec<(Event, usize, thread::ThreadId)>>>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mapping {
    None,
    Normal,
    Swapped,
    Hat,
}

#[derive(Clone, Debug)]
struct Expected {
    present: bool,
    mapping: Mapping,
    buttons: [bool; 15],
    axes: [f32; 6],
    hat: u8,
}

impl Default for Expected {
    fn default() -> Self {
        Self {
            present: false,
            mapping: Mapping::None,
            buttons: [false; 15],
            axes: [0.0; 6],
            hat: 0,
        }
    }
}

impl Expected {
    fn mapped_buttons(&self) -> [bool; 15] {
        if !self.present || self.mapping == Mapping::None {
            return [false; 15];
        }
        let mut buttons = self.buttons;
        if self.mapping == Mapping::Swapped {
            buttons.swap(0, 1);
        }
        if self.mapping == Mapping::Hat {
            buttons[11..].copy_from_slice(&[1, 2, 4, 8].map(|bit| self.hat & bit != 0));
        }
        buttons
    }

    fn matches(&self, sample: Option<&JoystickSample>, hats_as_buttons: bool, name: &str) -> bool {
        let Some(sample) = sample else {
            return !self.present;
        };
        if !self.present
            || sample.name() != Some(name)
            || sample.guid() != Some(GUID)
            || sample.axes().len() != 6
            || sample.hats().len() != 1
            || sample.hats()[0].as_raw() != self.hat
            || sample.buttons().len() != 15 + if hats_as_buttons { 4 } else { 0 }
            || !axes_match(sample.axes(), &self.axes)
        {
            return false;
        }
        let mut raw: Vec<_> = self.buttons.into_iter().collect();
        if hats_as_buttons {
            raw.extend([1, 2, 4, 8].map(|bit| self.hat & bit != 0));
        }
        if !sample
            .buttons()
            .iter()
            .zip(raw)
            .all(|(actual, expected)| button_matches(*actual, expected))
        {
            return false;
        }
        match (sample.gamepad(), self.mapping) {
            (None, Mapping::None) => true,
            (Some(gamepad), Mapping::Normal | Mapping::Swapped | Mapping::Hat) => {
                gamepad.name() == Some("VMNL Controlled")
                    && axes_match(gamepad.axes(), &self.axes)
                    && gamepad
                        .buttons()
                        .iter()
                        .zip(self.mapped_buttons())
                        .all(|(actual, expected)| button_matches(*actual, expected))
            }
            _ => false,
        }
    }
}

fn button_matches(actual: JoystickButtonState, pressed: bool) -> bool {
    actual
        == if pressed {
            JoystickButtonState::Pressed
        } else {
            JoystickButtonState::Released
        }
}

fn axes_match(actual: &[f32], expected: &[f32]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(a, b)| a.is_finite() && (*a - *b).abs() <= 0.0001)
}

#[derive(Clone, Debug, PartialEq)]
struct Snapshot {
    status: JoystickStatus,
    sample: Option<JoystickSample>,
    buttons: [[bool; 3]; 15],
}

fn snapshot(window: &Window, id: JoystickId) -> Snapshot {
    let state = window.input().joystick(id);
    Snapshot {
        status: state.status(),
        sample: state.sample().cloned(),
        buttons: BUTTONS.map(|button| {
            [
                state.is_down(button),
                state.is_pressed(button),
                state.is_released(button),
            ]
        }),
    }
}

fn all_snapshots(windows: &[Window]) -> Vec<Vec<Snapshot>> {
    windows
        .iter()
        .map(|window| {
            (0..JoystickId::COUNT)
                .filter_map(JoystickId::from_index)
                .map(|id| snapshot(window, id))
                .collect()
        })
        .collect()
}

struct Client {
    first: InputRuntime,
    second: InputRuntime,
    windows: Vec<Window>,
    _context: Option<Context>,
    callbacks: CallbackTrace,
    // Retain the allocation until every runtime/window owner has dropped.
    pointer: Box<u64>,
    name: String,
    id: Option<JoystickId>,
    hat_buttons: bool,
    reverse: bool,
    deliveries: [bool; 4],
    tracking: [bool; 4],
    expected: Expected,
    original: Option<JoystickSample>,
}

impl Client {
    fn new(windowed: bool, hat_buttons: bool, reverse: bool, name: String) -> VMNLResult<Self> {
        let mut first = InputRuntime::builder().hat_buttons(hat_buttons).build()?;
        let mut second = InputRuntime::acquire()?;
        require(
            first.configuration().hat_buttons() == hat_buttons,
            "wrong resolved hat_buttons",
        )?;
        let callbacks: CallbackTrace = Rc::default();
        let captured = Rc::clone(&callbacks);
        second.set_joystick_callback(move |event, pointer| {
            captured
                .borrow_mut()
                .push((event.clone(), pointer as usize, thread::current().id()));
        });
        let _ = first.poll_events();
        let _ = second.poll_events();
        callbacks.borrow_mut().clear();
        let context = if windowed {
            Some(Context::new()?)
        } else {
            None
        };
        let mut windows = Vec::new();
        if let Some(context) = &context {
            for (tracking, delivery) in [(true, true), (true, true), (true, false), (false, true)] {
                let mut window = Window::builder()
                    .title("VMNL virtual joystick")
                    .size(160, 120)
                    .joystick_tracking(tracking)
                    .joystick_event_delivery(delivery)
                    .build(context)?;
                // Joystick state is independent of keyboard focus and drawable visibility.
                window.hide();
                let _ = window.poll_events();
                windows.push(window);
            }
        }
        Ok(Self {
            first,
            second,
            windows,
            _context: context,
            callbacks,
            pointer: Box::new(94),
            name,
            id: None,
            hat_buttons,
            reverse,
            deliveries: [true, true, false, true],
            tracking: [true, true, true, false],
            expected: Expected::default(),
            original: None,
        })
    }

    fn action(&mut self, parts: &[&str]) -> VMNLResult<()> {
        let previous_present = self.expected.present;
        let mut fresh_tracking = false;
        let window_first = matches!(parts, ["WINDOW_CONNECT"] | ["WINDOW_DISCONNECT"]);
        require(
            !window_first || !self.windows.is_empty(),
            "window-first action requires windows",
        )?;
        let pumping_window = usize::from(self.reverse);
        match parts {
            ["CONNECT"] | ["RECONNECT"] | ["WINDOW_CONNECT"] => {
                self.id = None;
                self.expected.present = true;
                self.expected.buttons = [false; 15];
                self.expected.axes = [0.0; 6];
                self.expected.hat = 0;
            }
            ["MAP"] => {
                self.mapping(Mapping::Normal)?;
            }
            ["MAP_HAT"] => {
                self.mapping(Mapping::Hat)?;
            }
            ["SWAP"] => {
                self.mapping(Mapping::Swapped)?;
            }
            ["LOSS"] => {
                self.mapping(Mapping::None)?;
            }
            ["BUTTON", index, pressed] => {
                let index = index.parse::<usize>().map_err(invalid)?;
                require(
                    index < 15 && matches!(*pressed, "0" | "1"),
                    "invalid public button command",
                )?;
                self.expected.buttons[index] = *pressed == "1";
            }
            ["AXES"] => {
                self.expected.axes = [-1.0, -0.5, 0.0, 0.25, 0.75, 1.0];
            }
            ["AXES_ALT"] => {
                self.expected.axes = [1.0, 0.5, -0.75, -0.25, -1.0, 0.75];
            }
            ["AXES_MIN"] => {
                self.expected.axes = [-1.0; 6];
            }
            ["AXES_MAX"] => {
                self.expected.axes = [1.0; 6];
            }
            ["LATE"] => {
                let id = self
                    .id
                    .ok_or_else(|| invalid("late subscription requires a discovered fixture"))?;
                let before = all_snapshots(&self.windows);
                let mut late = InputRuntime::acquire()?;
                check_events(&late.poll_events(), id, None)?;
                require(
                    self.expected.matches(
                        late.sample_joystick(id)?.as_ref(),
                        self.hat_buttons,
                        &self.name,
                    ),
                    "late subscriber sample differs",
                )?;
                require(
                    all_snapshots(&self.windows) == before,
                    "late runtime advanced window snapshots",
                )?;
            }
            ["HAT", mask] => {
                self.expected.hat = mask.parse().map_err(invalid)?;
                require(
                    [0, 1, 2, 4, 8, 3, 6, 9, 12].contains(&self.expected.hat),
                    "invalid public hat mask",
                )?;
            }
            ["NEUTRAL"] => {
                self.expected.buttons = [false; 15];
                self.expected.axes = [0.0; 6];
                self.expected.hat = 0;
            }
            ["TRACK_OFF"] | ["TRACK_ON"] => {
                require(
                    !self.windows.is_empty(),
                    "tracking action requires a window",
                )?;
                if let Some(window) = self.windows.first_mut() {
                    let enabled = parts[0] == "TRACK_ON";
                    window.set_joystick_tracking(enabled)?;
                    self.tracking[0] = enabled;
                    fresh_tracking = enabled;
                    require(
                        !enabled || window.is_joystick_tracking_enabled(),
                        "tracking did not enable",
                    )?;
                    if !enabled {
                        let id = self.id.ok_or_else(|| invalid("no fixture slot"))?;
                        let value = snapshot(window, id);
                        require(
                            value.status == JoystickStatus::NotTracked
                                && value.sample.is_none()
                                && value.buttons == [[false; 3]; 15],
                            "tracking disable did not immediately clear state",
                        )?;
                    }
                }
            }
            ["EVENTS_OFF"] | ["EVENTS_ON"] => {
                require(
                    !self.windows.is_empty(),
                    "delivery action requires a window",
                )?;
                if let Some(window) = self.windows.first_mut() {
                    self.deliveries[0] = parts[0] == "EVENTS_ON";
                    window.set_joystick_event_delivery(self.deliveries[0]);
                    require(
                        window.is_joystick_event_delivery_enabled() == self.deliveries[0],
                        "wrong event delivery setting",
                    )?;
                }
            }
            ["DISCONNECT"] | ["WINDOW_DISCONNECT"] => {
                self.expected.present = false;
            }
            ["EMPTY"] => {}
            _ => return Err(invalid("unknown joystick action")),
        }
        let before = all_snapshots(&self.windows);
        let mut events = Vec::new();
        let deadline = Instant::now() + TIMEOUT;
        let observed = loop {
            if window_first {
                events.extend(self.windows[pumping_window].poll_events());
            } else if self.reverse {
                events.extend(self.second.poll_events());
            } else {
                events.extend(self.first.poll_events());
            }
            if self.id.is_none() {
                let mut matches = Vec::new();
                for index in 0..JoystickId::COUNT {
                    let id = JoystickId::from_index(index)
                        .ok_or_else(|| invalid("invalid public slot"))?;
                    let sample = if window_first {
                        self.windows[pumping_window]
                            .input()
                            .joystick(id)
                            .sample()
                            .cloned()
                    } else {
                        self.first.sample_joystick(id)?
                    };
                    if sample.is_some_and(|sample| sample.name() == Some(&self.name)) {
                        matches.push(id);
                    }
                }
                require(matches.len() <= 1, "ambiguous fixture identity")?;
                self.id = matches.first().copied();
            }
            let sample = if window_first {
                self.id.and_then(|id| {
                    self.windows[pumping_window]
                        .input()
                        .joystick(id)
                        .sample()
                        .cloned()
                })
            } else {
                self.id
                    .map(|id| self.first.sample_joystick(id))
                    .transpose()?
                    .flatten()
            };
            if self.id.is_some()
                && self
                    .expected
                    .matches(sample.as_ref(), self.hat_buttons, &self.name)
            {
                break sample;
            }
            require(
                Instant::now() < deadline,
                &format!(
                    "native observation timed out: expected={:?}, sample={sample:?}, slot={:?}",
                    self.expected, self.id
                ),
            )?;
            thread::sleep(Duration::from_millis(5));
        };
        let id = self
            .id
            .ok_or_else(|| invalid("fixture was not discovered"))?;
        if let Some(sample) = &observed {
            require(
                sample.id() == id,
                "sample identity differs from the queried slot",
            )?;
        }
        let after_pump = all_snapshots(&self.windows);
        for (index, snapshots) in after_pump.iter().enumerate() {
            require(
                (window_first && index == pumping_window) || *snapshots == before[index],
                "pumping a consumer advanced another window",
            )?;
        }
        let connection =
            (previous_present != self.expected.present).then_some(self.expected.present);
        check_events(&events, id, connection)?;
        if window_first {
            let first = self.first.poll_events();
            check_events(&first, id, connection)?;
            require(
                fixture_events(&first, id) == fixture_events(&events, id),
                "window poll consumed/changed runtime notifications",
            )?;
        }
        // The other subscriber has not been drained by the pumping consumer.
        let other = if self.reverse && !window_first {
            self.first.poll_events()
        } else {
            self.second.poll_events()
        };
        check_events(&other, id, connection)?;
        require(
            fixture_events(&events, id) == fixture_events(&other, id),
            "subscribers received different event identities/timestamps",
        )?;
        check_events(&self.first.poll_events(), id, None)?;
        check_events(&self.second.poll_events(), id, None)?;
        require(
            all_snapshots(&self.windows) == after_pump,
            "runtime consumer advanced window snapshots",
        )?;
        self.check_callbacks(id, connection, &events)?;
        if connection == Some(true) {
            let pointer = std::ptr::from_mut(self.pointer.as_mut()).cast();
            // SAFETY: this main-thread-owned Box remains allocated until every runtime/window
            // drops; callbacks only compare its address and never retain Rust references.
            unsafe {
                self.first.set_joystick_user_pointer(id, pointer);
                require(
                    self.second.joystick_user_pointer(id) == pointer,
                    "native user pointer round-trip failed",
                )?;
            }
            if self.original.is_none() {
                self.original = observed.clone();
            }
        } else if connection == Some(false) {
            // SAFETY: only compare the returned address; never dereference a disconnected slot.
            require(
                unsafe { self.first.joystick_user_pointer(id).is_null() },
                "disconnect did not clear the slot pointer",
            )?;
        }
        self.check_windows(
            id,
            &before,
            connection,
            fresh_tracking,
            &events,
            window_first.then_some(pumping_window),
        )?;
        if let Some(original) = &self.original {
            let baseline = Expected {
                present: true,
                ..Expected::default()
            };
            require(
                baseline.matches(Some(original), self.hat_buttons, &self.name),
                "an owned original sample changed after injection/mapping/disconnect",
            )?;
        }
        Ok(())
    }

    fn mapping(&mut self, mapping: Mapping) -> VMNLResult<()> {
        let prefix = match mapping {
            Mapping::Normal | Mapping::Hat => "a:b0,b:b1,",
            Mapping::Swapped => "a:b1,b:b0,",
            Mapping::None => "a:b99,b:b1,",
        };
        let suffix = if mapping == Mapping::Hat {
            MAPPING.replace(
                "dpup:b11,dpright:b12,dpdown:b13,dpleft:b14,",
                "dpup:h0.1,dpright:h0.2,dpdown:h0.4,dpleft:h0.8,",
            )
        } else {
            MAPPING.to_owned()
        };
        self.first
            .update_gamepad_mappings(&format!("{GUID},VMNL Controlled,{prefix}{suffix}"))?;
        self.expected.mapping = mapping;
        Ok(())
    }

    fn check_callbacks(
        &self,
        id: JoystickId,
        connection: Option<bool>,
        expected_events: &[Event],
    ) -> VMNLResult<()> {
        let callbacks = self.callbacks.take();
        let relevant: Vec<_> = callbacks
            .iter()
            .filter(|(e, _, _)| event_slot(e) == Some(id))
            .collect();
        check_events(
            &relevant
                .iter()
                .map(|(e, _, _)| e.clone())
                .collect::<Vec<_>>(),
            id,
            connection,
        )?;
        require(
            relevant
                .iter()
                .map(|(event, _, _)| event)
                .eq(fixture_events(expected_events, id)),
            "callback/queued event identity differs",
        )?;
        for (_, pointer, thread_id) in relevant {
            require(
                *thread_id == thread::current().id(),
                "callback ran on another thread",
            )?;
            let expected = if connection == Some(false) {
                std::ptr::from_ref(self.pointer.as_ref()) as usize
            } else {
                0
            };
            require(
                *pointer == expected,
                "disconnect callback lost the live user pointer",
            )?;
        }
        Ok(())
    }

    fn check_windows(
        &mut self,
        id: JoystickId,
        before: &[Vec<Snapshot>],
        connection: Option<bool>,
        fresh: bool,
        expected_events: &[Event],
        already_polled: Option<usize>,
    ) -> VMNLResult<()> {
        let order = if self.reverse {
            [1, 0, 2, 3]
        } else {
            [0, 1, 2, 3]
        };
        for (position, index) in order.into_iter().enumerate() {
            if self.windows.is_empty() {
                break;
            }
            let window = &mut self.windows[index];
            let events = if already_polled == Some(index) {
                expected_events.to_vec()
            } else {
                window.poll_events()
            };
            check_events(
                &events,
                id,
                if self.deliveries[index] {
                    connection
                } else {
                    None
                },
            )?;
            if self.deliveries[index] {
                require(
                    fixture_events(&events, id) == fixture_events(expected_events, id),
                    "window queue changed event identity/timestamp",
                )?;
            }
            let state = window.input().joystick(id);
            if self.tracking[index] {
                require(
                    state.status()
                        == if self.expected.present {
                            JoystickStatus::Present
                        } else {
                            JoystickStatus::Absent
                        },
                    "wrong window presence",
                )?;
                require(
                    self.expected
                        .matches(state.sample(), self.hat_buttons, &self.name),
                    "window sample differs from injected values",
                )?;
                if let Some(sample) = state.sample() {
                    require(
                        sample.id() == id,
                        "window sample identity differs from its slot",
                    )?;
                }
                let down = self.expected.mapped_buttons();
                for (button_index, button) in BUTTONS.iter().copied().enumerate() {
                    let old = before[index][id.index()].buttons[button_index][0];
                    let initial = index == 0 && fresh;
                    require(state.is_down(button) == down[button_index]
                        && state.is_pressed(button) == (!initial && !old && down[button_index])
                        && state.is_released(button) == (!initial && old && !down[button_index]),
                        &format!("wrong per-window transition: window={index}, button={button:?}, old={old}, down={}, initial={initial}", down[button_index]))?;
                }
            } else {
                require(
                    state.status() == JoystickStatus::NotTracked && state.sample().is_none(),
                    "event-only/disabled window started tracking",
                )?;
                require(
                    BUTTONS.iter().all(|b| {
                        !state.is_down(*b) && !state.is_pressed(*b) && !state.is_released(*b)
                    }),
                    "disabled tracking retains held controls",
                )?;
            }
            require(
                snapshot(window, id) == snapshot(window, id),
                "queries consumed window transitions",
            )?;
            if position == 0 {
                let other = order[1];
                require(
                    snapshot(&self.windows[other], id) == before[other][id.index()],
                    "polling one window advanced another",
                )?;
            }
            let window = &mut self.windows[index];
            if index % 2 == 0 {
                let held = snapshot(window, id);
                window.clear_input_transitions();
                let cleared = snapshot(window, id);
                require(
                    cleared.status == held.status
                        && cleared.sample == held.sample
                        && cleared
                            .buttons
                            .iter()
                            .zip(held.buttons)
                            .all(|(new, old)| *new == [old[0], false, false]),
                    "explicit transition clear changed current state or retained flags",
                )?;
            }
            check_events(&window.poll_events(), id, None)?;
            require(
                BUTTONS.iter().all(|b| {
                    !window.input().joystick(id).is_pressed(*b)
                        && !window.input().joystick(id).is_released(*b)
                }),
                "empty batch retained transitions",
            )?;
            let held = snapshot(window, id);
            window.clear_input_transitions();
            require(
                snapshot(window, id) == held,
                "clearing transitions changed current state",
            )?;
        }
        Ok(())
    }
}

fn event_slot(event: &Event) -> Option<JoystickId> {
    match event.kind() {
        EventKind::JoystickConnected { id } | EventKind::JoystickDisconnected { id } => Some(*id),
        _ => None,
    }
}

fn fixture_events(events: &[Event], id: JoystickId) -> Vec<&Event> {
    events
        .iter()
        .filter(|event| event_slot(event) == Some(id))
        .collect()
}

fn check_events(events: &[Event], id: JoystickId, connected: Option<bool>) -> VMNLResult<()> {
    let events: Vec<_> = events
        .iter()
        .filter(|e| event_slot(e) == Some(id))
        .collect();
    require(
        events.len() == usize::from(connected.is_some()),
        &format!(
            "missing/duplicate/consumed notification: expected={connected:?}, events={events:?}"
        ),
    )?;
    if let Some(event) = events.first() {
        require(
            event.timestamp_seconds().is_finite() && event.timestamp_seconds() >= 0.0,
            "invalid dispatch timestamp",
        )?;
        require(
            matches!(
                (event.kind(), connected),
                (EventKind::JoystickConnected { .. }, Some(true))
                    | (EventKind::JoystickDisconnected { .. }, Some(false))
            ),
            "wrong connection event kind",
        )?;
    }
    Ok(())
}

pub(super) fn run() -> VMNLResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "physical") {
        return physical::run(&args[1..]);
    }
    require(
        args.len() == 4
            && matches!(args[0].as_str(), "runtime" | "window")
            && matches!(args[1].as_str(), "true" | "false")
            && matches!(args[2].as_str(), "ab" | "ba"),
        "expected runtime|window true|false ab|ba fixture-name",
    )?;
    let mut client = Client::new(
        args[0] == "window",
        args[1] == "true",
        args[2] == "ba",
        args[3].clone(),
    )?;
    emit("HELLO\tJOYSTICK\t1")?;
    let stdin = io::stdin();
    let mut commands = stdin.lock().lines();
    let mut sequence = 0_u64;
    while let Some(command) = commands.next() {
        let command = command.map_err(invalid)?;
        if command == "FINISH" {
            require(
                !client.expected.present,
                "finished before fixture disconnect",
            )?;
            emit("DONE")?;
            return Ok(());
        }
        sequence += 1;
        let parts: Vec<_> = command.split('\t').collect();
        require(
            parts.len() >= 3 && parts[0] == "ARM" && parts[1] == sequence.to_string(),
            "malformed/out-of-order ARM",
        )?;
        emit(&format!("READY\t{sequence}"))?;
        let injected = commands
            .next()
            .ok_or_else(|| invalid("driver disconnected before INJECTED"))?
            .map_err(invalid)?;
        require(
            injected == format!("INJECTED\t{sequence}"),
            "invalid injection barrier",
        )?;
        match client.action(&parts[2..]) {
            Ok(()) => {
                emit(&format!("PASS\t{sequence}\tObservation"))?;
                emit(&format!("OBSERVED\t{sequence}"))?;
            }
            Err(error) => {
                let reason: String = error
                    .to_string()
                    .chars()
                    .take(1500)
                    .map(|c| if c.is_control() { ' ' } else { c })
                    .collect();
                emit(&format!("FAIL\t{sequence}\tObservation\t{reason}"))?;
                return Err(error);
            }
        }
    }
    Err(invalid("driver disconnected before FINISH"))
}

fn emit(record: &str) -> VMNLResult<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{record}")
        .and_then(|()| stdout.flush())
        .map_err(invalid)
}
fn require(condition: bool, reason: &str) -> VMNLResult<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(reason))
    }
}
fn invalid(reason: impl std::fmt::Display) -> VMNLError {
    VMNLError::new(VMNLErrorKind::InvalidState(reason.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_oracle_rejects_missing_wrong_and_nonfinite_values() {
        assert!(axes_match(&[-1.0, 0.25, 1.0], &[-1.0, 0.25, 1.0]));
        for actual in [
            vec![],
            vec![0.0, 0.25, 1.0],
            vec![f32::NAN, 0.25, 1.0],
            vec![f32::INFINITY, 0.25, 1.0],
        ] {
            assert!(!axes_match(&actual, &[-1.0, 0.25, 1.0]));
        }
    }

    #[test]
    fn mapping_oracle_has_independent_button_and_loss_expectations() {
        let mut expected = Expected {
            present: true,
            mapping: Mapping::Normal,
            ..Expected::default()
        };
        expected.buttons[0] = true;
        assert!(expected.mapped_buttons()[0]);
        assert!(!expected.mapped_buttons()[1]);
        expected.mapping = Mapping::Swapped;
        assert!(!expected.mapped_buttons()[0]);
        assert!(expected.mapped_buttons()[1]);
        expected.mapping = Mapping::None;
        assert_eq!(expected.mapped_buttons(), [false; 15]);
        assert!(!expected.matches(None, false, "fixture"));
        expected.mapping = Mapping::Hat;
        expected.buttons = [false; 15];
        expected.hat = 9;
        assert_eq!(
            &expected.mapped_buttons()[11..],
            &[true, false, false, true]
        );
    }

    #[test]
    fn button_oracle_requires_exact_press_release_not_unknown_state() {
        assert!(button_matches(JoystickButtonState::Pressed, true));
        assert!(button_matches(JoystickButtonState::Released, false));
        assert!(!button_matches(JoystickButtonState::Released, true));
        assert!(!button_matches(JoystickButtonState::Pressed, false));
        assert!(!button_matches(JoystickButtonState::Other(7), false));
    }
}
