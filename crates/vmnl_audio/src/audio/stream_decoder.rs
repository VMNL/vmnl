// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

//! Incremental decoders for the formats already supported by `vmnl_audio`.
//!
//! Decoding and resampling run on the control thread, never in the CPAL callback.

use crate::audio::error::{AudioError, AudioResult};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

const SOURCE_CHUNK_SAMPLES: usize = 8_192;

type WavReader = hound::WavReader<BufReader<File>>;
type Mp3Decoder = minimp3::Decoder<BufReader<File>>;
type OggReader = lewton::inside_ogg::OggStreamReader<BufReader<File>>;
type FlacReader = claxon::FlacReader<File>;

// Keep codec state inline: boxing it solely for size would change control-side
// allocation and destruction costs. No backend state is created on the callback.
#[allow(clippy::large_enum_variant)]
enum DecoderBackend {
    WavFloat(WavReader),
    WavInt {
        reader: WavReader,
        bits: u16,
    },
    Mp3(Mp3Decoder),
    Ogg(OggReader),
    Flac {
        reader: FlacReader,
        bits: u32,
        frame_buffer: Vec<i32>,
    },
}

/// Reads native PCM incrementally and converts it to interleaved stereo at the
/// requested output sample rate. Its methods are intended for the control thread.
pub(crate) struct StreamingDecoder {
    path: PathBuf,
    backend: DecoderBackend,
    source_channels: usize,
    source_sample_rate: u32,
    target_sample_rate: u32,
    source_buffer: Vec<f32>,
    // Exact source-frame position in units of 1 / target_sample_rate.
    source_position: u64,
    staging: Vec<f32>,
    eof: bool,
}

impl StreamingDecoder {
    pub(crate) fn open<P: AsRef<Path>>(path: P, target_sample_rate: u32) -> AudioResult<Self> {
        Self::open_internal(path, Some(target_sample_rate))
    }

    /// Opens a decoder without resampling; this is used when caching a complete
    /// sound effect so its source sample rate can be retained in `DecodedAudio`.
    pub(crate) fn open_native<P: AsRef<Path>>(path: P) -> AudioResult<Self> {
        Self::open_internal(path, None)
    }

    // Keep each codec's metadata extraction next to its validation and setup.
    #[allow(clippy::too_many_lines)]
    fn open_internal<P: AsRef<Path>>(
        path: P,
        requested_target_sample_rate: Option<u32>,
    ) -> AudioResult<Self> {
        let path = path.as_ref().to_path_buf();
        if requested_target_sample_rate == Some(0) {
            return Err(AudioError::InvalidState(
                "target sample rate must be greater than zero".to_owned(),
            ));
        }

        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let mut first_samples = Vec::new();
        let (backend, channels, sample_rate) = match extension.as_str() {
            "wav" => {
                let reader = hound::WavReader::open(&path)
                    .map_err(|error| AudioError::DecoderFailed(error.to_string()))?;
                let spec = reader.spec();
                let channels = usize::from(spec.channels);
                Self::validate_source(channels, spec.sample_rate)?;
                if spec.bits_per_sample == 0 || spec.bits_per_sample > 32 {
                    return Err(AudioError::DecoderFailed(format!(
                        "unsupported WAV bit depth: {}",
                        spec.bits_per_sample
                    )));
                }

                let backend = match spec.sample_format {
                    hound::SampleFormat::Float => {
                        if spec.bits_per_sample != 32 {
                            return Err(AudioError::UnsupportedFormat(
                                "only 32-bit floating-point WAV is supported".to_owned(),
                            ));
                        }
                        DecoderBackend::WavFloat(reader)
                    }
                    hound::SampleFormat::Int => DecoderBackend::WavInt {
                        reader,
                        bits: spec.bits_per_sample,
                    },
                };
                (backend, channels, spec.sample_rate)
            }
            "mp3" => {
                let file = File::open(&path)?;
                let mut decoder = minimp3::Decoder::new(BufReader::new(file));
                // Prime one frame because MP3 channel count and sample rate are
                // only known after a frame has been decoded.
                let frame = match decoder.next_frame() {
                    Ok(frame) => frame,
                    Err(minimp3::Error::Eof) => {
                        return Err(AudioError::DecoderFailed(
                            "MP3 contains no decodable audio frames".to_owned(),
                        ));
                    }
                    Err(error) => {
                        return Err(AudioError::DecoderFailed(error.to_string()));
                    }
                };
                let channels = frame.channels;
                let sample_rate = u32::try_from(frame.sample_rate).unwrap_or(0);
                Self::validate_source(channels, sample_rate)?;
                first_samples.extend(
                    frame
                        .data
                        .into_iter()
                        .map(|sample| f32::from(sample) / f32::from(i16::MAX)),
                );
                (DecoderBackend::Mp3(decoder), channels, sample_rate)
            }
            "ogg" => {
                let file = File::open(&path)?;
                let reader = lewton::inside_ogg::OggStreamReader::new(BufReader::new(file))
                    .map_err(|error| AudioError::DecoderFailed(error.to_string()))?;
                let channels = usize::from(reader.ident_hdr.audio_channels);
                let sample_rate = reader.ident_hdr.audio_sample_rate;
                Self::validate_source(channels, sample_rate)?;
                (DecoderBackend::Ogg(reader), channels, sample_rate)
            }
            "flac" => {
                let reader = claxon::FlacReader::open(&path)
                    .map_err(|error| AudioError::DecoderFailed(error.to_string()))?;
                let info = reader.streaminfo();
                let channels = info.channels as usize;
                let sample_rate = info.sample_rate;
                let bits = info.bits_per_sample;
                Self::validate_source(channels, sample_rate)?;
                if bits == 0 || bits > 32 {
                    return Err(AudioError::DecoderFailed(format!(
                        "unsupported FLAC bit depth: {bits}"
                    )));
                }
                (
                    DecoderBackend::Flac {
                        reader,
                        bits,
                        frame_buffer: Vec::new(),
                    },
                    channels,
                    sample_rate,
                )
            }
            _ => {
                return Err(AudioError::UnsupportedFormat(path.display().to_string()));
            }
        };

        let target_sample_rate = requested_target_sample_rate.unwrap_or(sample_rate);
        let decoder = Self {
            path,
            backend,
            source_channels: channels,
            source_sample_rate: sample_rate,
            target_sample_rate,
            source_buffer: first_samples,
            source_position: 0,
            staging: Vec::with_capacity(SOURCE_CHUNK_SAMPLES),
            eof: false,
        };

        if !decoder
            .source_buffer
            .len()
            .is_multiple_of(decoder.source_channels)
        {
            return Err(AudioError::DecoderFailed(
                "decoder returned an incomplete PCM frame".to_owned(),
            ));
        }

        Ok(decoder)
    }

