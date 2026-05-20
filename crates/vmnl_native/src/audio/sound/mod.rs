////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::decoder::DecodedAudio;
use crate::audio::device::AudioDevice;
use crate::audio::error::AudioError;
use crate::audio::sound::instance::{PlaybackState, SoundInstance,};
use crate::audio::sound::voice::SoundVoice;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub mod instance;
pub mod voice;

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
    voice: Arc<Mutex<SoundVoice>>,
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
        let instance = Arc::new(Mutex::new(SoundInstance
        {
            cursor: 0,
            volume: 1.0,
            looping: false,
            state: PlaybackState::Playing,
        }));

        let voice = Arc::new(Mutex::new(SoundVoice::new(
            self.decoded_audio.clone(),
            instance.clone(),
        )));

        self.device.register_sound_voice(voice.clone());

        Ok(SoundHandle { voice })
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
        let voice = self.voice.lock().unwrap();
        let mut instance = voice.instance.lock().unwrap();

        instance.state = PlaybackState::Stopped;
        instance.cursor = 0;
    }

    pub fn pause(&self)
    {
        let voice = self.voice.lock().unwrap();
        let mut instance = voice.instance.lock().unwrap();

        instance.state = PlaybackState::Paused;
    }

    pub fn resume(&self)
    {
        let voice = self.voice.lock().unwrap();
        let mut instance = voice.instance.lock().unwrap();

        instance.state = PlaybackState::Playing;
    }


    pub fn set_volume(&self, volume: f32)
    {
        let volume = volume.clamp(0.0, 1.0);

        let voice = self.voice.lock().unwrap();
        let mut instance = voice.instance.lock().unwrap();

        instance.volume = volume;
    }

    pub fn set_looping(&self, looping: bool)
    {
        let voice = self.voice.lock().unwrap();
        let mut instance = voice.instance.lock().unwrap();

        instance.looping = looping;
    }

    pub fn is_playing(&self) -> bool
    {
        let voice = self.voice.lock().unwrap();
        let instance = voice.instance.lock().unwrap();

        instance.state == PlaybackState::Playing
    }

    pub fn is_paused(&self) -> bool
    {
        let voice = self.voice.lock().unwrap();
        let instance = voice.instance.lock().unwrap();

        instance.state == PlaybackState::Paused
    }

    pub fn is_stopped(&self) -> bool
    {
        let voice = self.voice.lock().unwrap();
        let instance = voice.instance.lock().unwrap();

        instance.state == PlaybackState::Stopped
    }
}