///////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::sound::instance::PlaybackState;
use crate::audio::sound::voice::SoundVoice;

use std::sync::{Arc, Mutex};

pub struct AudioMixer;

impl AudioMixer
{
    pub fn mix_sound_voices(voices: &[Arc<Mutex<SoundVoice>>],output: &mut [f32],)
    {
        output.fill(0.0);

        for voice in voices
        {
            let voice_guard = voice.lock().unwrap();
            let mut instance = voice_guard.instance.lock().unwrap();

            if instance.state != PlaybackState::Playing
            {
                continue;
            }

            let decoded = &voice_guard.decoded_audio;
            let channels = decoded.channels as usize;
            if channels == 0
            {
                continue;
            }

            let samples = &decoded.samples;
            let total_samples = samples.len();

            for frame in 0..(output.len() / 2)
            {
                let cursor_frame = instance.cursor;
                let src_base = cursor_frame * channels;

                if src_base + channels > total_samples
                {
                    if instance.looping
                    {
                        instance.cursor = 0;
                        continue;
                    }
                    else
                    {
                        instance.state = PlaybackState::Stopped;
                        break;
                    }
                }

                let left = samples[src_base];
                let right = if channels > 1
                {
                    samples[src_base + 1]
                }
                else
                {
                    left
                };

                let out_index = frame * 2;
                output[out_index] += left * instance.volume;
                output[out_index + 1] += right * instance.volume;

                instance.cursor += 1;
            }
        }

        for sample in output.iter_mut()
        {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}