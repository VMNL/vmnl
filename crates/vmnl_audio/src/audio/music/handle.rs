// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::music::MusicStream;
use crate::audio::{AudioResult, PlaybackState};

use std::sync::Arc;

/// Shared controls for a single music playback; clones affect the same instance.
///
/// Dropping a handle does not stop playback while the runtime retains it.
/// Stopped is terminal: pause/resume cannot restart this instance, including
/// when a control command races with stop or natural completion. Mixed output
/// already handed to the backend is not withdrawn by a control change.
#[derive(Clone)]
pub struct MusicHandle {
    stream: Arc<MusicStream>,
}

impl MusicHandle {
    pub(crate) fn new(stream: Arc<MusicStream>) -> Self {
        Self { stream }
    }

    /// Marks this playback as stopped; runtime cleanup later removes it.
    pub fn stop(&self) {
        self.stream.stop();
    }

    /// Pauses this instance without changing its cursor.
    ///
    /// Has no effect on an instance that is already paused or stopped.
    pub fn pause(&self) {
        self.stream.pause();
    }

    /// Resumes a paused instance; playing and stopped instances are unchanged.
    pub fn resume(&self) {
        self.stream.resume();
    }

    /// Sets this instance's finite linear gain in `0.0..=1.0`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AudioError::InvalidState`] for invalid gains without
    /// replacing the stored value.
    pub fn set_volume(&self, volume: f32) -> AudioResult<()> {
        self.stream.set_volume(volume)
    }

    /// Sets the loop flag used by subsequent decoder refills.
    ///
    /// Enabling it on a live instance permits refill after decoder EOF, even if
    /// preloaded PCM drains before the next control cycle. A stopped instance is
    /// never restarted; completion racing with a control change may win first.
    /// Disabling looping preserves already buffered PCM, including prefetched
    /// repetitions, and stops when that data and the remaining source drain.
    pub fn set_looping(&self, looping: bool) {
        self.stream.set_looping(looping);
    }

    /// Reports the current playback state; this does not prove audible output.
    ///
    /// A muted/paused bus or a streaming underrun can still produce silence.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.stream.state() == PlaybackState::Playing
    }

    /// Reports whether this instance is marked paused, independently of bus pause.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.stream.state() == PlaybackState::Paused
    }

    /// Reports whether this instance is currently marked stopped.
    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.stream.state() == PlaybackState::Stopped
    }

    /// Returns PCM frames consumed at the device sample rate, cumulatively across loops.
    ///
    /// Silence inserted for an underrun does not advance this cursor. This is not
    /// the exact hardware presentation position.
    #[must_use]
    pub fn cursor_frames(&self) -> usize {
        self.stream.cursor_frames()
    }
}
