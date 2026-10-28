// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::sound::SoundVoice;
use crate::audio::{AudioResult, PlaybackState};

use std::sync::Arc;

/// Shared controls for a single sound playback; clones affect the same instance.
///
/// Dropping a handle does not stop playback while the runtime retains it.
/// Stopped is terminal: pause/resume cannot restart this instance, including
/// when a control command races with stop or natural completion. Mixed output
/// already handed to the backend is not withdrawn by a control change.
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
    /// Has no effect on an instance that is already paused or stopped.
    pub fn pause(&self) {
        self.voice.pause();
    }

    /// Resumes a paused instance; playing and stopped instances are unchanged.
    pub fn resume(&self) {
        self.voice.resume();
    }

    /// Sets this instance's finite linear gain in `0.0..=1.0`.
    ///
    /// Zero silences output while playback continues, unless paused individually
    /// or through a routing bus. Changing volume preserves the playback state.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AudioError::InvalidState`] for invalid gains without
    /// replacing the stored value.
    pub fn set_volume(&self, volume: f32) -> AudioResult<()> {
        self.voice.set_volume(volume)
    }

    /// Sets the loop flag for subsequent mixing; stopped instances stay stopped.
    ///
    /// Disabling looping finishes the current repetition and preserves the
    /// cumulative cursor. At a completed repetition boundary no new pass starts.
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
    /// Counts complete PCM frames actually mixed. A buffer without a complete
    /// frame leaves it unchanged. If playback has never looped, the cursor stops
    /// at the decoded length; previous loop repetitions remain counted after
    /// looping is disabled. This is not the hardware presentation position.
    #[must_use]
    pub fn cursor_frames(&self) -> usize {
        self.voice.cursor_frames()
    }
}
