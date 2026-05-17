////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::error::AudioError;

use std::path::Path;

#[derive(Debug, Clone)]
pub struct DecodedAudio
{
    pub channels: u32,
    pub sample_rate: u32,
    pub samples: Vec<f32>
}

pub struct AudioDecoder;

impl AudioDecoder
{
    pub fn decode_file<P>(path: P) -> Result<DecodedAudio, AudioError>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();

        if !path.exists()
        {
            return Err(AudioError::DecoderFailed(format!("File does not exist: {:}",path)));
        }

        // TODO:
        // Miniaudio decoder integration

        Ok(DecodedAudio {
            channels: 2,
            sample_rate: 44100,
            samples: Vec::new(),
        })
    }
}