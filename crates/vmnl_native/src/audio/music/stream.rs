////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::bus::BusKind;
use crate::audio::decoder::DecodedAudio;
use crate::audio::PlaybackState;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct MusicStream {
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
    pub fn new(id: u64, path: PathBuf, decoded_audio: Arc<DecodedAudio>, bus: BusKind) -> Self {
        Self {
            id,
            path,
            decoded_audio,
            cursor_frames: AtomicUsize::new(0),
            volume_bits: AtomicU32::new(1.0f32.to_bits()),
            looping: AtomicBool::new(false),
            state: AtomicU8::new(PlaybackState::Playing as u8),
            finished: AtomicBool::new(false),
            bus,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn bus(&self) -> BusKind {
        self.bus
    }

    pub fn set_looping(&self, looping: bool) {
        self.looping.store(looping, Ordering::Relaxed);
    }

    pub fn looping(&self) -> bool {
        self.looping.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, volume: f32) {
        self.volume_bits
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.volume_bits.load(Ordering::Relaxed))
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
        let channels = decoded.channels.max(1) as usize;
        let input = &decoded.samples;
        let volume = self.volume();

        if input.is_empty() {
            self.stop();
            return;
        }

        let frame_count = output.len() / 2;
        let mut cursor = self.cursor_frames();
        let total_frames = decoded.frame_count();

        for frame in 0..frame_count {
            if cursor >= total_frames {
                if self.looping() {
                    cursor = 0;
                } else {
                    self.stop();
                    break;
                }
            }

            let base = cursor * channels;
            if base + channels > input.len() {
                self.stop();
                break;
            }

            let left = input[base];
            let right = if channels >= 2 { input[base + 1] } else { left };

            let out = frame * 2;
            output[out] += left * volume * gain;
            output[out + 1] += right * volume * gain;

            cursor += 1;
        }

        self.cursor_frames.store(cursor, Ordering::Relaxed);
    }
}
