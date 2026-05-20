////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::decoder::DecodedAudio;
use crate::audio::device::AudioDevice;
use crate::audio::error::AudioError;
use crate::audio::sound::instance::{PlaybackState, SoundInstance,};

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub mod instance;

#[derive(Clone)]
pub struct Sound
{
    device: AudioDevice,
    path: PathBuf,
    #[allow(dead_code)]
    decoded_audio: Arc<DecodedAudio>,
}

#[derive(Clone)]
pub struct SoundHandle
{
    state: Arc<Mutex<SoundInstance>>,
    sound: Arc<Mutex<miniaudio::Sound<'static>>>,
}

impl Sound
{
    pub(crate) fn from_file<P>(device: AudioDevice, path: P) -> Result<Self, AudioError>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref().to_path_buf();
        let decoded_audio = device.get_or_decode_audio(&path)?;

        Ok(Self {device, path, decoded_audio})
    }

    pub fn play(&self) -> Result<SoundHandle, AudioError>
    {
        let backend = self.device.backend.lock().unwrap();

        let mut sound = miniaudio::Sound::from_file(&backend.engine, self.path.to_str().unwrap(), miniaudio::SoundFlags::DECODE,)
        .map_err(|e| {
            AudioError::DecoderFailed(e.to_string())
        })?;

        sound.start().map_err(|e| {
            AudioError::InvalidState(e.to_string())
        })?;

        drop(backend);

        let instance = SoundInstance {
            cursor: 0,
            volume: 1.0,
            looping: false,
            state: PlaybackState::Playing,
        };

        self.device
            .register_sound_instance(instance.clone());

        Ok(SoundHandle {
            instance: Arc::new(Mutex::new(instance)),
            sound: Arc::new(Mutex::new(sound)),
        })
    }

    pub fn path(&self) -> &Path
    {
        &self.path
    }
}

impl SoundHandle
{
    pub fn stop(&self)
    {
        {
            let mut instance = self.instance.lock().unwrap();

            instance.state = PlaybackState::Stopped;
            instance.cursor = 0;
        }

        let mut sound = self.sound.lock().unwrap();

        let _ = sound.stop();
    }

    pub fn pause(&self)
    {
        {
            let mut instance = self.instance.lock().unwrap();

            instance.state = PlaybackState::Paused;
        }

        let mut sound = self.sound.lock().unwrap();

        let _ = sound.stop();
    }

    pub fn resume(&self)
    {
        {
            let mut instance = self.instance.lock().unwrap();

            instance.state = PlaybackState::Playing;
        }

        let mut sound = self.sound.lock().unwrap();

        let _ = sound.start();
    }

    pub fn set_volume(&self, volume: f32)
    {
        let volume = volume.clamp(0.0, 1.0);

        {
            let mut instance = self.instance.lock().unwrap();

            instance.volume = volume;
        }

        let mut sound = self.sound.lock().unwrap();

        sound.set_volume(volume);
    }

    pub fn set_looping(&self, looping: bool)
    {
        {
            let mut instance = self.instance.lock().unwrap();

            instance.looping = looping;
        }

        let mut sound = self.sound.lock().unwrap();

        sound.set_looping(looping);
    }

    pub fn is_playing(&self) -> bool
    {
        let instance = self.instance.lock().unwrap();

        instance.state == PlaybackState::Playing;
    }

    pub fn is_paused(&self) -> bool
    {
        let instance = self.instance.lock().unwrap();

        instance.state == PlaybackState::Paused
    }

    pub fn is_stopped(&self) -> bool
    {
        let instance = self.instance.lock().unwrap();

        instance.state == PlaybackState::Stopped
    }
}