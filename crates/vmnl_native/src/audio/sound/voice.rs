///////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::decoder::DecodedAudio;
use crate::audio::sound::instance::SoundInstance;

use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct SoundVoice
{
    pub decoded_audio: Arc<DecodedAudio>,
    pub instance: Arc<Mutex<SoundInstance>>,
}

impl SoundVoice
{
    pub fn new(decoded_audio: Arc<DecodedAudio>, instance: Arc<Mutex<SoundInstance>>,) -> Self
    {
        Self
        {
            decoded_audio,
            instance,
        }
    }
}