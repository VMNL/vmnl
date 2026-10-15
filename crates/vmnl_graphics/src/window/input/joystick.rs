// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Owned joystick and mapped gamepad samples.

use super::transitions::TransitionState;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};

/// One of GLFW's sixteen reusable joystick slots.
///
/// A slot is not a persistent physical-device identity. Its attached device may change after a
/// disconnect; use a sample's GUID to identify a device model, noting that identical units may
/// share a GUID.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum JoystickId {
    /// GLFW joystick slot 1 (raw slot index 0).
    Joystick1 = 0,
    /// GLFW joystick slot 2 (raw slot index 1).
    Joystick2 = 1,
    /// GLFW joystick slot 3 (raw slot index 2).
    Joystick3 = 2,
    /// GLFW joystick slot 4 (raw slot index 3).
    Joystick4 = 3,
    /// GLFW joystick slot 5 (raw slot index 4).
    Joystick5 = 4,
    /// GLFW joystick slot 6 (raw slot index 5).
    Joystick6 = 5,
    /// GLFW joystick slot 7 (raw slot index 6).
    Joystick7 = 6,
    /// GLFW joystick slot 8 (raw slot index 7).
    Joystick8 = 7,
    /// GLFW joystick slot 9 (raw slot index 8).
    Joystick9 = 8,
    /// GLFW joystick slot 10 (raw slot index 9).
    Joystick10 = 9,
    /// GLFW joystick slot 11 (raw slot index 10).
    Joystick11 = 10,
    /// GLFW joystick slot 12 (raw slot index 11).
    Joystick12 = 11,
    /// GLFW joystick slot 13 (raw slot index 12).
    Joystick13 = 12,
    /// GLFW joystick slot 14 (raw slot index 13).
    Joystick14 = 13,
    /// GLFW joystick slot 15 (raw slot index 14).
    Joystick15 = 14,
    /// GLFW joystick slot 16 (raw slot index 15).
    Joystick16 = 15,
}

impl JoystickId {
    /// Number of joystick slots GLFW 3.4 defines.
    pub const COUNT: usize = 16;

    /// Converts a zero-based GLFW slot index to its valid identifier.
    #[inline]
    #[must_use]
    pub const fn from_index(index: usize) -> Option<Self> {
        Some(match index {
            0 => Self::Joystick1,
            1 => Self::Joystick2,
            2 => Self::Joystick3,
            3 => Self::Joystick4,
            4 => Self::Joystick5,
            5 => Self::Joystick6,
            6 => Self::Joystick7,
            7 => Self::Joystick8,
            8 => Self::Joystick9,
            9 => Self::Joystick10,
            10 => Self::Joystick11,
            11 => Self::Joystick12,
            12 => Self::Joystick13,
            13 => Self::Joystick14,
            14 => Self::Joystick15,
            15 => Self::Joystick16,
            _ => return None,
        })
    }

    /// Returns this identifier's zero-based GLFW slot index.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub(crate) const fn as_raw(self) -> i32 {
        self as i32
    }
}

/// Preserved raw state of one joystick button.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JoystickButtonState {
    /// GLFW reported `GLFW_RELEASE`.
    Released,
    /// GLFW reported `GLFW_PRESS`.
    Pressed,
    /// GLFW reported another byte value, retained for forward compatibility.
    Other(u8),
}

impl JoystickButtonState {
    /// Returns the GLFW byte value represented by this state.
    #[inline]
    #[must_use]
    pub const fn as_raw(self) -> u8 {
        match self {
            Self::Released => 0,
            Self::Pressed => 1,
            Self::Other(value) => value,
        }
    }

    /// Returns whether GLFW reported this button pressed.
    #[inline]
    #[must_use]
    pub const fn is_pressed(self) -> bool {
        matches!(self, Self::Pressed)
    }

    pub(crate) const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Released,
            1 => Self::Pressed,
            value => Self::Other(value),
        }
    }
}

/// Raw bit field for one joystick hat, including GLFW's diagonal combinations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct JoystickHatState(u8);

impl JoystickHatState {
    /// Hat is centered.
    pub const CENTERED: Self = Self(0x00);
    /// Hat points up.
    pub const UP: Self = Self(0x01);
    /// Hat points right.
    pub const RIGHT: Self = Self(0x02);
    /// Hat points down.
    pub const DOWN: Self = Self(0x04);
    /// Hat points left.
    pub const LEFT: Self = Self(0x08);
    /// Hat points up and right.
    pub const RIGHT_UP: Self = Self(0x03);
    /// Hat points down and right.
    pub const RIGHT_DOWN: Self = Self(0x06);
    /// Hat points up and left.
    pub const LEFT_UP: Self = Self(0x09);
    /// Hat points down and left.
    pub const LEFT_DOWN: Self = Self(0x0c);

    /// Creates a hat state from its raw GLFW bit field.
    #[inline]
    #[must_use]
    pub const fn from_raw(value: u8) -> Self {
        Self(value)
    }

