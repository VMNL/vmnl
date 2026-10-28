// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::error::{AudioError, AudioResult};
use crate::audio::stream_decoder::StreamingDecoder;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DecodedAudio {
    channels: u32,
    sample_rate: u32,
    samples: Vec<f32>,
}

impl DecodedAudio {
    #[must_use]
    pub fn frame_count(&self) -> usize {
        if self.channels == 0 {
            0
        } else {
            self.samples.len() / self.channels as usize
        }
    }

    #[must_use]
    // Used by the external source-copy tests, not this crate's empty unit target.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub(crate) fn channels(&self) -> u32 {
        self.channels
    }

    pub(crate) fn samples(&self) -> &[f32] {
        &self.samples
    }

    // Linear interpolation intentionally narrows its fraction to f32; preserve
    // the existing floating phase/index policy rather than changing the DSP here.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub(crate) fn resample_to_stereo(&self, target_sample_rate: u32) -> AudioResult<Self> {
        if target_sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "target sample rate must be greater than zero".to_string(),
            ));
        }

        match self.channels {
            1 | 2 => {}

            channels => {
                return Err(AudioError::UnsupportedFormat(format!(
                    "sound input has {channels} channels, \
                        but the mixer accepts mono or stereo"
                )));
            }
        }

        let source_frames = self.frame_count();

        if source_frames == 0 {
            return Ok(Self {
                channels: 2,
                sample_rate: target_sample_rate,
                samples: Vec::new(),
            });
        }

        /*
         * No conversion necessary.
         */
        if self.channels == 2 && self.sample_rate == target_sample_rate {
            return Ok(self.clone());
        }

        let target_frames_u128 = (source_frames as u128 * u128::from(target_sample_rate))
            .div_ceil(u128::from(self.sample_rate));

        let target_frames = usize::try_from(target_frames_u128)
            .map_err(|_| AudioError::DecoderFailed("resampled audio is too large".to_string()))?;

        let source_channels = self.channels as usize;

        let mut samples = Vec::with_capacity(target_frames.saturating_mul(2));

        for frame in 0..target_frames {
            let source_position =
                frame as f64 * f64::from(self.sample_rate) / f64::from(target_sample_rate);

            let source_index = source_position.floor() as usize;

            let fraction = (source_position - source_index as f64) as f32;

            let i0 = source_index.min(source_frames - 1);

            let i1 = (i0 + 1).min(source_frames - 1);

            let sample =
                |index: usize, channel: usize| self.samples[index * source_channels + channel];

            let left0 = sample(i0, 0);
            let left1 = sample(i1, 0);

            let left = left0 + (left1 - left0) * fraction;

            let right = if self.channels == 1 {
                left
            } else {
                let right0 = sample(i0, 1);

                let right1 = sample(i1, 1);

                right0 + (right1 - right0) * fraction
            };

            samples.push(left);
            samples.push(right);
        }

        Ok(Self {
            channels: 2,
            sample_rate: target_sample_rate,
            samples,
        })
    }
}

/// Decodes supported media formats through the same incremental decoder used by
/// music streaming. Sound effects are accumulated in memory; music is not.
pub(crate) struct AudioDecoder;

impl AudioDecoder {
    // This bounded 32 KiB scratch belongs to file loading, never the audio callback.
    #[allow(clippy::large_stack_arrays)]
    pub(crate) fn decode_file<P: AsRef<Path>>(path: P) -> AudioResult<DecodedAudio> {
        let mut decoder = StreamingDecoder::open_native(path)?;
        let sample_rate = decoder.target_sample_rate();
        let mut samples = Vec::new();
        let mut scratch = [0.0f32; 8192];

        loop {
            let read = decoder.read_stereo(&mut scratch)?;
            if read == 0 {
                break;
            }
            samples.extend_from_slice(&scratch[..read]);
        }

        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(AudioError::DecoderFailed(
                "decoded audio contains a non-finite sample".to_owned(),
            ));
        }

        Ok(DecodedAudio {
            channels: 2,
            sample_rate,
            samples,
        })
    }
}
