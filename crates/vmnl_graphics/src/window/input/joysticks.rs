// SPDX-FileCopyrightText: 2026 BOuhali Naouel
// SPDX-License-Identifier: MIT

//! Raw input, device metadata, and complete mapped gamepad snapshots.
//!
//! This module preserves mapped axes and tracks configurable directions and click transitions.
//! Presence is tracked even without a gamepad mapping; stick input requires one.

use crate::{Event, VMNLError, VMNLErrorKind, VMNLResult};
use glfw::{
    Action, GamepadAxis as GlfwAxis, GamepadButton as GlfwButton, GamepadState as GlfwState,
};

/// A stick direction in degrees or a stick click button.
///
/// Angles use the range `[0, 360)`. By default they are counterclockwise: right is 0 degrees,
/// up is 90, left is 180, and down is 270. [`StickSettings`] can change this convention.
/// This range is not enforced by
/// construction. A centered stick has no direction and uses `None`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Joystick {
    /// The left stick direction, or `None` when centered.
    JoystickLeft {
        /// Direction in degrees, following the enum's angle convention.
        degrees: Option<f32>,
    },
    /// The left Joystick Button.
    JoystickLeftButton,
    /// The right stick direction, or `None` when centered.
    JoystickRight {
        /// Direction in degrees, following the enum's angle convention.
        degrees: Option<f32>,
    },
    /// The right Joystick Button.
    JoystickRightButton,
}

/// Controls checked by aggregate input queries.
///
/// Direction values are selectors here, not live controller state.
pub(crate) const ALL_JOYSTICK: &[Joystick] = &[
    Joystick::JoystickLeft { degrees: None },
    Joystick::JoystickLeftButton,
    Joystick::JoystickRight { degrees: None },
    Joystick::JoystickRightButton,
];

/// Identifies a controller slot, not a permanent physical device.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JoystickId {
    /// Controller slot 1.
    Slot1,
    /// Controller slot 2.
    Slot2,
    /// Controller slot 3.
    Slot3,
    /// Controller slot 4.
    Slot4,
    /// Controller slot 5.
    Slot5,
    /// Controller slot 6.
    Slot6,
    /// Controller slot 7.
    Slot7,
    /// Controller slot 8.
    Slot8,
    /// Controller slot 9.
    Slot9,
    /// Controller slot 10.
    Slot10,
    /// Controller slot 11.
    Slot11,
    /// Controller slot 12.
    Slot12,
    /// Controller slot 13.
    Slot13,
    /// Controller slot 14.
    Slot14,
    /// Controller slot 15.
    Slot15,
    /// Controller slot 16.
    Slot16,
}

impl JoystickId {
    /// All device slots in ascending order. Slots can be reused after disconnection.
    pub const ALL: [Self; 16] = [
        Self::Slot1,
        Self::Slot2,
        Self::Slot3,
        Self::Slot4,
        Self::Slot5,
        Self::Slot6,
        Self::Slot7,
        Self::Slot8,
        Self::Slot9,
        Self::Slot10,
        Self::Slot11,
        Self::Slot12,
        Self::Slot13,
        Self::Slot14,
        Self::Slot15,
        Self::Slot16,
    ];

    /// Convert Id of the Joystick to a storage position
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Convert Id ro GLFW type
    pub(crate) const fn to_glfw(self) -> glfw::JoystickId {
        match self {
            Self::Slot1 => glfw::JoystickId::Joystick1,
            Self::Slot2 => glfw::JoystickId::Joystick2,
            Self::Slot3 => glfw::JoystickId::Joystick3,
            Self::Slot4 => glfw::JoystickId::Joystick4,
            Self::Slot5 => glfw::JoystickId::Joystick5,
            Self::Slot6 => glfw::JoystickId::Joystick6,
            Self::Slot7 => glfw::JoystickId::Joystick7,
            Self::Slot8 => glfw::JoystickId::Joystick8,
            Self::Slot9 => glfw::JoystickId::Joystick9,
            Self::Slot10 => glfw::JoystickId::Joystick10,
            Self::Slot11 => glfw::JoystickId::Joystick11,
            Self::Slot12 => glfw::JoystickId::Joystick12,
            Self::Slot13 => glfw::JoystickId::Joystick13,
            Self::Slot14 => glfw::JoystickId::Joystick14,
            Self::Slot15 => glfw::JoystickId::Joystick15,
            Self::Slot16 => glfw::JoystickId::Joystick16,
        }
    }
}

/// Direction of a raw joystick hat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HatState {
    /// No direction pressed
    Centered,
    /// Up direction pressed
    Up,
    /// Right direction pressed
    Right,
    /// Down direction pressed
    Down,
    /// Left direction pressed
    Left,
    /// Up + Right direction pressed
    UpRight,
    /// Down + Right direction pressed
    DownRight,
    /// Down + Left direction pressed
    DownLeft,
    /// Up + Left direction pressed
    UpLeft,
}

