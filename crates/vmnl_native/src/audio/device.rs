////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::bus::AudioBus;
use crate::audio::decoder::DecodedAudio;
use crate::audio::error::AudioError;
use crate::audio::music::Music;
use crate::audio::sound::instance::SoundInstance;
use crate::audio::sound::Sound;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
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

pub(crate) struct AudioBackend
{
    pub engine: miniaudio::Engine,
    pub master_volume: f32,
    pub sound_cache: HashMap<PathBuf, Arc<DecodedAudio>>,
    pub active_sound_instances: Vec<Arc<Mutex<SoundInstance>>>,

    pub music_bus: AudioBus,
    pub sfx_bus: AudioBus,
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
        let mut engine = miniaudio::Engine::new(&miniaudio::EngineConfig::default())
            .map_err(|e| {AudioError::BackendInitFailed(e.to_string())})?;
        
        engine.set_volume(config.master_volume);

        let backend = AudioBackend {
            master_volume: config.master_volume,
            engine,
            sound_cache: HashMap::new(),
            active_sound_instances: Vec::new(),

            music_bus: AudioBus::new(),
            sfx_bus: AudioBus::new(),
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
        backend.master_volume = volume;
        backend.engine.set_volume(volume);
    }

    pub fn master_volume(&self) -> f32
    {
        let backend = self.backend.lock().unwrap();
        backend.master_volume
    }

    pub fn get_or_decode_audio<P>(&self, path: P) -> Result <Arc<DecodedAudio>, AudioError>
    where 
        P: AsRef<Path>
    {
        let path = path.as_ref().to_path_buf();
        {
            let backend = self.backend.lock().unwrap();

            if let Some(decoded_audio) = backend.sound_cache.get(&path)
            {
                return Ok(decoded_audio.clone());
            }
        }
        let decoded_audio = Arc::new(AudioDecoder::decode_file(&path)?);

        let mut backend = self.backend.lock().unwrap();

        backend
            .sound_cache
            .insert(path, decoded_audio.clone());

        Ok(decoded_audio)
    }

    pub(crate) fn register_sound_instance(&self, instance: Arc<Mutex<SoundInstance>>)
    {
        let mut backend = self.backend.lock().unwrap();

        backend.active_sound_instances.push(instance);
    }

    pub fn update(&self)
    {
        let mut backend = self.backend.lock().unwrap();

        backend.active_sound_instances.retain(|instance| {
            let instance = instance.lock().unwrap();

            instance.state != crate::audio::sound::instance::PlaybackState::Stopped
        });
    }

    pub fn music_bus(&self) -> AudioBus
    {
        let backend = self.backend.lock().unwrap();

        backend.music_bus.clone()
    }

    pub fn sfx_bus(&self) -> AudioBus
    {
        let backend = self.backend.lock().unwrap();

        backend.sfx_bus.clone()
    }
}
