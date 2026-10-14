////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
////////////////////////////////////////////////////////////////////////////////
use crate::audio::decoder::DecodedAudio;
use crate::audio::error::{validate_gain, AudioResult};

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PlaybackState {
    Playing = 0,
    Paused = 1,
    Stopped = 2,
}

#[derive(Debug)]
pub(crate) struct SoundVoice {
    id: u64,
    decoded_audio: Arc<DecodedAudio>,
    cursor_frames: AtomicUsize,
    volume_bits: AtomicU32,
    looping: AtomicBool,
    state: AtomicU8,
}

impl SoundVoice {
    pub(crate) fn new(
        id: u64,
        decoded_audio: Arc<DecodedAudio>,
        volume: f32,
        looping: bool,
        state: PlaybackState,
    ) -> AudioResult<Self> {
        Ok(Self {
            id,
            decoded_audio,
            cursor_frames: AtomicUsize::new(0),
            volume_bits: AtomicU32::new(validate_gain(volume)?.to_bits()),
            looping: AtomicBool::new(looping),
            state: AtomicU8::new(state as u8),
        })
    }

    #[must_use]
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn set_looping(&self, looping: bool) {
        self.looping.store(looping, Ordering::Relaxed);
    }

    fn looping(&self) -> bool {
        self.looping.load(Ordering::Relaxed)
    }

    pub(crate) fn set_volume(&self, volume: f32) -> AudioResult<()> {
        let volume = validate_gain(volume)?;
        self.volume_bits.store(volume.to_bits(), Ordering::Relaxed);
        Ok(())
    }

    fn volume(&self) -> f32 {
        f32::from_bits(self.volume_bits.load(Ordering::Relaxed))
    }

    fn set_state(&self, state: PlaybackState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }

    pub(crate) fn state(&self) -> PlaybackState {
        match self.state.load(Ordering::Relaxed) {
            0 => PlaybackState::Playing,
            1 => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        }
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.state() == PlaybackState::Stopped
    }

    pub(crate) fn stop(&self) {
        self.set_state(PlaybackState::Stopped);
    }

    pub(crate) fn pause(&self) {
        self.set_state(PlaybackState::Paused);
    }

    pub(crate) fn resume(&self) {
        if !self.is_stopped() {
            self.set_state(PlaybackState::Playing);
        }
    }

    pub(crate) fn cursor_frames(&self) -> usize {
        self.cursor_frames.load(Ordering::Relaxed)
    }

    pub(crate) fn mix_into(&self, output: &mut [f32], gain: f32) {
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

        // CPAL is the only runtime mixing consumer. Keep a monotonic cursor so
        // the voice remains safe if that backend contract changes in the future.
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