/// Named mapped gamepad buttons, in GLFW mapping order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    /// `A` control.
    A,
    /// `B` control.
    B,
    /// `X` control.
    X,
    /// `Y` control.
    Y,
    /// `LeftBumper` control.
    LeftBumper,
    /// `RightBumper` control.
    RightBumper,
    /// `Back` control.
    Back,
    /// `Start` control.
    Start,
    /// `Guide` control.
    Guide,
    /// `LeftThumb` control.
    LeftThumb,
    /// `RightThumb` control.
    RightThumb,
    /// `DpadUp` control.
    DpadUp,
    /// `DpadRight` control.
    DpadRight,
    /// `DpadDown` control.
    DpadDown,
    /// `DpadLeft` control.
    DpadLeft,
}
impl GamepadButton {
    /// Every mapped control, in storage order.
    pub const ALL: [Self; 15] = [
        Self::A,
        Self::B,
        Self::X,
        Self::Y,
        Self::LeftBumper,
        Self::RightBumper,
        Self::Back,
        Self::Start,
        Self::Guide,
        Self::LeftThumb,
        Self::RightThumb,
        Self::DpadUp,
        Self::DpadRight,
        Self::DpadDown,
        Self::DpadLeft,
    ];
    const fn to_glfw(self) -> GlfwButton {
        match self {
            Self::A => GlfwButton::ButtonA,
            Self::B => GlfwButton::ButtonB,
            Self::X => GlfwButton::ButtonX,
            Self::Y => GlfwButton::ButtonY,
            Self::LeftBumper => GlfwButton::ButtonLeftBumper,
            Self::RightBumper => GlfwButton::ButtonRightBumper,
            Self::Back => GlfwButton::ButtonBack,
            Self::Start => GlfwButton::ButtonStart,
            Self::Guide => GlfwButton::ButtonGuide,
            Self::LeftThumb => GlfwButton::ButtonLeftThumb,
            Self::RightThumb => GlfwButton::ButtonRightThumb,
            Self::DpadUp => GlfwButton::ButtonDpadUp,
            Self::DpadRight => GlfwButton::ButtonDpadRight,
            Self::DpadDown => GlfwButton::ButtonDpadDown,
            Self::DpadLeft => GlfwButton::ButtonDpadLeft,
        }
    }
}
/// Named mapped gamepad axes, in GLFW mapping order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    /// `LeftX` control.
    LeftX,
    /// `LeftY` control.
    LeftY,
    /// `RightX` control.
    RightX,
    /// `RightY` control.
    RightY,
    /// `LeftTrigger` control.
    LeftTrigger,
    /// `RightTrigger` control.
    RightTrigger,
}
impl GamepadAxis {
    /// Every mapped control, in storage order.
    pub const ALL: [Self; 6] = [
        Self::LeftX,
        Self::LeftY,
        Self::RightX,
        Self::RightY,
        Self::LeftTrigger,
        Self::RightTrigger,
    ];
    const fn to_glfw(self) -> GlfwAxis {
        match self {
            Self::LeftX => GlfwAxis::AxisLeftX,
            Self::LeftY => GlfwAxis::AxisLeftY,
            Self::RightX => GlfwAxis::AxisRightX,
            Self::RightY => GlfwAxis::AxisRightY,
            Self::LeftTrigger => GlfwAxis::AxisLeftTrigger,
            Self::RightTrigger => GlfwAxis::AxisRightTrigger,
        }
    }
}

/// Complete mapped snapshot. No dead zone, calibration, or clamping is applied.
/// Axes normally range from -1 to 1; X points right, Y down, and triggers run
/// from -1 (released) to 1 (pressed). Stored only when a mapped sample is available.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GamepadState {
    buttons: [bool; 15],
    axes: [f32; 6],
}
impl GamepadState {
    fn from_glfw(state: &GlfwState) -> Self {
        Self {
            buttons: GamepadButton::ALL
                .map(|button| state.get_button_state(button.to_glfw()) == Action::Press),
            axes: GamepadAxis::ALL.map(|axis| state.get_axis(axis.to_glfw())),
        }
    }
    /// All buttons in `GamepadButton::ALL` order; true means pressed.
    #[must_use]
    pub const fn buttons(&self) -> &[bool; 15] {
        &self.buttons
    }
    /// All unfiltered axes in `GamepadAxis::ALL` order.
    #[must_use]
    pub const fn axes(&self) -> &[f32; 6] {
        &self.axes
    }
    /// Whether a named button is down.
    #[must_use]
    pub const fn is_down(&self, button: GamepadButton) -> bool {
        self.buttons[button as usize]
    }
    /// Unfiltered sample for a named axis, including any non-finite backend value.
    #[must_use]
    pub const fn axis(&self, axis: GamepadAxis) -> f32 {
        self.axes[axis as usize]
    }
}

/// Device information sampled during polling. Strings are owned; cloning may allocate.
/// GUID identifies a mapping/device class, not a unique physical controller.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JoystickInfo {
    pub(crate) name: Option<String>,
    pub(crate) guid: Option<String>,
    pub(crate) gamepad_name: Option<String>,
    pub(crate) is_gamepad: bool,
}
impl JoystickInfo {
    /// Backend device name, if available.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// SDL-compatible mapping GUID, if available. GLFW exposes no separate mapping ID.
    #[must_use]
    pub fn guid(&self) -> Option<&str> {
        self.guid.as_deref()
    }
    /// Name from the active mapping, if available.
    #[must_use]
    pub fn gamepad_name(&self) -> Option<&str> {
        self.gamepad_name.as_deref()
    }
    /// Whether GLFW recognizes this device as a mapped gamepad.
    #[must_use]
    pub const fn is_gamepad(&self) -> bool {
        self.is_gamepad
    }
}

#[derive(Default)]
struct JoystickData(Option<Box<dyn std::any::Any>>);
impl std::fmt::Debug for JoystickData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoystickData")
            .field("is_set", &self.0.is_some())
            .finish()
    }
}

/// Number of sticks on one controller, not the number of connected controllers.
pub(crate) const JOYSTICK_COUNT: usize = 2;

/// Default radial threshold at or below which a stick is considered centered.
const STICK_DEAD_ZONE: f32 = 0.15;

/// Owned snapshot of a device's numbered axes, buttons, and hats.
///
/// Counts and ordering are device/backend dependent. Default is empty. Polling and
/// cloning may allocate vectors; getters only borrow. No gamepad mapping is required.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawJoystickState {
    axes: Vec<f32>,
    buttons: Vec<bool>,
    hats: Vec<HatState>,
}

impl RawJoystickState {
    pub(crate) fn from_glfw(axes: Vec<f32>, buttons: &[i32], hats: &[glfw::JoystickHats]) -> Self {
        Self {
            axes,
            buttons: buttons
                .iter()
                .map(|&button| button == glfw::ffi::GLFW_PRESS)
                .collect(),
            hats: hats
                .iter()
                .map(|hat| match hat.bits() {
                    glfw::ffi::GLFW_HAT_UP => HatState::Up,
                    glfw::ffi::GLFW_HAT_RIGHT => HatState::Right,
                    glfw::ffi::GLFW_HAT_DOWN => HatState::Down,
                    glfw::ffi::GLFW_HAT_LEFT => HatState::Left,
                    glfw::ffi::GLFW_HAT_RIGHT_UP => HatState::UpRight,
                    glfw::ffi::GLFW_HAT_RIGHT_DOWN => HatState::DownRight,
                    glfw::ffi::GLFW_HAT_LEFT_DOWN => HatState::DownLeft,
                    glfw::ffi::GLFW_HAT_LEFT_UP => HatState::UpLeft,
                    _ => HatState::Centered,
                })
                .collect(),
        }
    }
    /// Returns raw axes without VMNL filtering or clamping, normally in `[-1, 1]`.
    /// Axis meanings and orientation are device/backend dependent; non-finite values
    /// are retained if supplied by the backend.
    #[must_use]
    pub fn axes(&self) -> &[f32] {
        &self.axes
    }

