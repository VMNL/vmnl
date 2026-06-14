////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use crate::audio::{MusicStream, PlaybackState};

use std::sync::Arc;

#[derive(Clone)]
pub struct MusicHandle {
    stream: Arc<MusicStream>,
}

impl MusicHandle {
    pub(crate) fn new(stream: Arc<MusicStream>) -> Self {
        Self { stream }
    }

    pub fn stop(&self) {
        self.stream.stop();
    }

    pub fn pause(&self) {
        self.stream.pause();
    }

    pub fn resume(&self) {
        self.stream.resume();
    }

    pub fn set_volume(&self, volume: f32) {
        self.stream.set_volume(volume);
    }

    pub fn set_looping(&self, looping: bool) {
        self.stream.set_looping(looping);
    }

    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.stream.state() == PlaybackState::Playing
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.stream.state() == PlaybackState::Paused
    }

    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.stream.state() == PlaybackState::Stopped
    }

    #[must_use]
    pub fn cursor_frames(&self) -> usize {
        self.stream.cursor_frames()
    }
}
