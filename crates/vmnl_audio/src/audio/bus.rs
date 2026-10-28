// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::error::{validate_gain, AudioResult};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum BusKind {
    Master,
    Music,
    Sfx,
}

#[derive(Debug)]
struct AudioBusState {
    volume_bits: AtomicU32,
    muted: AtomicBool,
    paused: AtomicBool,
}

/// Shared controls for one routing bus; clones address the same state.
///
/// Volume is a finite linear gain in `0.0..=1.0`, initially `1.0`.
/// Mute and pause initially are disabled and do not replace the stored volume.
/// The current mixer skips PCM consumption for both mute and pause, or zero gain;
/// the acceptance target distinguishes silent progression from a paused cursor.
#[derive(Clone, Debug)]
pub struct AudioBus {
    state: Arc<AudioBusState>,
}

impl AudioBus {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(AudioBusState {
                volume_bits: AtomicU32::new(1.0f32.to_bits()),
                muted: AtomicBool::new(false),
                paused: AtomicBool::new(false),
            }),
        }
    }

    /// Sets the linear gain without changing mute or pause.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AudioError::InvalidState`] for a non-finite value or one
    /// outside `0.0..=1.0`; the previous value is preserved.
    pub fn set_volume(&self, volume: f32) -> AudioResult<()> {
        let volume = validate_gain(volume)?;
        self.state
            .volume_bits
            .store(volume.to_bits(), Ordering::Relaxed);
        Ok(())
    }

    /// Returns the stored linear gain, independently of mute and pause.
    #[must_use]
    pub fn volume(&self) -> f32 {
        f32::from_bits(self.state.volume_bits.load(Ordering::Relaxed))
    }

    /// Mutes the bus without replacing its stored volume.
    pub fn mute(&self) {
        self.state.muted.store(true, Ordering::Relaxed);
    }

    /// Clears mute without changing volume or pause.
    pub fn unmute(&self) {
        self.state.muted.store(false, Ordering::Relaxed);
    }

    /// Reports whether mute is enabled.
    #[must_use]
    pub fn is_muted(&self) -> bool {
        self.state.muted.load(Ordering::Relaxed)
    }

    /// Pauses consumption of playbacks routed through this bus.
    pub fn pause(&self) {
        self.state.paused.store(true, Ordering::Relaxed);
    }

    /// Clears the bus pause; individual playback states are preserved.
    pub fn resume(&self) {
        self.state.paused.store(false, Ordering::Relaxed);
    }

    /// Reports whether the bus is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.state.paused.load(Ordering::Relaxed)
    }

    /// Returns zero when muted or paused, otherwise the stored linear gain.
    #[must_use]
    pub fn gain(&self) -> f32 {
        if self.is_muted() || self.is_paused() {
            0.0
        } else {
            self.volume()
        }
    }
}
