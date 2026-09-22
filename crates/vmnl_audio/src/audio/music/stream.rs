////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use crate::audio::bus::BusKind;
use crate::audio::decoder::DecodedAudio;
use crate::audio::error::{validate_gain, AudioResult};
use crate::audio::PlaybackState;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct MusicStream {
    id: u64,
    path: PathBuf,
    decoded_audio: Arc<DecodedAudio>,
    cursor_frames: AtomicUsize,
    volume_bits: AtomicU32,
    looping: AtomicBool,
    state: AtomicU8,
    finished: AtomicBool,
    bus: BusKind,
}

impl MusicStream {
    #[must_use]
    pub(crate) fn new(id: u64, path: PathBuf, decoded_audio: Arc<DecodedAudio>, bus: BusKind, volume: f32, looping: bool, state: PlaybackState) -> AudioResult<Self> {
        Self {
            id,
            path,
            decoded_audio,
            cursor_frames: AtomicUsize::new(0),
            volume_bits: AtomicU32::new(validate_gain(volume)?.to_bits()),
            looping: AtomicBool::new(looping),
            state: AtomicU8::new(state as u8),
            finished: AtomicBool::new(false),
            bus,
        }
    }

    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn bus(&self) -> BusKind {
        self.bus
    }

    pub fn set_looping(&self, looping: bool) {
        self.looping.store(looping, Ordering::Relaxed);
    }

    #[must_use]
    pub fn looping(&self) -> bool {
        self.looping.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, volume: f32) -> AudioResult<()> {
        let volume = validate_gain(volume)?;

        self.volume_bits.store(volume.to_bits(), Ordering::Relaxed);

        Ok(())
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume_bits.load(Ordering::Relaxed))
    }

    pub(crate) fn start(&self) {
        self.set_state(PlaybackState::Playing);
    }

    pub fn set_state(&self, state: PlaybackState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }

    pub fn state(&self) -> PlaybackState {
        match self.state.load(Ordering::Relaxed) {
            0 => PlaybackState::Playing,
            1 => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.state() == PlaybackState::Playing
    }

    pub fn is_paused(&self) -> bool {
        self.state() == PlaybackState::Paused
    }

    pub fn is_stopped(&self) -> bool {
        self.state() == PlaybackState::Stopped
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.cursor_frames.store(0, Ordering::Relaxed);
        self.finished.store(true, Ordering::Relaxed);
        self.set_state(PlaybackState::Stopped);
    }

    pub fn pause(&self) {
        self.set_state(PlaybackState::Paused);
    }

    pub fn resume(&self) {
        if !self.is_stopped() {
            self.set_state(PlaybackState::Playing);
        }
    }

    #[must_use]
    pub fn cursor_frames(&self) -> usize {
        self.cursor_frames.load(Ordering::Relaxed)
    }

    pub fn set_cursor_frames(&self, cursor: usize) {
        self.cursor_frames.store(cursor, Ordering::Relaxed);
    }

    pub fn pump(&self) {}

    pub fn mix_into(&self, output: &mut [f32], gain: f32) {
        if self.state() != PlaybackState::Playing || gain <= 0.0 {
            return;
        }

        let decoded = self.decoded_audio.as_ref();
        let channels = decoded.channels().max(1) as usize;
        let input = decoded.samples();
        let volume = self.volume();
        let total_frames = decoded.frame_count();
        let frame_count = output.len() / 2;

        if input.is_empty() || total_frames == 0 || frame_count == 0 {
            self.stop();
            return;
        }

        // Reserve a distinct cursor range for this mixer invocation. This makes
        // concurrent render_into() calls safe: two mixers cannot consume the
        // same frames and overwrite each other's cursor progress.
        let start = self.cursor_frames.fetch_add(frame_count, Ordering::AcqRel);

        if self.state() != PlaybackState::Playing {
            return;
        }

        let looping = self.looping();
        let available = total_frames.saturating_sub(start);
        let frames_to_mix = if looping {
            frame_count
        } else {
            available.min(frame_count)
        };

        for frame in 0..frames_to_mix {
            let cursor = if looping {
                (start + frame) % total_frames
            } else {
                start + frame
            };

            let base = cursor * channels;
            if base + channels > input.len() {
                self.stop();
                return;
            }

            let left = input[base];
            let right = if channels >= 2 { input[base + 1] } else { left };
            let out = frame * 2;
            output[out] += left * volume * gain;
            output[out + 1] += right * volume * gain;
        }

        if !looping && frames_to_mix < frame_count {
            self.stop();
        }
    }
}
