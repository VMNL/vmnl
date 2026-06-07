////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::bus::AudioBus;
use crate::audio::decoder::DecodedAudio;
use crate::audio::error::AudioError;
use crate::audio::music::Music;
use crate::audio::runtime::{AudioCommand, AudioRuntime};
use crate::audio::sound::Sound;

use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub sample_rate: u32,
    pub channels: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sample_rate: 44_100,
            channels: 2,
        }
    }
}

#[derive(Clone)]
pub struct AudioDevice {
    runtime: Arc<AudioRuntime>,
    sample_rate: u32,
    channels: u32,
}

impl AudioDevice {
    pub fn new(config: AudioConfig) -> Result<Self, AudioError> {
        if config.channels == 0 {
            return Err(AudioError::InvalidState(
                "channels must be greater than zero".to_string(),
            ));
        }
        if config.sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "sample_rate must be greater than zero".to_string(),
            ));
        }

        let runtime = Arc::new(AudioRuntime::new());
        runtime.master_bus.set_volume(config.master_volume);

        Ok(Self {
            runtime,
            sample_rate: config.sample_rate,
            channels: config.channels,
        })
    }

    pub fn runtime(&self) -> Arc<AudioRuntime> {
        self.runtime.clone()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn load_sound<P>(&self, path: P) -> Result<Sound, AudioError>
    where
        P: AsRef<Path>,
    {
        Sound::from_file(self.clone(), path)
    }

    pub fn load_music<P>(&self, path: P) -> Result<Music, AudioError>
    where
        P: AsRef<Path>,
    {
        Music::from_file(self.clone(), path)
    }

    pub fn set_master_volume(&self, volume: f32) {
        self.runtime.enqueue(AudioCommand::SetMasterVolume(volume));
        self.update();
    }

    pub fn master_volume(&self) -> f32 {
        self.runtime.master_bus.volume()
    }

    pub fn music_bus(&self) -> AudioBus {
        self.runtime.music_bus.clone()
    }

    pub fn sfx_bus(&self) -> AudioBus {
        self.runtime.sfx_bus.clone()
    }

    pub fn get_or_decode_audio<P>(&self, path: P) -> Result<Arc<DecodedAudio>, AudioError>
    where
        P: AsRef<Path>,
    {
        self.runtime.get_or_decode_audio(path)
    }

    pub fn update(&self) {
        self.runtime.apply_commands();
        self.runtime.pump_music_streams();
        self.runtime.cleanup();
    }

    pub fn render_into(&self, output: &mut [f32]) {
        self.update();
        self.runtime.mix_into(output);
    }

    pub fn stop_all(&self) {
        self.runtime.force_stop_all();
        self.update();
    }

    pub fn pause_all(&self) {
        self.runtime.force_pause_all();
        self.update();
    }

    pub fn resume_all(&self) {
        self.runtime.force_resume_all();
        self.update();
    }

    pub fn set_max_sound_voices(&self, max: usize) {
        self.runtime.enqueue(AudioCommand::SetMaxVoices(max));
        self.update();
    }
}