    /// Returns the raw GLFW bit field without normalization.
    #[inline]
    #[must_use]
    pub const fn as_raw(self) -> u8 {
        self.0
    }

    /// Returns whether the hat points upward, including diagonals.
    #[inline]
    #[must_use]
    pub const fn is_up(self) -> bool {
        self.0 & Self::UP.0 != 0
    }

    /// Returns whether the hat points right, including diagonals.
    #[inline]
    #[must_use]
    pub const fn is_right(self) -> bool {
        self.0 & Self::RIGHT.0 != 0
    }

    /// Returns whether the hat points downward, including diagonals.
    #[inline]
    #[must_use]
    pub const fn is_down(self) -> bool {
        self.0 & Self::DOWN.0 != 0
    }

    /// Returns whether the hat points left, including diagonals.
    #[inline]
    #[must_use]
    pub const fn is_left(self) -> bool {
        self.0 & Self::LEFT.0 != 0
    }
}

/// One of GLFW's fifteen standard mapped gamepad buttons.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum GamepadButton {
    /// South face button (A in GLFW's Xbox-like layout).
    A = 0,
    /// East face button (B).
    B = 1,
    /// West face button (X).
    X = 2,
    /// North face button (Y).
    Y = 3,
    /// Left shoulder button.
    LeftBumper = 4,
    /// Right shoulder button.
    RightBumper = 5,
    /// Back/select button.
    Back = 6,
    /// Start button.
    Start = 7,
    /// Guide/system button, when the platform exposes it.
    Guide = 8,
    /// Left stick click.
    LeftThumb = 9,
    /// Right stick click.
    RightThumb = 10,
    /// D-pad up.
    DpadUp = 11,
    /// D-pad right.
    DpadRight = 12,
    /// D-pad down.
    DpadDown = 13,
    /// D-pad left.
    DpadLeft = 14,
}

impl GamepadButton {
    /// Number of standard mapped gamepad buttons in GLFW 3.4.
    pub const COUNT: usize = 15;

    const fn index(self) -> usize {
        self as usize
    }
}

/// One of GLFW's six standard mapped gamepad axes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum GamepadAxis {
    /// Left stick horizontal axis.
    LeftX = 0,
    /// Left stick vertical axis.
    LeftY = 1,
    /// Right stick horizontal axis.
    RightX = 2,
    /// Right stick vertical axis.
    RightY = 3,
    /// Left trigger axis.
    LeftTrigger = 4,
    /// Right trigger axis.
    RightTrigger = 5,
}

impl GamepadAxis {
    /// Number of standard mapped gamepad axes in GLFW 3.4.
    pub const COUNT: usize = 6;

    const fn index(self) -> usize {
        self as usize
    }
}

/// Owned standard gamepad state produced by GLFW's current mapping database.
#[derive(Clone, Debug, PartialEq)]
pub struct GamepadState {
    name: Option<String>,
    buttons: [JoystickButtonState; GamepadButton::COUNT],
    axes: [f32; GamepadAxis::COUNT],
}

impl GamepadState {
    /// Returns the mapped gamepad name, if GLFW provides one.
    #[inline]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the state for a standard gamepad button.
    #[inline]
    #[must_use]
    pub const fn button(&self, button: GamepadButton) -> JoystickButtonState {
        self.buttons[button.index()]
    }

    /// Returns all fifteen standard button states in GLFW order.
    #[inline]
    #[must_use]
    pub const fn buttons(&self) -> &[JoystickButtonState; GamepadButton::COUNT] {
        &self.buttons
    }

    /// Returns the value for a standard gamepad axis, unchanged from GLFW.
    #[inline]
    #[must_use]
    pub const fn axis(&self, axis: GamepadAxis) -> f32 {
        self.axes[axis.index()]
    }

    /// Returns all six standard axis values in GLFW order.
    #[inline]
    #[must_use]
    pub const fn axes(&self) -> &[f32; GamepadAxis::COUNT] {
        &self.axes
    }

    pub(crate) fn from_native(
        name: Option<String>,
        buttons: [u8; GamepadButton::COUNT],
        axes: [f32; GamepadAxis::COUNT],
    ) -> Self {
        Self {
            name,
            buttons: buttons.map(JoystickButtonState::from_raw),
            axes,
        }
    }
}

/// Convention used when converting a processed stick vector to an angle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StickAngleConvention {
    /// Zero points right; positive angles rotate counter-clockwise with Y up.
    Math2D,
    /// Zero points right; positive angles rotate clockwise with screen Y down.
    Screen2D,
    /// Zero points up; positive angles turn right on a local movement plane.
    Heading,
}

/// Processing configuration for one mapped gamepad stick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StickConfig {
    radial_dead_zone: f32,
    angle_convention: StickAngleConvention,
}

