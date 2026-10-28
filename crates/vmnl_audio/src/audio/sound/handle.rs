// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::sound::SoundVoice;
use crate::audio::{AudioResult, PlaybackState};

use std::sync::Arc;

/// Shared controls for a single sound playback; clones affect the same instance.
///
/// Dropping a handle does not stop playback while the runtime retains it.
/// Controls use shared state, but the current pause/stop transitions do not yet
/// enforce a terminal stopped state under every command sequence or race.
#[derive(Clone)]
pub struct SoundHandle {
    voice: Arc<SoundVoice>,
}

impl SoundHandle {
    pub(crate) fn new(voice: Arc<SoundVoice>) -> Self {
        Self { voice }
    }

    /// Marks this playback as stopped; runtime cleanup later removes it.
    pub fn stop(&self) {
        self.voice.stop();
    }

    /// Pauses this instance without changing its cursor.
    ///
    /// Current limitation: this also overwrites a stopped state. Do not call it
    /// on a stopped handle while terminal-state enforcement is pending.
    pub fn pause(&self) {
        self.voice.pause();
    }

    /// Resumes a non-stopped instance; an instance still marked stopped is unchanged.
    pub fn resume(&self) {
        self.voice.resume();
    }

    /// Sets this instance's finite linear gain in `0.0..=1.0`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AudioError::InvalidState`] for invalid gains without
    /// replacing the stored value.
    pub fn set_volume(&self, volume: f32) -> AudioResult<()> {
        self.voice.set_volume(volume)
    }

    /// Sets the loop flag; this does not restart an instance marked stopped.
    pub fn set_looping(&self, looping: bool) {
        self.voice.set_looping(looping);
    }

    /// Reports the current playback state; this does not prove audible output.
    ///
    /// A muted/paused bus or a streaming underrun can still produce silence.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.voice.state() == PlaybackState::Playing
    }

    /// Reports whether this instance is marked paused, independently of bus pause.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.voice.state() == PlaybackState::Paused
    }

    /// Reports whether this instance is currently marked stopped.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.voice.state() == PlaybackState::Stopped
    }

    /// Returns the cumulative voice cursor in frames at the device sample rate.
    ///
    /// This is not the hardware presentation position. Current limitation: the
    /// voice counts requested output frames and can overshoot a non-looping source.
    #[must_use]
    pub fn cursor_frames(&self) -> usize {
        self.voice.cursor_frames()
    }
}
