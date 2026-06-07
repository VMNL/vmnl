////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::{PlaybackState, SoundVoice};

use std::sync::Arc;

#[derive(Clone)]
pub struct SoundHandle {
    voice: Arc<SoundVoice>,
}

impl SoundHandle {
    pub(crate) fn new(voice: Arc<SoundVoice>) -> Self {
        Self { voice }
    }

    pub fn stop(&self) {
        self.voice.stop();
    }

    pub fn pause(&self) {
        self.voice.pause();
    }

    pub fn resume(&self) {
        self.voice.resume();
    }

    pub fn set_volume(&self, volume: f32) {
        self.voice.set_volume(volume);
    }

    pub fn set_looping(&self, looping: bool) {
        self.voice.set_looping(looping);
    }

    pub fn is_playing(&self) -> bool {
        self.voice.state() == PlaybackState::Playing
    }

    pub fn is_paused(&self) -> bool {
        self.voice.state() == PlaybackState::Paused
    }

    pub fn is_stopped(&self) -> bool {
        self.voice.state() == PlaybackState::Stopped
    }

    pub fn cursor_frames(&self) -> usize {
        self.voice.cursor_frames()
    }
}
