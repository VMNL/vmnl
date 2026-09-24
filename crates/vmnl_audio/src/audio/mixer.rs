////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use crate::audio::{
    runtime::AudioRuntime,
    BusKind,
    PlaybackState,
};

pub(crate) struct AudioMixer;

impl AudioMixer {
    pub(crate) fn mix(
        runtime: &AudioRuntime,
        output: &mut [f32],
    ) {
        output.fill(0.0);

        let master_gain =
            runtime.master_bus().gain();

        if master_gain <= 0.0 {
            return;
        }

        /*
         * ArcSwap:
         * no Mutex/RwLock on the realtime path.
         */
        let sfx_gain =
            runtime.bus_gain(BusKind::Sfx);

        if sfx_gain > 0.0 {
            let voices =
                runtime.sound_voice_snapshot.load();

            for voice in voices.iter() {
                if voice.state()
                    == PlaybackState::Playing
                {
                    voice.mix_into(
                        output,
                        master_gain
                            * sfx_gain,
                    );
                }
            }
        }

        let music_gain =
            runtime.bus_gain(BusKind::Music);

        if music_gain > 0.0 {
            let streams =
                runtime.music_stream_snapshot.load();

            for stream in streams.iter() {
                if stream.state()
                    == PlaybackState::Playing
                {
                    stream.mix_into(
                        output,
                        master_gain
                            * music_gain,
                    );
                }
            }
        }

        for sample in output.iter_mut() {
            *sample =
                sample.clamp(-1.0, 1.0);
        }
    }
}