    /// Returns button states; true means pressed.
    /// GLFW's default initialization also includes synthesized hat buttons.
    #[must_use]
    pub fn buttons(&self) -> &[bool] {
        &self.buttons
    }

    /// Returns the direction of each hat.
    #[must_use]
    pub fn hats(&self) -> &[HatState] {
        &self.hats
    }
}
/// Per-stick interpretation; original mapped axes are never filtered or overwritten.
///
/// Defaults use a radial dead zone of 0.15 and counterclockwise degrees from right.
/// Use the original axes for application-defined calibration and response curves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StickSettings {
    /// Radial threshold in mapped axis units. Must be finite and nonnegative.
    /// Zero disables filtering except at the exact center; values above 1 are allowed.
    pub dead_zone: f32,
    /// Direction used as zero, in counterclockwise degrees from right.
    /// Any finite value is accepted, modulo 360.
    pub zero_degrees: f32,
    /// Whether angles increase clockwise from the configured zero direction.
    pub clockwise: bool,
}

impl Default for StickSettings {
    fn default() -> Self {
        Self {
            dead_zone: STICK_DEAD_ZONE,
            zero_degrees: 0.0,
            clockwise: false,
        }
    }
}

impl StickSettings {
    fn validate(self) -> VMNLResult<()> {
        if !self.dead_zone.is_finite() || self.dead_zone < 0.0 || !self.zero_degrees.is_finite() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "stick settings require a finite nonnegative dead zone and a finite zero direction"
                    .into(),
            )));
        }
        Ok(())
    }
}

/// Original mapped axes, derived direction, and click state of one stick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StickState {
    /// Original mapped X/Y samples, including values inside the dead zone.
    axes: [f32; 2],
    /// Angle in degrees, or `None` when centered.
    degrees: Option<f32>,
    /// Whether the stick button is held down.
    clicked: bool,
}

impl StickState {
    /// Returns original mapped axes, without VMNL filtering, calibration, or clamping.
    ///
    /// X points right and Y points down. GLFW normally supplies each axis in `[-1, 1]`.
    /// Non-finite samples are retained here but have no derived direction.
    #[must_use]
    pub const fn axes(&self) -> [f32; 2] {
        self.axes
    }

    /// Returns the unfiltered Euclidean length, without clamping at 1.
    /// Non-finite samples follow floating-point `hypot` semantics.
    #[must_use]
    pub fn magnitude(&self) -> f32 {
        self.axes[0].hypot(self.axes[1])
    }

    /// Constructs a CPU-only snapshot from mapped axes using explicit settings.
    ///
    /// Axes are preserved exactly. Invalid settings return `InvalidState`; no GLFW
    /// calls or GPU work occur. Valid construction does not allocate.
    ///
    /// # Errors
    /// Returns `InvalidState` if the dead zone is negative or non-finite, or the
    /// zero direction is non-finite.
    pub fn with_axes(axes: [f32; 2], clicked: bool, settings: StickSettings) -> VMNLResult<Self> {
        settings.validate()?;
        Ok(Self::interpreted(axes, clicked, settings))
    }

    /// Returns the direction in `[0, 360)`, or `None` inside the dead zone.
    ///
    /// Defaults use right as 0 degrees, up as 90, left as 180, and down as 270.
    /// Explicit settings may change the zero direction and rotation sense.
    #[must_use]
    pub const fn degrees(&self) -> Option<f32> {
        self.degrees
    }

    /// Returns whether the stick click button is held down.
    #[must_use]
    pub const fn is_clicked(&self) -> bool {
        self.clicked
    }

    /// Converts normalized GLFW axes and a click button into one stick state.
    ///
    /// GLFW gamepad Y axes point down, so Y is inverted for counterclockwise angles.
    /// Non-finite coordinates are treated as centered. Clicks remain independent.
    #[cfg(test)]
    fn from_axes(x: f32, y: f32, clicked: bool) -> Self {
        Self::interpreted([x, y], clicked, StickSettings::default())
    }

    fn interpreted(axes: [f32; 2], clicked: bool, settings: StickSettings) -> Self {
        let [x, y] = axes;
        let degrees = if !x.is_finite() || !y.is_finite() || x.hypot(y) <= settings.dead_zone {
            None
        } else {
            let angle = (-y).atan2(x).to_degrees() - settings.zero_degrees.rem_euclid(360.0);
            let angle = (if settings.clockwise { -angle } else { angle }).rem_euclid(360.0);
            // Floating-point rounding can produce the excluded upper endpoint.
            Some(if angle >= 360.0 { 0.0 } else { angle })
        };
        Self {
            axes,
            degrees,
            clicked,
        }
    }
}

/// Raw input, metadata, application data, and current/previous mapped controls for one slot.
///
/// New states assume an absent device, centered sticks, and released click buttons.
/// A device already present on the first update produces a connection event. Updates use a
/// radial dead zone of 0.15 by default, replaceable per stick with `set_settings`.
/// Original mapped axes and their magnitude remain accessible inside the dead zone.
/// Queries select the left or right control; any angle carried by the selector
/// is ignored. Read the observed angle through `left().degrees()` or `right().degrees()`.
///
/// This snapshot does not poll devices or process window events. Transitions are
/// relative to the last update, so changes entirely between samples can be missed.
/// Application data is local to this snapshot/window, not shared across windows.
/// It is dropped on reset or an observed disconnect; stick settings survive both.
/// Arbitrary application data means this type is not Send or Sync.
#[derive(Debug, Default)]
pub struct JoystickState {
    raw: RawJoystickState,
    mapped: Option<GamepadState>,
    previous_mapped: Option<GamepadState>,
    info: JoystickInfo,
    user_data: JoystickData,
    settings: [StickSettings; JOYSTICK_COUNT],
    /// Whether GLFW reported the device present during the current update.
    connected: bool,
    /// Whether GLFW reported the device present during the previous update.
    previous_connected: bool,
    /// Current states: left stick, then right stick.
    current: [StickState; JOYSTICK_COUNT],
    /// States from the previous update, in the same order.
    previous: [StickState; JOYSTICK_COUNT],
}