impl StickConfig {
    /// Creates a configuration with a radial dead zone and angle convention.
    ///
    /// The dead zone must be finite and in `[0, 1)`. Values outside it preserve their original
    /// axis components; VMNL does not rescale the remaining range.
    ///
    /// # Errors
    /// Returns `InvalidState` when the dead zone is non-finite or outside `[0, 1)`.
    pub fn new(radial_dead_zone: f32, angle_convention: StickAngleConvention) -> VMNLResult<Self> {
        if !radial_dead_zone.is_finite() || !(0.0..1.0).contains(&radial_dead_zone) {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "stick radial dead zone must be finite and in [0, 1)".to_owned(),
            )));
        }

        Ok(Self {
            radial_dead_zone,
            angle_convention,
        })
    }

    /// Returns the configured radial dead-zone threshold.
    #[inline]
    #[must_use]
    pub const fn radial_dead_zone(self) -> f32 {
        self.radial_dead_zone
    }

    /// Returns the configured angle convention.
    #[inline]
    #[must_use]
    pub const fn angle_convention(self) -> StickAngleConvention {
        self.angle_convention
    }

    /// Returns a copy of this configuration with a validated radial dead zone.
    ///
    /// # Errors
    /// Returns `InvalidState` when the dead zone is non-finite or outside `[0, 1)`.
    pub fn with_radial_dead_zone(self, radial_dead_zone: f32) -> VMNLResult<Self> {
        Self::new(radial_dead_zone, self.angle_convention)
    }

    /// Returns a copy of this configuration with the given angle convention.
    #[inline]
    #[must_use]
    pub const fn with_angle_convention(mut self, angle_convention: StickAngleConvention) -> Self {
        self.angle_convention = angle_convention;
        self
    }

    /// Applies the radial dead zone and returns the processed vector.
    ///
    /// Inputs are the stick's X/Y axes. Values at or below the configured magnitude threshold
    /// become `(0, 0)`. Values outside it remain unchanged. This pure operation also accepts axes
    /// from a standalone [`JoystickSample`]; no particular raw-axis pair is inferred.
    #[inline]
    #[must_use]
    pub fn process(self, x: f32, y: f32) -> StickState {
        let magnitude = x.hypot(y);
        let (x, y) = if magnitude <= self.radial_dead_zone {
            (0.0, 0.0)
        } else {
            (x, y)
        };

        StickState {
            x,
            y,
            magnitude: if x == 0.0 && y == 0.0 { 0.0 } else { magnitude },
            angle_convention: self.angle_convention,
        }
    }
}

impl Default for StickConfig {
    /// Uses no dead zone and the Y-up `Math2D` angle convention.
    fn default() -> Self {
        Self {
            radial_dead_zone: 0.0,
            angle_convention: StickAngleConvention::Math2D,
        }
    }
}

/// A processed two-axis stick vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StickState {
    x: f32,
    y: f32,
    magnitude: f32,
    angle_convention: StickAngleConvention,
}

impl StickState {
    /// Returns the processed X component.
    #[inline]
    #[must_use]
    pub const fn x(self) -> f32 {
        self.x
    }

    /// Returns the processed Y component.
    #[inline]
    #[must_use]
    pub const fn y(self) -> f32 {
        self.y
    }

    /// Returns the vector magnitude after the dead zone is applied.
    #[inline]
    #[must_use]
    pub const fn magnitude(self) -> f32 {
        self.magnitude
    }

    /// Returns the vector angle in radians normalized to `[0, 2π)`, or `None` at the center or
    /// when either processed component is non-finite.
    ///
    /// `Math2D` uses `atan2(-y, x)` for Y-up cartesian coordinates. `Screen2D` uses `atan2(y, x)`
    /// for Y-down screen coordinates. `Heading` uses `atan2(x, -y)` for local planar movement;
    /// it is not world or camera yaw. The angle is calculated only when this method is called.
    #[must_use]
    pub fn angle(self) -> Option<f32> {
        if (self.x == 0.0 && self.y == 0.0) || !self.x.is_finite() || !self.y.is_finite() {
            return None;
        }

        let angle = match self.angle_convention {
            StickAngleConvention::Math2D => (-self.y).atan2(self.x),
            StickAngleConvention::Screen2D => self.y.atan2(self.x),
            StickAngleConvention::Heading => self.x.atan2(-self.y),
        };
        let tau = std::f32::consts::TAU;
        let normalized = if angle < 0.0 { angle + tau } else { angle };
        if normalized >= tau {
            Some(f32::from_bits(tau.to_bits() - 1))
        } else {
            Some(if normalized == 0.0 { 0.0 } else { normalized })
        }
    }
}

/// Tracking status for one GLFW joystick slot in a window snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoystickStatus {
    /// This window has not enabled tracking, or has not completed its first sample.
    NotTracked,
    /// Tracking is active and no device occupies this slot in the latest snapshot.
    Absent,
    /// Tracking is active and a device occupies this slot in the latest snapshot.
    Present,
}

/// Per-window joystick sample and mapped-button transitions.
///
/// Native samples advance only when the owning window processes `Window::poll_events`. Disabling
/// tracking clears the snapshot immediately. Standalone [`Input::new`](crate::Input::new) values
/// remain detached from GLFW.
#[derive(Debug)]
pub struct JoystickState {
    status: JoystickStatus,
    sample: Option<JoystickSample>,
    transitions: TransitionState<{ GamepadButton::COUNT }>,
    left_stick_config: StickConfig,
    right_stick_config: StickConfig,
}

