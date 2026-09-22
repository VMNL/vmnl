// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Backend-independent keyboard modifier flags used by input events.

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

/// Keyboard modifier flags captured when an input event is generated.
///
/// Caps Lock and Num Lock are reported only when lock-key modifiers are enabled for the window.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    /// No modifier is active.
    pub const NONE: Self = Self(0);
    /// The Shift modifier is active.
    pub const SHIFT: Self = Self(1 << 0);
    /// The Control modifier is active.
    pub const CONTROL: Self = Self(1 << 1);
    /// The Alt modifier is active.
    pub const ALT: Self = Self(1 << 2);
    /// The Super modifier is active.
    pub const SUPER: Self = Self(1 << 3);
    /// Caps Lock is active and lock-key modifiers are enabled.
    pub const CAPS_LOCK: Self = Self(1 << 4);
    /// Num Lock is active and lock-key modifiers are enabled.
    pub const NUM_LOCK: Self = Self(1 << 5);

    /// Returns `true` when no modifier flag is set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` when every flag in `other` is set.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns the VMNL-defined modifier bits.
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }

    pub(crate) fn from_glfw(modifiers: glfw::Modifiers) -> Self {
        let mut converted = Self::NONE;

        for (source, target) in [
            (glfw::Modifiers::Shift, Self::SHIFT),
            (glfw::Modifiers::Control, Self::CONTROL),
            (glfw::Modifiers::Alt, Self::ALT),
            (glfw::Modifiers::Super, Self::SUPER),
            (glfw::Modifiers::CapsLock, Self::CAPS_LOCK),
            (glfw::Modifiers::NumLock, Self::NUM_LOCK),
        ] {
            if modifiers.contains(source) {
                converted |= target;
            }
        }

        converted
    }
}

impl BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Modifiers {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Modifiers {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use super::Modifiers;

    #[test]
    fn converts_every_glfw_modifier_without_exposing_backend_bits() {
        let source = glfw::Modifiers::Shift
            | glfw::Modifiers::Alt
            | glfw::Modifiers::CapsLock
            | glfw::Modifiers::NumLock;
        let converted = Modifiers::from_glfw(source);

        assert!(converted.contains(Modifiers::SHIFT));
        assert!(converted.contains(Modifiers::ALT));
        assert!(converted.contains(Modifiers::CAPS_LOCK));
        assert!(converted.contains(Modifiers::NUM_LOCK));
        assert!(!converted.contains(Modifiers::CONTROL));
        assert!(!converted.contains(Modifiers::SUPER));
    }

    #[test]
    fn supports_composition_and_intersection() {
        let combined = Modifiers::SHIFT | Modifiers::CONTROL;

        assert_eq!(combined & Modifiers::SHIFT, Modifiers::SHIFT);
        assert!(combined.contains(Modifiers::SHIFT | Modifiers::CONTROL));
        assert!(!combined.is_empty());
        assert_eq!(Modifiers::NONE.bits(), 0);
    }
}