impl JoystickState {
    /// Complete mapped sample, or None when disconnected, unmapped, or unavailable.
    #[must_use]
    pub const fn gamepad(&self) -> Option<&GamepadState> {
        self.mapped.as_ref()
    }

    /// Previous mapped sample; cleared by reset.
    #[must_use]
    pub const fn previous_gamepad(&self) -> Option<&GamepadState> {
        self.previous_mapped.as_ref()
    }

    /// Latest device metadata, empty when disconnected or reset.
    #[must_use]
    pub const fn info(&self) -> &JoystickInfo {
        &self.info
    }

    /// Whether the named mapped button became pressed at the last poll.
    #[must_use]
    pub fn is_gamepad_pressed(&self, button: GamepadButton) -> bool {
        self.mapped.is_some_and(|s| s.is_down(button))
            && !self.previous_mapped.is_some_and(|s| s.is_down(button))
    }

    /// Whether the named mapped button was released, including mapping loss/disconnection.
    #[must_use]
    pub fn is_gamepad_released(&self, button: GamepadButton) -> bool {
        !self.mapped.is_some_and(|s| s.is_down(button))
            && self.previous_mapped.is_some_and(|s| s.is_down(button))
    }

    /// Stores one application-owned value for this window's device slot.
    /// Replacing, clearing, reset, or an observed disconnect drops the old value.
    /// This boxes the value and may allocate; no GLFW user pointer is exposed.
    pub fn set_user_data<T: 'static>(&mut self, value: T) {
        self.user_data.0 = Some(Box::new(value));
    }

    /// Borrows the attached value; None means absent or a different concrete type.
    #[must_use]
    pub fn user_data<T: 'static>(&self) -> Option<&T> {
        self.user_data.0.as_ref()?.downcast_ref()
    }

    /// Mutably borrows the attached value with the requested concrete type.
    pub fn user_data_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.user_data.0.as_mut()?.downcast_mut()
    }

    /// Drops attached application data without changing input or settings.
    pub fn clear_user_data(&mut self) {
        self.user_data.0 = None;
    }

    pub(crate) fn set_info(&mut self, info: JoystickInfo) {
        self.info = if self.connected {
            info
        } else {
            JoystickInfo::default()
        };
    }

    /// Returns the latest raw input, even without a gamepad mapping.
    ///
    /// New, disconnected, and reset snapshots contain empty slices. This does not
    /// poll hardware or allocate. Control ordering depends on the device/backend.
    #[must_use]
    pub const fn raw(&self) -> &RawJoystickState {
        &self.raw
    }

    /// Updates raw and mapped views from one polling pass (not an atomic hardware sample).
    pub(crate) fn update_with_raw(
        &mut self,
        connected: bool,
        gamepad: Option<&GlfwState>,
        raw: RawJoystickState,
    ) {
        self.update(connected, gamepad);
        self.raw = if connected {
            raw
        } else {
            RawJoystickState::default()
        };
    }

    /// Returns whether the device was present during the last input update.
    ///
    /// Presence does not require a gamepad mapping. This reads the stored snapshot
    /// without polling hardware. New and reset snapshots report `false`.
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        self.connected
    }

    /// Returns the resolved settings for the selected stick (button selectors also work).
    #[must_use]
    pub const fn settings(&self, joystick: Joystick) -> StickSettings {
        self.settings[Self::index(joystick)]
    }

    /// Reinterprets both snapshots using new settings, preserving axes and clicks.
    ///
    /// Both samples use the same settings for transition queries. This does not poll
    /// hardware or enqueue events, and already returned events remain unchanged.
    ///
    /// # Errors
    /// Invalid settings return `InvalidState` without changing any state: the dead
    /// zone must be finite and nonnegative and the zero direction must be finite.
    pub fn set_settings(&mut self, joystick: Joystick, settings: StickSettings) -> VMNLResult<()> {
        settings.validate()?;
        let index = Self::index(joystick);
        self.settings[index] = settings;
        for states in [&mut self.current, &mut self.previous] {
            let state = states[index];
            states[index] = StickState::interpreted(state.axes, state.clicked, settings);
        }
        Ok(())
    }

    /// Returns the index corresponding to a joystick control.
    ///
    /// # Arguments
    /// - `joystick`: The direction or click button identifying the stick.
    const fn index(joystick: Joystick) -> usize {
        match joystick {
            Joystick::JoystickLeft { .. } | Joystick::JoystickLeftButton => 0,
            Joystick::JoystickRight { .. } | Joystick::JoystickRightButton => 1,
        }
    }

    /// Updates device presence and both sticks from a GLFW gamepad snapshot.
    ///
    /// # Arguments
    /// - `connected`: Presence reported by `glfw::Joystick::is_present`, independent of mapping.
    /// - `gamepad`: The result of `glfw::Joystick::get_gamepad_state`, borrowed
    ///   with `as_ref()`. `None` means disconnected, unmapped, or unavailable.
    ///
    /// # Behavior
    /// Use one `JoystickState` per device and call once per input update. Missing
    /// snapshots clear current state and produce one update of release transitions.
    /// Reset before assigning this state to a different device. This conversion
    /// performs no GLFW calls, heap allocation, or GPU work.
    pub(crate) fn update(&mut self, connected: bool, gamepad: Option<&GlfwState>) {
        if !connected {
            self.raw = RawJoystickState::default();
            self.info = JoystickInfo::default();
            self.clear_user_data();
        }
        self.previous_connected = self.connected;
        self.connected = connected;
        self.previous_mapped = self.mapped;
        self.mapped = gamepad.filter(|_| connected).map(GamepadState::from_glfw);
        self.previous = self.current;
        self.current = match (connected, gamepad) {
            (true, Some(gamepad)) => [
                StickState::interpreted(
                    [
                        gamepad.get_axis(GlfwAxis::AxisLeftX),
                        gamepad.get_axis(GlfwAxis::AxisLeftY),
                    ],
                    gamepad.get_button_state(GlfwButton::ButtonLeftThumb) == Action::Press,
                    self.settings[0],
                ),
                StickState::interpreted(
                    [
                        gamepad.get_axis(GlfwAxis::AxisRightX),
                        gamepad.get_axis(GlfwAxis::AxisRightY),
                    ],
                    gamepad.get_button_state(GlfwButton::ButtonRightThumb) == Action::Press,
                    self.settings[1],
                ),
            ],
            _ => [StickState::default(); JOYSTICK_COUNT],
        };
    }

    /// Appends transitions from the last update to the window event batch.
    ///
    /// Call once after each update. Existing events are preserved, followed by
    /// a presence transition if any, then left stick changes and then right stick changes, with clicks before movement.
    /// Axis comparisons use sample bits, including changes inside the dead zone and
    /// magnitude-only changes. Identical NaN samples do not repeat events. Missing
    /// gamepad state releases clicks and centers sticks.
    #[cfg(test)]
    pub(crate) fn append_events(&self, id: JoystickId, events: &mut Vec<Event>) {
        self.append_events_after_connections(id, events, None);
    }

    pub(crate) fn append_events_after_connections(
        &self,
        id: JoystickId,
        events: &mut Vec<Event>,
        last_connection: Option<bool>,
    ) {
        let previous_connected = last_connection.unwrap_or(self.previous_connected);
        if self.connected && !previous_connected {
            events.push(Event::JoystickConnected { id });
        }
        if !self.connected && previous_connected {
            events.push(Event::JoystickDisconnected { id });
        }

        for (index, button) in [Joystick::JoystickLeftButton, Joystick::JoystickRightButton]
            .into_iter()
            .enumerate()
        {
            if self.is_pressed(button) {
                events.push(Event::JoystickButtonPressed {
                    id,
                    joystick: button,
                });
            }
            if self.is_released(button) {
                events.push(Event::JoystickButtonReleased {
                    id,
                    joystick: button,
                });
            }
            if self.current[index].axes.map(f32::to_bits)
                != self.previous[index].axes.map(f32::to_bits)
            {
                let degrees = self.current[index].degrees;
                let joystick = match button {
                    Joystick::JoystickLeftButton => Joystick::JoystickLeft { degrees },
                    _ => Joystick::JoystickRight { degrees },
                };
                events.push(Event::JoystickMoved {
                    id,
                    joystick,
                    axes: self.current[index].axes,
                });
            }
        }
    }

    /// Returns the current direction and click state of the left stick.
    #[must_use]
    pub const fn left(&self) -> &StickState {
        &self.current[0]
    }

    /// Returns the current direction and click state of the right stick.
    #[must_use]
    pub const fn right(&self) -> &StickState {
        &self.current[1]
    }

    /// Returns whether a selected control is active in a snapshot.
    const fn is_active(states: &[StickState; JOYSTICK_COUNT], joystick: Joystick) -> bool {
        let state = &states[Self::index(joystick)];
        match joystick {
            Joystick::JoystickLeft { .. } | Joystick::JoystickRight { .. } => {
                state.degrees.is_some()
            }
            Joystick::JoystickLeftButton | Joystick::JoystickRightButton => state.clicked,
        }
    }

    /// Returns whether the selected stick is tilted or its selected click button is held.
    ///
    /// # Arguments
    /// - `joystick`: The control to check. A direction selector's angle is ignored.
    #[must_use]
    pub const fn is_down(&self, joystick: Joystick) -> bool {
        Self::is_active(&self.current, joystick)
    }

    /// Returns whether the selected control became active during the last update.
    ///
    /// # Arguments
    /// - `joystick`: The control to check. For directions, this detects leaving
    ///   the dead zone, not angle changes while the stick remains tilted.
    #[must_use]
    pub const fn is_pressed(&self, joystick: Joystick) -> bool {
        Self::is_active(&self.current, joystick) && !Self::is_active(&self.previous, joystick)
    }

    /// Returns whether the selected control became inactive during the last update.
    ///
    /// # Arguments
    /// - `joystick`: The control to check. For directions, this detects returning
    ///   to the dead zone. Unavailable gamepads also release active controls.
    #[must_use]
    pub const fn is_released(&self, joystick: Joystick) -> bool {
        !Self::is_active(&self.current, joystick) && Self::is_active(&self.previous, joystick)
    }

    /// Returns whether any selected controls are active during the last update.
    ///
    /// # Arguments
    /// - `joysticks`: The controls to check. An empty slice returns `false`.
    #[must_use]
    pub fn is_any_down(&self, joysticks: &[Joystick]) -> bool {
        joysticks.iter().any(|&joystick| self.is_down(joystick))
    }

    /// Returns whether any selected controls became active during the last update.
    ///
    /// # Arguments
    /// - `joysticks`: The controls to check. An empty slice returns `false`.
    #[must_use]
    pub fn is_any_pressed(&self, joysticks: &[Joystick]) -> bool {
        joysticks.iter().any(|&joystick| self.is_pressed(joystick))
    }

    /// Returns whether any selected controls became inactive during the last update.
    ///
    /// # Arguments
    /// - `joysticks`: The controls to check. An empty slice returns `false`.
    #[must_use]
    pub fn is_any_released(&self, joysticks: &[Joystick]) -> bool {
        joysticks.iter().any(|&joystick| self.is_released(joystick))
    }

    /// Returns whether any selected controls are active or were released during the last update.
    ///
    /// # Arguments
    /// - `joysticks`: The controls to check. An empty slice returns `false`.
    #[must_use]
    pub fn is_any_used(&self, joysticks: &[Joystick]) -> bool {
        joysticks
            .iter()
            .any(|&joystick| self.is_down(joystick) || self.is_released(joystick))
    }

    /// Returns whether any stick direction or click button is active during the last update.
    #[must_use]
    pub fn is_one_down(&self) -> bool {
        self.is_any_down(ALL_JOYSTICK)
    }

    /// Returns whether any stick direction or click button became active during the last update.
    #[must_use]
    pub fn is_one_pressed(&self) -> bool {
        self.is_any_pressed(ALL_JOYSTICK)
    }

    /// Returns whether any stick direction or click button became inactive during the last update.
    #[must_use]
    pub fn is_one_released(&self) -> bool {
        self.is_any_released(ALL_JOYSTICK)
    }

    /// Returns whether any stick direction or click button is active or was released during the last update.
    #[must_use]
    pub fn is_one_used(&self) -> bool {
        self.is_any_used(ALL_JOYSTICK)
    }

    /// Clears both snapshots and presence history, preserving settings and producing no releases.
    ///
    /// A present device produces a connection event on the next update after reset.
    ///
    /// This can be useful when pausing or assigning the snapshot to another device.
    pub fn reset(&mut self) {
        *self = Self {
            settings: self.settings,
            ..Self::new()
        };
    }

    /// Creates a snapshot with centered sticks and released click buttons.
    ///
    /// Construction does not initialize GLFW or connect to a controller.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mapped_control_preserves_values_and_transitions() -> Result<(), &'static str> {
        let axes = [-0.8, 0.2, 0.6, -0.4, -1.0, 1.0];
        for (index, button) in GamepadButton::ALL.into_iter().enumerate() {
            let mut sample = glfw::ffi::GLFWgamepadstate {
                buttons: [0; 15],
                axes,
            };
            sample.buttons[index] = 1;
            let sample: GlfwState = sample.into();
            let mut state = JoystickState::new();
            state.update(true, Some(&sample));
            let mapped = state.gamepad().ok_or("mapped sample missing")?;
            assert_eq!(mapped.buttons().iter().filter(|&&v| v).count(), 1);
            assert!(mapped.is_down(button));
            for (axis, expected) in GamepadAxis::ALL.into_iter().zip(axes) {
                assert_eq!(mapped.axis(axis).to_bits(), expected.to_bits());
            }
            assert_eq!(mapped.axes().map(f32::to_bits), axes.map(f32::to_bits));
            assert!(state.is_gamepad_pressed(button));
            state.update(true, Some(&sample));
            assert!(!state.is_gamepad_pressed(button));
            state.update(true, None);
            assert!(state.gamepad().is_none());
            assert!(state.previous_gamepad().is_some());
            assert!(state.is_gamepad_released(button));
            state.update(false, Some(&sample));
            assert!(state.gamepad().is_none());
            assert!(!state.is_gamepad_released(button));
            state.reset();
            assert!(state.previous_gamepad().is_none());
        }
        Ok(())
    }

    #[test]
    fn metadata_and_application_data_follow_device_lifetime() -> Result<(), &'static str> {
        let mut state = JoystickState::new();
        let info = JoystickInfo {
            name: Some("Raw controller".into()),
            guid: Some("guid".into()),
            gamepad_name: None,
            is_gamepad: false,
        };
        state.update(true, None);
        state.set_info(info.clone());
        assert_eq!(state.info().name(), Some("Raw controller"));
        assert_eq!(state.info().guid(), Some("guid"));
        assert!(!state.info().is_gamepad());
        assert_eq!(state.info().gamepad_name(), None);
        state.set_user_data(42_u32);
        assert_eq!(state.user_data::<u32>(), Some(&42));
        assert_eq!(state.user_data::<String>(), None);
        *state
            .user_data_mut::<u32>()
            .ok_or("application data missing")? = 7;
        state.update(true, None);
        assert_eq!(state.user_data::<u32>(), Some(&7));
        state.update(false, None);
        state.set_info(info);
        assert_eq!(state.info(), &JoystickInfo::default());
        assert!(state.user_data::<u32>().is_none());
        state.set_user_data(9_u32);
        state.reset();
        assert!(state.user_data::<u32>().is_none());
        Ok(())
    }

    #[test]
    fn raw_conversion_preserves_axes_buttons_and_all_hat_directions() {
        let values = [0, 1, 2, 4, 8, 3, 6, 12, 9];
        let hats: Vec<_> = values
            .into_iter()
            .map(glfw::JoystickHats::from_bits_truncate)
            .collect();
        let axes = vec![0.01, -1.0, 1.0, f32::NAN];
        let raw = RawJoystickState::from_glfw(axes.clone(), &[0, 1, 0, 1], &hats);
        assert_eq!(
            raw.axes()
                .iter()
                .map(|axis| axis.to_bits())
                .collect::<Vec<_>>(),
            axes.iter().map(|axis| axis.to_bits()).collect::<Vec<_>>()
        );
        assert_eq!(raw.buttons(), &[false, true, false, true]);
        assert_eq!(
            raw.hats(),
            &[
                HatState::Centered,
                HatState::Up,
                HatState::Right,
                HatState::Down,
                HatState::Left,
                HatState::UpRight,
                HatState::DownRight,
                HatState::DownLeft,
                HatState::UpLeft
            ]
        );
    }

    #[test]
    fn raw_input_survives_mapping_loss_and_clears_on_disconnect_and_reset() {
        let mut state = JoystickState::new();
        let raw =
            RawJoystickState::from_glfw(vec![0.1, 0.2, 0.3], &[1, 0], &[glfw::JoystickHats::Up]);
        state.update_with_raw(true, None, raw.clone());
        assert!(state.is_connected());
        assert_eq!(state.raw(), &raw);
        assert_eq!(state.left().degrees(), None);
        let mapped = gamepad([1.0, 0.0, 0.0, 0.0], true, false);
        state.update_with_raw(true, Some(&mapped), raw.clone());
        assert!(state.is_one_down());
        state.update_with_raw(true, None, raw.clone());
        assert!(!state.is_one_down());
        assert_eq!(state.raw(), &raw);
        let smaller = RawJoystickState::from_glfw(vec![0.5], &[], &[]);
        state.update_with_raw(true, None, smaller.clone());
        assert_eq!(state.raw(), &smaller);
        state.update_with_raw(false, Some(&mapped), raw.clone());
        assert_eq!(state.raw(), &RawJoystickState::default());
        state.update_with_raw(true, None, raw);
        state.reset();
        assert_eq!(state.raw(), &RawJoystickState::default());
    }

    #[test]
    fn public_presence_tracks_connection_independently_of_mapping() {
        let mut state = JoystickState::new();
        assert!(!state.is_connected());
        let sample = gamepad([1.0, 0.0, 0.0, 0.0], true, false);

        state.update(true, None);
        assert!(state.is_connected());
        assert!(!state.is_one_used());
        state.update(true, Some(&sample));
        assert!(state.is_connected());
        assert!(state.is_one_down());
        state.update(true, None);
        assert!(state.is_connected());
        assert!(!state.is_one_down());

        state.update(false, Some(&sample));
        assert!(!state.is_connected());
        assert!(!state.is_one_down());
        state.update(true, None);
        assert!(state.is_connected());
        state.reset();
        assert!(!state.is_connected());
    }

    #[test]
    fn settings_reinterpret_history_and_survive_reset() -> VMNLResult<()> {
        let mut state = JoystickState::new();
        let left = Joystick::JoystickLeft { degrees: None };
        let sample = gamepad([0.1, 0.0, 0.1, 0.0], true, false);
        state.update(true, Some(&sample));
        state.update(true, Some(&sample));
        let settings = StickSettings {
            dead_zone: 0.0,
            zero_degrees: 90.0,
            clockwise: true,
        };
        state.set_settings(left, settings)?;
        assert_eq!(
            state.left().axes().map(f32::to_bits),
            [0.1_f32, 0.0].map(f32::to_bits)
        );
        assert_eq!(state.left().degrees(), Some(90.0));
        assert_eq!(state.right().degrees(), None);
        assert!(state.is_down(left));
        assert!(!state.is_pressed(left));
        assert!(!state.is_released(left));
        assert!(state.left().is_clicked());
        let mut events = Vec::new();
        state.append_events(JoystickId::Slot1, &mut events);
        assert!(events.is_empty());
        state.update(true, Some(&sample));
        assert_eq!(state.left().degrees(), Some(90.0));
        state.reset();
        assert_eq!(state.settings(left), settings);
        assert_eq!(
            state.left().axes().map(f32::to_bits),
            [0.0_f32, 0.0].map(f32::to_bits)
        );
        assert!(!state.is_one_used());
        Ok(())
    }

    #[test]
    fn non_finite_samples_are_retained_without_repeated_movement() {
        let mut state = JoystickState::new();
        let sample = gamepad([f32::NAN, f32::INFINITY, 0.0, 0.0], true, false);
        state.update(true, Some(&sample));
        assert!(state.left().axes()[0].is_nan());
        assert_eq!(state.left().axes()[1].to_bits(), f32::INFINITY.to_bits());
        assert_eq!(state.left().degrees(), None);
        assert!(state.left().is_clicked());
        state.update(true, Some(&sample));
        let mut events = Vec::new();
        state.append_events(JoystickId::Slot1, &mut events);
        assert!(events.is_empty());
    }

    /// Builds a mapped snapshot without initializing GLFW or accessing a device.
    fn gamepad(axes: [f32; 4], left_click: bool, right_click: bool) -> GlfwState {
        let mut raw = glfw::ffi::GLFWgamepadstate {
            buttons: [0; 15],
            axes: [0.0; 6],
        };
        raw.axes[..4].copy_from_slice(&axes);
        raw.buttons[GlfwButton::ButtonLeftThumb as usize] = u8::from(left_click);
        raw.buttons[GlfwButton::ButtonRightThumb as usize] = u8::from(right_click);
        raw.into()
    }

    #[test]
    fn glfw_axes_convert_to_counterclockwise_degrees() {
        for (x, y, expected) in [
            (1.0, 0.0, 0.0),
            (0.0, -1.0, 90.0),
            (-1.0, 0.0, 180.0),
            (0.0, 1.0, 270.0),
            (1.0, -1.0, 45.0),
        ] {
            let state = StickState::from_axes(x, y, false);
            assert!(state
                .degrees()
                .is_some_and(|angle| (angle - expected).abs() < 0.001));
        }
        let state = StickState::from_axes(1.0, f32::EPSILON, false);
        assert!(state
            .degrees()
            .is_some_and(|angle| (0.0..360.0).contains(&angle)));
    }

    #[test]
    fn dead_zone_and_invalid_axes_preserve_independent_clicks() {
        for (x, y) in [
            (0.0, 0.0),
            (0.1, 0.1),
            (STICK_DEAD_ZONE, 0.0),
            (f32::NAN, 0.0),
            (0.0, f32::INFINITY),
        ] {
            let state = StickState::from_axes(x, y, true);
            assert_eq!(state.degrees(), None);
            assert!(state.is_clicked());
        }
        assert!(StickState::from_axes(0.16, 0.0, false).degrees().is_some());
    }

    #[test]
    fn mapped_sticks_and_thumb_buttons_are_independent() {
        let mut state = JoystickState::new();
        state.update(true, Some(&gamepad([1.0, 0.0, 0.0, -1.0], true, false)));
        assert_eq!(state.left().degrees(), Some(0.0));
        assert_eq!(state.right().degrees(), Some(90.0));
        assert!(state.is_pressed(Joystick::JoystickLeftButton));
        assert!(!state.is_down(Joystick::JoystickRightButton));
        state.update(true, Some(&gamepad([0.0, 1.0, -1.0, 0.0], false, true)));
        assert_eq!(state.left().degrees(), Some(270.0));
        assert_eq!(state.right().degrees(), Some(180.0));
        assert!(state.is_released(Joystick::JoystickLeftButton));
        assert!(state.is_pressed(Joystick::JoystickRightButton));
        assert!(!state.is_pressed(Joystick::JoystickLeft { degrees: None }));
    }

    #[test]
    fn updates_detect_press_hold_and_disconnect_release() {
        let mut state = JoystickState::default();
        assert!(!state.is_one_used());
        assert!(!state.is_any_down(&[]));
        assert!(!state.is_any_pressed(&[]));
        assert!(!state.is_any_released(&[]));
        assert!(!state.is_any_used(&[]));
        let sample = gamepad([1.0, 0.0, 0.0, 0.0], true, false);
        state.update(true, Some(&sample));
        assert!(state.is_one_down());
        assert!(state.is_one_pressed());
        assert!(!state.is_one_released());
        state.update(true, Some(&sample));
        assert!(state.is_one_down());
        assert!(!state.is_one_pressed());
        state.update(true, None);
        assert!(!state.is_one_down());
        assert!(state.is_released(Joystick::JoystickLeft { degrees: None }));
        assert!(state.is_released(Joystick::JoystickLeftButton));
        assert!(!state.is_released(Joystick::JoystickRightButton));
        assert!(state.is_one_used());
        state.update(true, None);
        assert!(!state.is_one_used());
    }

    #[test]
    fn reset_clears_current_and_previous_states() {
        let mut state = JoystickState::new();
        state.update(true, Some(&gamepad([1.0, 0.0, 1.0, 0.0], true, true)));
        state.reset();
        assert!(!state.is_one_used());
        assert_eq!(state.left().degrees(), None);
        assert_eq!(state.right().degrees(), None);
        state.update(true, None);
        assert!(!state.is_one_released());
    }
    #[test]
    fn events_preserve_window_events_and_report_independent_stick_changes() {
        let mut state = JoystickState::new();
        let mut events = vec![Event::Closed];
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(events, vec![Event::Closed]);

        let sample = gamepad([1.0, 0.0, 0.0, -1.0], true, true);
        state.update(true, Some(&sample));
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(
            events,
            vec![
                Event::Closed,
                Event::JoystickConnected {
                    id: JoystickId::Slot1
                },
                Event::JoystickButtonPressed {
                    id: JoystickId::Slot1,
                    joystick: Joystick::JoystickLeftButton
                },
                Event::JoystickMoved {
                    id: JoystickId::Slot1,
                    axes: [1.0, 0.0],
                    joystick: Joystick::JoystickLeft { degrees: Some(0.0) }
                },
                Event::JoystickButtonPressed {
                    id: JoystickId::Slot1,
                    joystick: Joystick::JoystickRightButton
                },
                Event::JoystickMoved {
                    id: JoystickId::Slot1,
                    axes: [0.0, -1.0],
                    joystick: Joystick::JoystickRight {
                        degrees: Some(90.0)
                    }
                },
            ]
        );

        events.clear();
        state.update(true, Some(&sample));
        state.append_events(JoystickId::Slot1, &mut events);
        assert!(events.is_empty());

        state.update(true, Some(&gamepad([0.0, -1.0, 0.0, -1.0], false, true)));
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(
            events,
            vec![
                Event::JoystickButtonReleased {
                    id: JoystickId::Slot1,
                    joystick: Joystick::JoystickLeftButton
                },
                Event::JoystickMoved {
                    id: JoystickId::Slot1,
                    axes: [0.0, -1.0],
                    joystick: Joystick::JoystickLeft {
                        degrees: Some(90.0)
                    }
                },
            ]
        );
    }

    #[test]
    fn movement_events_preserve_magnitude_and_dead_zone_motion() {
        let mut state = JoystickState::new();
        state.update(true, Some(&gamepad([1.0, 0.0, 0.0, 0.0], false, false)));
        state.update(true, Some(&gamepad([0.5, 0.0, 0.0, 0.0], false, false)));
        let mut events = Vec::new();
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(
            events,
            vec![Event::JoystickMoved {
                id: JoystickId::Slot1,
                joystick: Joystick::JoystickLeft { degrees: Some(0.0) },
                axes: [0.5, 0.0],
            }]
        );
        events.clear();
        state.update(true, Some(&gamepad([0.1, 0.0, 0.0, 0.0], false, false)));
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(
            events,
            vec![Event::JoystickMoved {
                id: JoystickId::Slot1,
                axes: [0.1, 0.0],
                joystick: Joystick::JoystickLeft { degrees: None },
            }]
        );
        events.clear();
        state.update(true, Some(&gamepad([0.0, 0.1, 0.0, 0.0], false, false)));
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(
            events,
            vec![Event::JoystickMoved {
                id: JoystickId::Slot1,
                joystick: Joystick::JoystickLeft { degrees: None },
                axes: [0.0, 0.1],
            }]
        );
    }

    #[test]
    fn unavailable_gamepad_emits_release_and_center_events_once() {
        let mut state = JoystickState::new();
        state.update(true, Some(&gamepad([1.0, 0.0, 0.0, 1.0], true, true)));
        state.update(true, None);
        let mut events = Vec::new();
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(
            events,
            vec![
                Event::JoystickButtonReleased {
                    id: JoystickId::Slot1,
                    joystick: Joystick::JoystickLeftButton
                },
                Event::JoystickMoved {
                    id: JoystickId::Slot1,
                    axes: [0.0, 0.0],
                    joystick: Joystick::JoystickLeft { degrees: None }
                },
                Event::JoystickButtonReleased {
                    id: JoystickId::Slot1,
                    joystick: Joystick::JoystickRightButton
                },
                Event::JoystickMoved {
                    id: JoystickId::Slot1,
                    axes: [0.0, 0.0],
                    joystick: Joystick::JoystickRight { degrees: None }
                },
            ]
        );
        events.clear();
        state.update(true, None);
        state.append_events(JoystickId::Slot1, &mut events);
        assert!(events.is_empty());
    }
    #[test]
    fn presence_transitions_do_not_require_mapping_or_repeat() {
        let mut state = JoystickState::new();
        for (present, expected) in [
            (false, vec![]),
            (
                true,
                vec![Event::JoystickConnected {
                    id: JoystickId::Slot1,
                }],
            ),
            (true, vec![]),
            (
                false,
                vec![Event::JoystickDisconnected {
                    id: JoystickId::Slot1,
                }],
            ),
            (false, vec![]),
            (
                true,
                vec![Event::JoystickConnected {
                    id: JoystickId::Slot1,
                }],
            ),
        ] {
            state.update(present, None);
            let mut events = Vec::new();
            state.append_events(JoystickId::Slot1, &mut events);
            assert_eq!(events, expected);
        }
        state.reset();
        state.update(true, None);
        let mut events = Vec::new();
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(
            events,
            vec![Event::JoystickConnected {
                id: JoystickId::Slot1
            }]
        );
    }

    #[test]
    fn mapping_changes_do_not_emit_connection_events() {
        let mut state = JoystickState::new();
        state.update(true, None);
        let sample = gamepad([0.0; 4], false, false);
        state.update(true, Some(&sample));
        let mut events = Vec::new();
        state.append_events(JoystickId::Slot1, &mut events);
        assert!(events.is_empty());
        state.update(true, None);
        state.append_events(JoystickId::Slot1, &mut events);
        assert!(events.is_empty());
    }

    #[test]
    fn disconnect_precedes_release_and_center_even_with_stale_sample() {
        let mut state = JoystickState::new();
        let sample = gamepad([1.0, 0.0, 0.0, 0.0], true, false);
        state.update(true, Some(&sample));
        state.update(false, Some(&sample));
        let mut events = Vec::new();
        state.append_events(JoystickId::Slot1, &mut events);
        assert_eq!(
            events,
            vec![
                Event::JoystickDisconnected {
                    id: JoystickId::Slot1
                },
                Event::JoystickButtonReleased {
                    id: JoystickId::Slot1,
                    joystick: Joystick::JoystickLeftButton
                },
                Event::JoystickMoved {
                    id: JoystickId::Slot1,
                    axes: [0.0, 0.0],
                    joystick: Joystick::JoystickLeft { degrees: None }
                },
            ]
        );
        state.update(false, None);
        events.clear();
        state.append_events(JoystickId::Slot1, &mut events);
        assert!(events.is_empty());
    }
}
