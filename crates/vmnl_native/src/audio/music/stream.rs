////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use miniaudio::{Decoder, DecoderConfig};

use std::fs::File;
use std::io::{BufReader, Read};

use crate::audio::error::AudioError;

pub struct MusicStream
{
    decoder: Decoder,
    finished: bool,
}

impl MusicStream
{
    pub fn from_file(path: &std::path::Path) -> Result<Self, AudioError>
    {
        let decoder = Decoder::from_file(path,Some(&DecoderConfig::default()),)
        .map_err(|e| {
            AudioError::DecoderFailed(e.to_string())
        })?;

        Ok(Self {
            decoder,
            finished: false,
        })
    }

    pub fn read_chunk(&mut self, buffer: &mut [u8]) -> Result<usize, AudioError>
    {
        if self.finished
        {
            return Ok(0);
        }

        let frames_read = self.decoder
            .read_pcm_frames(output)
            .map_err(|e| {
                AudioError::DecoderFailed(e.to_string())
            })?;

        if frames_read == 0
        {
            self.finished = true;
        }

        Ok(frames_read)
    }

    pub fn finished(&self) -> bool
    {
        self.finished
    }
}