    fn validate_source(channels: usize, sample_rate: u32) -> AudioResult<()> {
        if !(channels == 1 || channels == 2) {
            return Err(AudioError::UnsupportedFormat(format!(
                "the mixer supports mono or stereo sources, got {channels} channels"
            )));
        }
        if sample_rate == 0 {
            return Err(AudioError::DecoderFailed(
                "audio source has a zero sample rate".to_owned(),
            ));
        }
        Ok(())
    }

    pub(crate) fn target_sample_rate(&self) -> u32 {
        self.target_sample_rate
    }

    // Integer PCM normalization intentionally rounds to the mixer's f32 format.
    #[allow(clippy::cast_precision_loss)]
    fn read_next_source_chunk(&mut self) -> AudioResult<usize> {
        self.staging.clear();

        match &mut self.backend {
            DecoderBackend::WavFloat(reader) => {
                for sample in reader.samples::<f32>().take(SOURCE_CHUNK_SAMPLES) {
                    let sample =
                        sample.map_err(|error| AudioError::DecoderFailed(error.to_string()))?;
                    if !sample.is_finite() {
                        return Err(AudioError::DecoderFailed(
                            "WAV contains a non-finite sample".to_owned(),
                        ));
                    }
                    self.staging.push(sample);
                }
            }
            DecoderBackend::WavInt { reader, bits } => {
                let denominator = (1_i64 << u32::from(*bits - 1)) as f32;
                for sample in reader.samples::<i32>().take(SOURCE_CHUNK_SAMPLES) {
                    let sample =
                        sample.map_err(|error| AudioError::DecoderFailed(error.to_string()))?;
                    self.staging.push(sample as f32 / denominator);
                }
            }
            DecoderBackend::Mp3(decoder) => match decoder.next_frame() {
                Ok(frame) => {
                    if frame.channels != self.source_channels
                        || u32::try_from(frame.sample_rate).unwrap_or(0) != self.source_sample_rate
                    {
                        return Err(AudioError::UnsupportedFormat(
                            "MP3 changes sample rate or channel count mid-stream".to_owned(),
                        ));
                    }
                    self.staging.extend(
                        frame
                            .data
                            .into_iter()
                            .map(|sample| f32::from(sample) / f32::from(i16::MAX)),
                    );
                }
                Err(minimp3::Error::Eof) => {}
                Err(error) => return Err(AudioError::DecoderFailed(error.to_string())),
            },
            DecoderBackend::Ogg(reader) => {
                if let Some(packet) = read_ogg_pcm_packet(reader)? {
                    self.staging.extend(
                        packet
                            .into_iter()
                            .map(|sample| f32::from(sample) / f32::from(i16::MAX)),
                    );
                }
            }
            DecoderBackend::Flac {
                reader,
                bits,
                frame_buffer,
            } => {
                // Claxon's high-level `samples().take(n)` iterator can discard
                // unread samples in the currently decoded FLAC frame when it is
                // dropped. Read whole FLAC blocks instead, then interleave them
                // into `staging` before returning the buffer for reuse.
                let result = {
                    let mut blocks = reader.blocks();
                    blocks.read_next_or_eof(std::mem::take(frame_buffer))
                };

                match result {
                    Ok(Some(block)) => {
                        if block.channels() as usize != self.source_channels {
                            *frame_buffer = block.into_buffer();
                            return Err(AudioError::UnsupportedFormat(
                                "FLAC channel count changes between frames".to_owned(),
                            ));
                        }

                        let denominator = (1_i64 << (*bits - 1)) as f32;
                        let frames = block.duration() as usize;
                        let left = block.channel(0);
                        if self.source_channels == 1 {
                            for sample in left.iter().take(frames) {
                                self.staging.push(*sample as f32 / denominator);
                            }
                        } else {
                            let right = block.channel(1);
                            for frame in 0..frames {
                                self.staging.push(left[frame] as f32 / denominator);
                                self.staging.push(right[frame] as f32 / denominator);
                            }
                        }
                        *frame_buffer = block.into_buffer();
                    }
                    Ok(None) => {}
                    Err(error) => {
                        return Err(AudioError::DecoderFailed(error.to_string()));
                    }
                }
            }
        }

        if !self.staging.len().is_multiple_of(self.source_channels) {
            return Err(AudioError::DecoderFailed(
                "decoder returned an incomplete PCM frame".to_owned(),
            ));
        }
        Ok(self.staging.len())
    }