impl Default for JoystickState {
    fn default() -> Self {
        Self {
            status: JoystickStatus::NotTracked,
            sample: None,
            transitions: TransitionState::new(),
            left_stick_config: StickConfig::default(),
            right_stick_config: StickConfig::default(),
        }
    }
}

impl JoystickState {
    pub(crate) const fn begin_batch(&mut self) {
        self.transitions.begin_batch();
    }

    pub(crate) const fn clear_transitions(&mut self) {
        self.transitions.clear_transitions();
    }

    pub(crate) fn apply_sample(&mut self, sample: Option<JoystickSample>, initial: bool) {
        if initial {
            self.transitions.reset();
            for button in ALL_GAMEPAD_BUTTONS {
                self.transitions.set_down(
                    button.index(),
                    sample
                        .as_ref()
                        .and_then(JoystickSample::gamepad)
                        .is_some_and(|gamepad| {
                            gamepad.button(*button) == JoystickButtonState::Pressed
                        }),
                );
            }
        } else {
            for button in ALL_GAMEPAD_BUTTONS {
                let was_down = self.transitions.is_down(button.index());
                let is_down = sample
                    .as_ref()
                    .and_then(JoystickSample::gamepad)
                    .is_some_and(|gamepad| gamepad.button(*button) == JoystickButtonState::Pressed);
                match (was_down, is_down) {
                    (false, true) => self.transitions.press(button.index()),
                    (true, false) => self.transitions.release(button.index()),
                    _ => {}
                }
            }
        }

        self.status = if sample.is_some() {
            JoystickStatus::Present
        } else {
            JoystickStatus::Absent
        };
        self.sample = sample;
    }

    pub(crate) fn clear_tracking(&mut self) {
        self.status = JoystickStatus::NotTracked;
        self.sample = None;
        self.transitions.reset();
    }

    pub(crate) fn set_stick_configs(
        &mut self,
        left_stick_config: StickConfig,
        right_stick_config: StickConfig,
    ) {
        self.left_stick_config = left_stick_config;
        self.right_stick_config = right_stick_config;
    }

    /// Returns whether this slot is untracked, absent, or present in the latest window snapshot.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> JoystickStatus {
        self.status
    }

    /// Returns the latest owned sample while this slot is present.
    #[inline]
    #[must_use]
    pub const fn sample(&self) -> Option<&JoystickSample> {
        self.sample.as_ref()
    }

    /// Returns the latest mapped gamepad state, if this present device has a GLFW mapping.
    #[inline]
    #[must_use]
    pub fn gamepad(&self) -> Option<&GamepadState> {
        self.sample.as_ref().and_then(JoystickSample::gamepad)
    }

    /// Returns the configured left stick when this device has a gamepad mapping.
    #[inline]
    #[must_use]
    pub fn left_stick(&self) -> Option<StickState> {
        let gamepad = self.gamepad()?;
        Some(self.left_stick_config.process(
            gamepad.axis(GamepadAxis::LeftX),
            gamepad.axis(GamepadAxis::LeftY),
        ))
    }

    /// Returns the configured right stick when this device has a gamepad mapping.
    #[inline]
    #[must_use]
    pub fn right_stick(&self) -> Option<StickState> {
        let gamepad = self.gamepad()?;
        Some(self.right_stick_config.process(
            gamepad.axis(GamepadAxis::RightX),
            gamepad.axis(GamepadAxis::RightY),
        ))
    }

    /// Returns whether a mapped button is held in the latest sample.
    #[inline]
    #[must_use]
    pub const fn is_down(&self, button: GamepadButton) -> bool {
        self.transitions.is_down(button.index())
    }

    /// Returns whether a mapped button was pressed during this window's current poll batch.
    #[inline]
    #[must_use]
    pub const fn is_pressed(&self, button: GamepadButton) -> bool {
        self.transitions.is_pressed(button.index())
    }

    /// Returns whether a mapped button was released during this window's current poll batch.
    #[inline]
    #[must_use]
    pub const fn is_released(&self, button: GamepadButton) -> bool {
        self.transitions.is_released(button.index())
    }

    /// Returns whether any listed mapped button is held.
    #[must_use]
    pub fn is_any_down(&self, buttons: &[GamepadButton]) -> bool {
        buttons.iter().any(|button| self.is_down(*button))
    }

    /// Returns whether any listed mapped button was pressed in this poll batch.
    #[must_use]
    pub fn is_any_pressed(&self, buttons: &[GamepadButton]) -> bool {
        buttons.iter().any(|button| self.is_pressed(*button))
    }

    /// Returns whether any listed mapped button was released in this poll batch.
    #[must_use]
    pub fn is_any_released(&self, buttons: &[GamepadButton]) -> bool {
        buttons.iter().any(|button| self.is_released(*button))
    }
}

