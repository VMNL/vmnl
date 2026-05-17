////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::error::AudioError;
use crate::audio::music::Music;
use crate::audio::sound::Sound;

use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct AudioConfig
{
    pub master_volume: f32
}

impl Default for AudioConfig
{
    fn default() -> Self
    {
        Self {
            master_volume: 1.0
        }
    }
}

#[derive(Debug)]
pub(crate) struct AudioBackend
{
    pub master_volume: f32
}

#[derive(Clone)]
pub struct AudioDevice
{
    backend: Arc<Mutex<AudioBackend>>
}

impl AudioDevice
{
    pub fn new(config: AudioConfig) -> Result<Self, AudioError>
    {
        let backend = AudioBackend {
            master_volume: config.master_volume
        };

        Ok(Self {backend: Arc::new(Mutex::new(backend))})
    }

    pub fn load_sound<P>(&self, path: P) -> Result<Sound, AudioError>
    where
        P: AsRef<Path>
    {
        Sound::from_file(self.clone(), path)
    }

    pub fn load_music<P>(&self, path: P) -> Result<Music, AudioError>
    where
        P: AsRef<Path>
    {
        Music::from_file(self.clone(), path)
    }

    pub fn set_master_volume(&self, volume: f32)
    {
        let mut backend = self.backend.lock().unwrap();
        backend.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn master_volume(&self) -> f32
    {
        let backend = self.backend.lock().unwrap();
        backend.master_volume
    }
}