    fn ensure_source_frame(&mut self, index: usize) -> AudioResult<()> {
        while self.source_buffer.len() / self.source_channels <= index && !self.eof {
            let sample_count = self.read_next_source_chunk()?;
            if sample_count == 0 {
                self.eof = true;
                break;
            }
            self.source_buffer.extend_from_slice(&self.staging);
        }
        Ok(())
    }

    fn source_frame_index(&self) -> AudioResult<usize> {
        usize::try_from(self.source_position / u64::from(self.target_sample_rate)).map_err(|_| {
            AudioError::DecoderFailed("resampling source position is too large".to_owned())
        })
    }

    /// Fills an interleaved stereo output buffer and returns the number of
    /// samples written (not frames). The returned count is always even.
    // Integer phase makes EOF independent of read boundaries. Only the fraction
    // narrows to f32; its numerator is below the u32 target rate (exact in f64).
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub(crate) fn read_stereo(&mut self, output: &mut [f32]) -> AudioResult<usize> {
        if !output.len().is_multiple_of(2) {
            return Err(AudioError::InvalidState(
                "stereo output buffer length must be even".to_owned(),
            ));
        }

        let target_rate = u64::from(self.target_sample_rate);
        let requested_frames = output.len() / 2;
        let source_channels = self.source_channels;
        let mut written_frames = 0usize;

        while written_frames < requested_frames {
            let index = self.source_frame_index()?;
            self.ensure_source_frame(index.saturating_add(1))?;

            let source_frames = self.source_buffer.len() / source_channels;
            if index >= source_frames {
                break;
            }

            let next_index = (index + 1).min(source_frames - 1);
            let fraction = ((self.source_position % target_rate) as f64
                / f64::from(self.target_sample_rate)) as f32;
            let left0 = self.source_buffer[index * source_channels];
            let left1 = self.source_buffer[next_index * source_channels];
            let left = left0 + (left1 - left0) * fraction;
            let right = if source_channels == 1 {
                left
            } else {
                let right0 = self.source_buffer[index * source_channels + 1];
                let right1 = self.source_buffer[next_index * source_channels + 1];
                right0 + (right1 - right0) * fraction
            };

            output[written_frames * 2] = left;
            output[written_frames * 2 + 1] = right;
            written_frames += 1;
            self.source_position = self
                .source_position
                .checked_add(u64::from(self.source_sample_rate))
                .ok_or_else(|| {
                    AudioError::DecoderFailed("resampling source position is too large".to_owned())
                })?;
        }

        // Keep one source frame for interpolation across chunk boundaries.
        let source_frames = self.source_buffer.len() / source_channels;
        let discard_frames = self
            .source_frame_index()?
            .saturating_sub(1)
            .min(source_frames);
        if discard_frames > 0 {
            self.source_buffer.drain(..discard_frames * source_channels);
            // discard_frames <= source_position / target_rate, so this product fits.
            self.source_position -= discard_frames as u64 * target_rate;
        }

        Ok(written_frames * 2)
    }

    pub(crate) fn reopen(&self) -> AudioResult<Self> {
        Self::open(&self.path, self.target_sample_rate)
    }
}

fn read_ogg_pcm_packet(reader: &mut OggReader) -> AudioResult<Option<Vec<i16>>> {
    // Vorbis priming packets can produce no PCM; only None is EOF.
    while let Some(packet) = reader
        .read_dec_packet_itl()
        .map_err(|error| AudioError::DecoderFailed(error.to_string()))?
    {
        if !packet.is_empty() {
            return Ok(Some(packet));
        }
    }
    Ok(None)
}
