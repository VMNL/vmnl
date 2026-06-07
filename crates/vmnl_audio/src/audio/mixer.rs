///////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::{AudioRuntime, BusKind, MusicStream, PlaybackState, SoundVoice};

use std::sync::Arc;

pub struct AudioMixer;

impl AudioMixer {
    pub fn mix(runtime: &AudioRuntime, output: &mut [f32]) {
        output.fill(0.0);

        let master_gain = runtime.master_bus.gain();
        if master_gain <= 0.0 {
            return;
        }

        let voices: Vec<Arc<SoundVoice>> = runtime
            .active_sound_voices
            .read()
            .ok()
            .map(|voices| voices.iter().cloned().collect())
            .unwrap_or_default();

        let streams: Vec<Arc<MusicStream>> = runtime
            .active_music_streams
            .read()
            .ok()
            .map(|streams| streams.iter().cloned().collect())
            .unwrap_or_default();

        let sfx_gain = runtime.bus_gain(BusKind::Sfx);
        if sfx_gain > 0.0 {
            for voice in voices {
                if voice.state() == PlaybackState::Playing {
                    voice.mix_into(output, master_gain * sfx_gain);
                }
            }
        }

        let music_gain = runtime.bus_gain(BusKind::Music);
        if music_gain > 0.0 {
            for stream in streams {
                if stream.state() == PlaybackState::Playing {
                    stream.mix_into(output, master_gain * music_gain);
                }
            }
        }

        for sample in output.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}
