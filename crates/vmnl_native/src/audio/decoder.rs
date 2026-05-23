////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::error::AudioError;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub channels: u32,
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

impl DecodedAudio {
    pub fn frame_count(&self) -> usize {
        if self.channels == 0 {
            0
        } else {
            self.samples.len() / self.channels as usize
        }
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

pub struct AudioDecoder;

impl AudioDecoder {
    pub fn decode_file<P>(path: P) -> Result<DecodedAudio, AudioError>
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

    fn decode_wav(path: &Path) -> Result<DecodedAudio, AudioError> {
        let mut reader =
            hound::WavReader::open(path).map_err(|e| AudioError::DecoderFailed(e.to_string()))?;
        let spec = reader.spec();
        let channels = spec.channels.max(1) as u32;
        let sample_rate = spec.sample_rate.max(1);

        let samples = match spec.sample_format {
            hound::SampleFormat::Float => {
                let mut out = Vec::new();
                for sample in reader.samples::<f32>() {
                    out.push(sample.map_err(|e| AudioError::DecoderFailed(e.to_string()))?);
                }
                out
            }
            hound::SampleFormat::Int => {
                let bits = spec.bits_per_sample.max(1);
                let denom = (1i64 << (bits.saturating_sub(1) as u32)).max(1) as f32;
                let mut out = Vec::new();
                if bits <= 16 {
                    for sample in reader.samples::<i16>() {
                        out.push(
                            sample.map_err(|e| AudioError::DecoderFailed(e.to_string()))? as f32
                                / denom,
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

    fn decode_mp3(path: &Path) -> Result<DecodedAudio, AudioError> {
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
                    samples.extend(frame.data.into_iter().map(|s| s as f32 / i16::MAX as f32));
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

    fn decode_ogg(path: &Path) -> Result<DecodedAudio, AudioError> {
        let file = File::open(path)?;
        let mut reader = lewton::inside_ogg::OggStreamReader::new(BufReader::new(file))
            .map_err(|e| AudioError::DecoderFailed(e.to_string()))?;
        let channels = reader.ident_hdr.audio_channels.max(1) as u32;
        let sample_rate = reader.ident_hdr.audio_sample_rate.max(1);
        let mut samples = Vec::new();

        while let Some(packet) = reader
            .read_dec_packet_itl()
            .map_err(|e| AudioError::DecoderFailed(e.to_string()))?
        {
            samples.extend(packet.into_iter().map(|s| s as f32 / i16::MAX as f32));
        }

        Ok(DecodedAudio {
            channels,
            sample_rate,
            samples,
        })
    }

    fn decode_flac(path: &Path) -> Result<DecodedAudio, AudioError> {
        let mut reader =
            claxon::FlacReader::open(path).map_err(|e| AudioError::DecoderFailed(e.to_string()))?;
        let info = reader.streaminfo();
        let channels = info.channels.max(1);
        let sample_rate = info.sample_rate.max(1);
        let bits = info.bits_per_sample.max(1);
        let denom = (1i64 << bits.saturating_sub(1)).max(1) as f32;
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
}
