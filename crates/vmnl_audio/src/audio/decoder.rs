////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use crate::audio::error::{AudioError, AudioResult};

use std::fs::File;
use std::io::BufReader;
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
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub(crate) fn channels(&self) -> u32 {
        self.channels
    }

    #[must_use]
    pub(crate) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }


    pub(crate) fn samples(&self) -> &[f32] {
        &self.samples
    }
}

pub struct AudioDecoder;

impl AudioDecoder {
    pub fn decode_file<P>(path: P) -> AudioResult<DecodedAudio>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match ext.as_str() {
            "wav" => Self::decode_wav(path),
            "mp3" => Self::decode_mp3(path),
            "ogg" => Self::decode_ogg(path),
            "flac" => Self::decode_flac(path),
            _ => Err(AudioError::UnsupportedFormat(path.display().to_string())),
        }
    }

    fn decode_wav(path: &Path) -> AudioResult<DecodedAudio> {
        let mut reader =
            hound::WavReader::open(path).map_err(|e| AudioError::DecoderFailed(e.to_string()))?;
        let spec = reader.spec();
        let channels = u32::from(spec.channels);
        let sample_rate = spec.sample_rate;

        if channels == 0 {
            return Err(AudioError::DecoderFailed(
                "WAV file has zero channels".to_string(),
            ));
        }

        if sample_rate == 0 {
            return Err(AudioError::DecoderFailed(
                "WAV file has zero sample rate".to_string(),
            ));
        }

        let samples = match spec.sample_format {
            hound::SampleFormat::Float => {
                let mut out = Vec::new();
                for sample in reader.samples::<f32>() {
                    let sample = sample.map_err(|e| AudioError::DecoderFailed(e.to_string()))?;

                    if !sample.is_finite() {
                        return Err(AudioError::DecoderFailed(
                            "WAV contains non-finite audio sample".to_string(),
                        ));
                    }

                    out.push(sample);
                }
                out
            }
            hound::SampleFormat::Int => {
                let bits = spec.bits_per_sample;

                if bits == 0 || bits > 32 {
                    return Err(AudioError::DecoderFailed(format!(
                        "unsupported WAV bit depth: {bits}"
                    )));
                }

                let denom = (1i64 << u32::from(bits - 1)) as f32;
                let mut out = Vec::new();

                if bits <= 16 {
                    for sample in reader.samples::<i16>() {
                        out.push(
                            f32::from(
                                sample.map_err(|e| AudioError::DecoderFailed(e.to_string()))?,
                            ) / denom,
                        );
                    }
                } else {
                    for sample in reader.samples::<i32>() {
                        out.push(
                            sample.map_err(|e| AudioError::DecoderFailed(e.to_string()))? as f32
                                / denom,
                        );
                    }
                }

                out
            }
        };

        Ok(DecodedAudio {
            channels,
            sample_rate,
            samples,
        })
    }

    fn decode_mp3(path: &Path) -> AudioResult<DecodedAudio> {
        let file = File::open(path)?;
        let mut decoder = minimp3::Decoder::new(BufReader::new(file));
        let mut samples = Vec::new();
        let mut channels = 2u32;
        let mut sample_rate = 44_100u32;

        loop {
            match decoder.next_frame() {
                Ok(frame) => {
                    channels = frame.channels.max(1) as u32;
                    sample_rate = frame.sample_rate.max(1) as u32;
                    samples.extend(
                        frame
                            .data
                            .into_iter()
                            .map(|s| f32::from(s) / f32::from(i16::MAX)),
                    );
                }
                Err(minimp3::Error::Eof) => break,
                Err(error) => return Err(AudioError::DecoderFailed(error.to_string())),
            }
        }

        Ok(DecodedAudio {
            channels,
            sample_rate,
            samples,
        })
    }

    fn decode_ogg(path: &Path) -> AudioResult<DecodedAudio> {
        let file = File::open(path)?;
        let mut reader = lewton::inside_ogg::OggStreamReader::new(BufReader::new(file))
            .map_err(|e| AudioError::DecoderFailed(e.to_string()))?;
        let channels = u32::from(reader.ident_hdr.audio_channels.max(1));
        let sample_rate = reader.ident_hdr.audio_sample_rate.max(1);
        let mut samples = Vec::new();

        while let Some(packet) = reader
            .read_dec_packet_itl()
            .map_err(|e| AudioError::DecoderFailed(e.to_string()))?
        {
            samples.extend(
                packet
                    .into_iter()
                    .map(|s| f32::from(s) / f32::from(i16::MAX)),
            );
        }

        Ok(DecodedAudio {
            channels,
            sample_rate,
            samples,
        })
    }

    fn decode_flac(path: &Path) -> AudioResult<DecodedAudio> {
        let mut reader =
            claxon::FlacReader::open(path).map_err(|e| AudioError::DecoderFailed(e.to_string()))?;
        let info = reader.streaminfo();
        let channels = info.channels;
        let sample_rate = info.sample_rate;
        let bits = info.bits_per_sample;

        if channels == 0 {
            return Err(AudioError::DecoderFailed(
                "FLAC file has zero channels".to_string(),
            ));
        }

        if sample_rate == 0 {
            return Err(AudioError::DecoderFailed(
                "FLAC file has zero sample rate".to_string(),
            ));
        }

        if bits == 0 || bits > 32 {
            return Err(AudioError::DecoderFailed(format!(
                "unsupported FLAC bit depth: {bits}"
            )));
        }

        let denom = (1i64 << (bits - 1)) as f32;
        let mut samples = Vec::new();

        for sample in reader.samples() {
            samples
                .push(sample.map_err(|e| AudioError::DecoderFailed(e.to_string()))? as f32 / denom);
        }

        Ok(DecodedAudio {
            channels,
            sample_rate,
            samples,
        })
    }

    pub(crate) fn resample_to_stereo(
    &self,
    target_sample_rate: u32,
    ) -> AudioResult<Self> {
        if target_sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "target sample rate must be greater than zero"
                    .to_string(),
            ));
        }

        match self.channels {
            1 | 2 => {}

            channels => {
                return Err(AudioError::UnsupportedFormat(
                    format!(
                        "sound input has {channels} channels, \
                        but the mixer accepts mono or stereo"
                    ),
                ));
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
        if self.channels == 2
            && self.sample_rate == target_sample_rate
        {
            return Ok(self.clone());
        }

        let target_frames_u128 =
            (source_frames as u128
                * target_sample_rate as u128
                + self.sample_rate as u128
                - 1)
                / self.sample_rate as u128;

        let target_frames =
            usize::try_from(target_frames_u128)
                .map_err(|_| {
                    AudioError::DecoderFailed(
                        "resampled audio is too large"
                            .to_string(),
                    )
                })?;

        let source_channels =
            self.channels as usize;

        let mut samples =
            Vec::with_capacity(
                target_frames.saturating_mul(2)
            );

        for frame in 0..target_frames {
            let source_position =
                frame as f64
                    * self.sample_rate as f64
                    / target_sample_rate as f64;

            let source_index =
                source_position.floor() as usize;

            let fraction =
                (source_position
                    - source_index as f64)
                    as f32;

            let i0 =
                source_index.min(
                    source_frames - 1
                );

            let i1 =
                (i0 + 1).min(
                    source_frames - 1
                );

            let sample =
                |index: usize, channel: usize| {
                    self.samples[
                        index * source_channels
                            + channel
                    ]
                };

            let left0 = sample(i0, 0);
            let left1 = sample(i1, 0);

            let left =
                left0
                    + (left1 - left0)
                        * fraction;

            let right =
                if self.channels == 1 {
                    left
                } else {
                    let right0 =
                        sample(i0, 1);

                    let right1 =
                        sample(i1, 1);

                    right0
                        + (right1 - right0)
                            * fraction
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
