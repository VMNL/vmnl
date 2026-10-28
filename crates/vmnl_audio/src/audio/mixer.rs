// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::bus::BusKind;
use crate::audio::{runtime::AudioRuntime, PlaybackState};

pub(crate) struct AudioMixer;

impl AudioMixer {
    pub(crate) fn mix(runtime: &AudioRuntime, output: &mut [f32]) {
        output.fill(0.0);

        let master = runtime.bus(BusKind::Master);
        if master.is_paused() {
            return;
        }
        let master_gain = master.gain();

        /*
         * ArcSwap:
         * no Mutex/RwLock on the realtime path.
         */
        let sfx = runtime.bus(BusKind::Sfx);

        // Zero gain silences output; only pause suppresses PCM consumption.
        if !sfx.is_paused() {
            let sfx_gain = sfx.gain();
            let voices = runtime.sound_voice_snapshot.load();

            for voice in voices.iter() {
                if voice.state() == PlaybackState::Playing {
                    voice.mix_into(output, master_gain * sfx_gain);
                }
            }
        }

        let music = runtime.bus(BusKind::Music);

        if !music.is_paused() {
            let music_gain = music.gain();
            let streams = runtime.music_stream_snapshot.load();

            for stream in streams.iter() {
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