/// GLFW's standard mapped gamepad button order.
pub(crate) const ALL_GAMEPAD_BUTTONS: &[GamepadButton] = &[
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

/// All reusable GLFW joystick slots in their numeric order.
pub(crate) const ALL_JOYSTICK_IDS: [JoystickId; JoystickId::COUNT] = [
    JoystickId::Joystick1,
    JoystickId::Joystick2,
    JoystickId::Joystick3,
    JoystickId::Joystick4,
    JoystickId::Joystick5,
    JoystickId::Joystick6,
    JoystickId::Joystick7,
    JoystickId::Joystick8,
    JoystickId::Joystick9,
    JoystickId::Joystick10,
    JoystickId::Joystick11,
    JoystickId::Joystick12,
    JoystickId::Joystick13,
    JoystickId::Joystick14,
    JoystickId::Joystick15,
    JoystickId::Joystick16,
];

/// Owned raw and optionally mapped data sampled from one joystick slot.
///
/// Raw axes and button arrays retain GLFW order and values. Names, GUIDs, and arrays are copied
/// from GLFW before this value is returned. Sampling does not preserve transitions; those belong
/// to per-window `JoystickState` snapshots.
#[derive(Clone, Debug, PartialEq)]
pub struct JoystickSample {
    id: JoystickId,
    name: Option<String>,
    guid: Option<String>,
    axes: Vec<f32>,
    buttons: Vec<JoystickButtonState>,
    hats: Vec<JoystickHatState>,
    gamepad: Option<GamepadState>,
}

impl JoystickSample {
    /// Returns the reusable slot from which this sample was read.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> JoystickId {
        self.id
    }

    /// Returns the human-readable raw joystick name, if GLFW provides one.
    #[inline]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the SDL-compatible device-model GUID, if GLFW provides one.
    #[inline]
    #[must_use]
    pub fn guid(&self) -> Option<&str> {
        self.guid.as_deref()
    }

    /// Returns raw axis values in GLFW order and range.
    #[inline]
    #[must_use]
    pub fn axes(&self) -> &[f32] {
        &self.axes
    }

    /// Returns raw button states in GLFW order, including any configured hat buttons.
    #[inline]
    #[must_use]
    pub fn buttons(&self) -> &[JoystickButtonState] {
        &self.buttons
    }

    /// Returns separate hat values in GLFW order, preserving diagonal bit combinations.
    #[inline]
    #[must_use]
    pub fn hats(&self) -> &[JoystickHatState] {
        &self.hats
    }

    /// Returns mapped state when GLFW has a gamepad mapping for this device.
    #[inline]
    #[must_use]
    pub const fn gamepad(&self) -> Option<&GamepadState> {
        self.gamepad.as_ref()
    }

    pub(crate) fn from_native(
        id: JoystickId,
        name: Option<String>,
        guid: Option<String>,
        axes: Vec<f32>,
        buttons: Vec<u8>,
        hats: Vec<u8>,
        gamepad: Option<GamepadState>,
    ) -> Self {
        Self {
            id,
            name,
            guid,
            axes,
            buttons: buttons
                .into_iter()
                .map(JoystickButtonState::from_raw)
                .collect(),
            hats: hats.into_iter().map(JoystickHatState::from_raw).collect(),
            gamepad,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    // Exact equality proves this copy-only conversion does not transform axis values.
    #![allow(clippy::float_cmp)]

    use super::*;

    #[test]
    fn joystick_slot_indices_cover_exactly_the_sixteen_glfw_slots() {
        for index in 0..JoystickId::COUNT {
            let id = JoystickId::from_index(index).expect("every GLFW slot should be valid");
            assert_eq!(id.index(), index);
            assert_eq!(id.as_raw(), i32::try_from(index).expect("slot fits i32"));
        }

        assert_eq!(JoystickId::from_index(JoystickId::COUNT), None);
        assert_eq!(JoystickId::from_index(usize::MAX), None);
    }

    #[test]
    fn raw_button_and_hat_values_match_glfw_constants() {
        assert_eq!(
            JoystickButtonState::Released.as_raw(),
            u8::try_from(glfw::ffi::GLFW_RELEASE).expect("GLFW release fits u8")
        );
        assert_eq!(
            JoystickButtonState::Pressed.as_raw(),
            u8::try_from(glfw::ffi::GLFW_PRESS).expect("GLFW press fits u8")
        );

        let hats = [
            (JoystickHatState::CENTERED, glfw::ffi::GLFW_HAT_CENTERED),
            (JoystickHatState::UP, glfw::ffi::GLFW_HAT_UP),
            (JoystickHatState::RIGHT, glfw::ffi::GLFW_HAT_RIGHT),
            (JoystickHatState::DOWN, glfw::ffi::GLFW_HAT_DOWN),
            (JoystickHatState::LEFT, glfw::ffi::GLFW_HAT_LEFT),
            (
                JoystickHatState::RIGHT_UP,
                glfw::ffi::GLFW_HAT_RIGHT | glfw::ffi::GLFW_HAT_UP,
            ),
            (
                JoystickHatState::RIGHT_DOWN,
                glfw::ffi::GLFW_HAT_RIGHT | glfw::ffi::GLFW_HAT_DOWN,
            ),
            (
                JoystickHatState::LEFT_UP,
                glfw::ffi::GLFW_HAT_LEFT | glfw::ffi::GLFW_HAT_UP,
            ),
            (
                JoystickHatState::LEFT_DOWN,
                glfw::ffi::GLFW_HAT_LEFT | glfw::ffi::GLFW_HAT_DOWN,
            ),
        ];
        for (hat, raw) in hats {
            assert_eq!(hat.as_raw(), u8::try_from(raw).expect("GLFW hat fits u8"));
        }
    }

    #[test]
    fn raw_button_hat_and_gamepad_values_are_preserved() {
        let sample = JoystickSample::from_native(
            JoystickId::Joystick3,
            Some("raw pad".to_owned()),
            Some("0123456789abcdef0123456789abcdef".to_owned()),
            vec![-1.0, 0.25, 1.0],
            vec![0, 1, 9],
            vec![0x03, 0x80],
            Some(GamepadState::from_native(
                Some("mapped pad".to_owned()),
                [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
                [-1.0, -0.5, 0.0, 0.5, 0.75, 1.0],
            )),
        );

        assert_eq!(sample.id(), JoystickId::Joystick3);
        assert_eq!(sample.name(), Some("raw pad"));
        assert_eq!(sample.guid(), Some("0123456789abcdef0123456789abcdef"));
        assert_eq!(sample.axes(), &[-1.0, 0.25, 1.0]);
        assert_eq!(sample.buttons()[0], JoystickButtonState::Released);
        assert_eq!(sample.buttons()[1], JoystickButtonState::Pressed);
        assert_eq!(sample.buttons()[2].as_raw(), 9);
        assert_eq!(sample.hats()[0], JoystickHatState::RIGHT_UP);
        assert_eq!(sample.hats()[1].as_raw(), 0x80);
        assert!(sample.hats()[0].is_up());
        assert!(sample.hats()[0].is_right());

        let gamepad = sample.gamepad().expect("mapped data should be present");
        assert_eq!(gamepad.name(), Some("mapped pad"));
        assert_eq!(gamepad.buttons().len(), GamepadButton::COUNT);
        assert_eq!(
            gamepad.button(GamepadButton::LeftThumb),
            JoystickButtonState::Pressed
        );
        assert_eq!(gamepad.axes(), &[-1.0, -0.5, 0.0, 0.5, 0.75, 1.0]);
        assert_eq!(gamepad.axis(GamepadAxis::RightTrigger), 1.0);
    }

    #[test]
    fn gamepad_selectors_match_glfw_standard_array_order() {
        let buttons = [
            (GamepadButton::A, glfw::ffi::GLFW_GAMEPAD_BUTTON_A),
            (GamepadButton::B, glfw::ffi::GLFW_GAMEPAD_BUTTON_B),
            (GamepadButton::X, glfw::ffi::GLFW_GAMEPAD_BUTTON_X),
            (GamepadButton::Y, glfw::ffi::GLFW_GAMEPAD_BUTTON_Y),
            (
                GamepadButton::LeftBumper,
                glfw::ffi::GLFW_GAMEPAD_BUTTON_LEFT_BUMPER,
            ),
            (
                GamepadButton::RightBumper,
                glfw::ffi::GLFW_GAMEPAD_BUTTON_RIGHT_BUMPER,
            ),
            (GamepadButton::Back, glfw::ffi::GLFW_GAMEPAD_BUTTON_BACK),
            (GamepadButton::Start, glfw::ffi::GLFW_GAMEPAD_BUTTON_START),
            (GamepadButton::Guide, glfw::ffi::GLFW_GAMEPAD_BUTTON_GUIDE),
            (
                GamepadButton::LeftThumb,
                glfw::ffi::GLFW_GAMEPAD_BUTTON_LEFT_THUMB,
            ),
            (
                GamepadButton::RightThumb,
                glfw::ffi::GLFW_GAMEPAD_BUTTON_RIGHT_THUMB,
            ),
            (
                GamepadButton::DpadUp,
                glfw::ffi::GLFW_GAMEPAD_BUTTON_DPAD_UP,
            ),
            (
                GamepadButton::DpadRight,
                glfw::ffi::GLFW_GAMEPAD_BUTTON_DPAD_RIGHT,
            ),
            (
                GamepadButton::DpadDown,
                glfw::ffi::GLFW_GAMEPAD_BUTTON_DPAD_DOWN,
            ),
            (
                GamepadButton::DpadLeft,
                glfw::ffi::GLFW_GAMEPAD_BUTTON_DPAD_LEFT,
            ),
        ];
        let axes = [
            (GamepadAxis::LeftX, glfw::ffi::GLFW_GAMEPAD_AXIS_LEFT_X),
            (GamepadAxis::LeftY, glfw::ffi::GLFW_GAMEPAD_AXIS_LEFT_Y),
            (GamepadAxis::RightX, glfw::ffi::GLFW_GAMEPAD_AXIS_RIGHT_X),
            (GamepadAxis::RightY, glfw::ffi::GLFW_GAMEPAD_AXIS_RIGHT_Y),
            (
                GamepadAxis::LeftTrigger,
                glfw::ffi::GLFW_GAMEPAD_AXIS_LEFT_TRIGGER,
            ),
            (
                GamepadAxis::RightTrigger,
                glfw::ffi::GLFW_GAMEPAD_AXIS_RIGHT_TRIGGER,
            ),
        ];

        assert_eq!(buttons.len(), GamepadButton::COUNT);
        assert_eq!(axes.len(), GamepadAxis::COUNT);
        for (button, glfw_index) in buttons {
            assert_eq!(
                button.index(),
                usize::try_from(glfw_index).expect("valid GLFW index")
            );
        }
        for (axis, glfw_index) in axes {
            assert_eq!(
                axis.index(),
                usize::try_from(glfw_index).expect("valid GLFW index")
            );
        }
    }

    fn mapped_sample(
        id: JoystickId,
        pressed: &[GamepadButton],
        axes: [f32; GamepadAxis::COUNT],
    ) -> JoystickSample {
        let mut buttons = [0; GamepadButton::COUNT];
        for button in pressed {
            buttons[button.index()] = 1;
        }
        JoystickSample::from_native(
            id,
            Some("test pad".to_owned()),
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Some(GamepadState::from_native(
                Some("test pad".to_owned()),
                buttons,
                axes,
            )),
        )
    }

    #[test]
    fn joystick_snapshot_initializes_without_transitions_and_tracks_button_changes() {
        let mut state = JoystickState::default();
        assert_eq!(state.status(), JoystickStatus::NotTracked);

        state.begin_batch();
        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick1,
                &[GamepadButton::A],
                [0.0; GamepadAxis::COUNT],
            )),
            true,
        );
        assert_eq!(state.status(), JoystickStatus::Present);
        assert!(state.is_down(GamepadButton::A));
        assert!(!state.is_pressed(GamepadButton::A));
        assert!(!state.is_released(GamepadButton::A));

        state.begin_batch();
        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick1,
                &[],
                [0.0; GamepadAxis::COUNT],
            )),
            false,
        );
        assert!(!state.is_down(GamepadButton::A));
        assert!(state.is_released(GamepadButton::A));
        assert!(state.is_released(GamepadButton::A));
    }

    #[test]
    fn transitions_accumulate_across_samples_and_mapping_loss_or_disconnect_releases_held_buttons()
    {
        let mut state = JoystickState::default();
        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick1,
                &[],
                [0.0; GamepadAxis::COUNT],
            )),
            true,
        );

        state.begin_batch();
        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick1,
                &[GamepadButton::A],
                [0.0; GamepadAxis::COUNT],
            )),
            false,
        );
        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick1,
                &[],
                [0.0; GamepadAxis::COUNT],
            )),
            false,
        );
        assert!(!state.is_down(GamepadButton::A));
        assert!(state.is_pressed(GamepadButton::A));
        assert!(state.is_released(GamepadButton::A));

        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick1,
                &[GamepadButton::A],
                [0.0; GamepadAxis::COUNT],
            )),
            false,
        );
        state.begin_batch();
        let unmapped = JoystickSample::from_native(
            JoystickId::Joystick1,
            Some("test pad".to_owned()),
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        );
        state.apply_sample(Some(unmapped), false);
        assert_eq!(state.status(), JoystickStatus::Present);
        assert!(state.gamepad().is_none());
        assert!(!state.is_down(GamepadButton::A));
        assert!(state.is_released(GamepadButton::A));

        state.begin_batch();
        state.apply_sample(None, false);
        assert_eq!(state.status(), JoystickStatus::Absent);
        assert!(!state.is_released(GamepadButton::A));
        assert!(!state.is_down(GamepadButton::A));

        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick1,
                &[GamepadButton::A],
                [0.0; GamepadAxis::COUNT],
            )),
            false,
        );
        state.begin_batch();
        state.apply_sample(None, false);
        assert!(state.is_released(GamepadButton::A));

        state.begin_batch();
        state.apply_sample(None, false);
        assert!(!state.is_released(GamepadButton::A));
    }

    #[test]
    fn tracking_clear_does_not_synthesize_release() {
        let mut state = JoystickState::default();
        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick1,
                &[GamepadButton::B],
                [0.0; GamepadAxis::COUNT],
            )),
            true,
        );

        state.clear_tracking();

        assert_eq!(state.status(), JoystickStatus::NotTracked);
        assert!(state.sample().is_none());
        assert!(!state.is_down(GamepadButton::B));
        assert!(!state.is_released(GamepadButton::B));
    }

    #[test]
    fn stick_config_validates_dead_zone_and_defaults_to_unmodified_math2d() {
        let default = StickConfig::default();
        assert_eq!(default.radial_dead_zone(), 0.0);
        assert_eq!(default.angle_convention(), StickAngleConvention::Math2D);
        assert_eq!(default.process(0.25, -0.5).x(), 0.25);
        assert_eq!(default.process(0.25, -0.5).y(), -0.5);

        for invalid in [-0.1, 1.0, f32::NAN, f32::INFINITY] {
            assert!(StickConfig::new(invalid, StickAngleConvention::Math2D).is_err());
        }
        assert!(StickConfig::new(0.999, StickAngleConvention::Heading).is_ok());
    }

    #[test]
    fn stick_dead_zone_zeros_threshold_and_preserves_values_outside_without_rescaling() {
        let config =
            StickConfig::new(0.5, StickAngleConvention::Math2D).expect("valid radial dead zone");
        let at_threshold = config.process(0.3, 0.4);
        assert_eq!((at_threshold.x(), at_threshold.y()), (0.0, 0.0));
        assert_eq!(at_threshold.magnitude(), 0.0);
        assert_eq!(at_threshold.angle(), None);

        let outside = config.process(0.6, 0.0);
        assert_eq!((outside.x(), outside.y()), (0.6, 0.0));
        assert_eq!(outside.magnitude(), 0.6);
        assert_eq!(outside.angle(), Some(0.0));
    }

    #[test]
    fn stick_angle_conventions_match_documented_cardinal_directions() {
        fn assert_angle(config: StickConfig, x: f32, y: f32, expected: f32) {
            let angle = config
                .process(x, y)
                .angle()
                .expect("non-center vector has an angle");
            assert!((angle - expected).abs() < 1e-6);
            assert!((0.0..std::f32::consts::TAU).contains(&angle));
        }

        let math = StickConfig::default();
        let math_vectors = [
            (1.0, 0.0, 0.0),
            (0.0, -1.0, std::f32::consts::FRAC_PI_2),
            (-1.0, 0.0, std::f32::consts::PI),
            (0.0, 1.0, 3.0 * std::f32::consts::FRAC_PI_2),
        ];
        for (x, y, expected) in math_vectors {
            assert_angle(math, x, y, expected);
        }

        let screen = math.with_angle_convention(StickAngleConvention::Screen2D);
        let screen_vectors = [
            (1.0, 0.0, 0.0),
            (0.0, 1.0, std::f32::consts::FRAC_PI_2),
            (-1.0, 0.0, std::f32::consts::PI),
            (0.0, -1.0, 3.0 * std::f32::consts::FRAC_PI_2),
        ];
        for (x, y, expected) in screen_vectors {
            assert_angle(screen, x, y, expected);
        }

        let heading = math.with_angle_convention(StickAngleConvention::Heading);
        let heading_vectors = [
            (0.0, -1.0, 0.0),
            (1.0, 0.0, std::f32::consts::FRAC_PI_2),
            (0.0, 1.0, std::f32::consts::PI),
            (-1.0, 0.0, 3.0 * std::f32::consts::FRAC_PI_2),
        ];
        for (x, y, expected) in heading_vectors {
            assert_angle(heading, x, y, expected);
        }

        let near_tau_vectors = [
            (math, 1.0, f32::EPSILON),
            (screen, 1.0, -f32::EPSILON),
            (heading, -f32::EPSILON, -1.0),
        ];
        for (config, x, y) in near_tau_vectors {
            let angle = config.process(x, y).angle().expect("non-center angle");
            assert!(angle < std::f32::consts::TAU);
            assert!(angle > std::f32::consts::PI);
        }

        for config in [math, screen, heading] {
            assert_eq!(config.process(0.0, 0.0).angle(), None);
        }
    }

    #[test]
    fn mapped_stick_configs_are_independent_and_raw_axes_remain_unchanged() {
        let mut state = JoystickState::default();
        let left =
            StickConfig::new(0.5, StickAngleConvention::Heading).expect("valid left-stick config");
        let right = StickConfig::new(0.0, StickAngleConvention::Screen2D)
            .expect("valid right-stick config");
        state.set_stick_configs(left, right);
        state.apply_sample(
            Some(mapped_sample(
                JoystickId::Joystick2,
                &[],
                [0.25, 0.0, 0.0, 1.0, -0.5, 0.5],
            )),
            true,
        );

        let left_stick = state.left_stick().expect("mapped left stick");
        assert_eq!((left_stick.x(), left_stick.y()), (0.0, 0.0));
        assert_eq!(left_stick.angle(), None);
        let right_stick = state.right_stick().expect("mapped right stick");
        assert_eq!((right_stick.x(), right_stick.y()), (0.0, 1.0));
        assert!(
            (right_stick.angle().expect("right stick angle") - std::f32::consts::FRAC_PI_2).abs()
                < 1e-6
        );
        assert_eq!(
            state.gamepad().expect("mapped gamepad").axes(),
            &[0.25, 0.0, 0.0, 1.0, -0.5, 0.5]
        );
    }
}
