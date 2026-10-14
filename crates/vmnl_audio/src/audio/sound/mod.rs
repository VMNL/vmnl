////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
////////////////////////////////////////////////////////////////////////////////
mod handle;
mod voice;

use crate::audio::decoder::DecodedAudio;
use crate::audio::device::AudioDevice;
use crate::audio::error::AudioResult;
use crate::audio::runtime::AudioRuntime;

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use handle::SoundHandle;
pub use voice::PlaybackState;
pub(crate) use voice::SoundVoice;

#[derive(Debug, Clone, Copy)]
pub struct SoundPlayConfig {
    /// Linear gain between 0.0 (silent) and 1.0 (full volume).
    pub volume: f32,
    /// Repeat the sound effect until its handle is stopped.
    pub looping: bool,
}

impl Default for SoundPlayConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            looping: false,
        }
    }
}

#[derive(Clone)]
pub struct Sound {
    runtime: Arc<AudioRuntime>,
    path: PathBuf,
    decoded_audio: Arc<DecodedAudio>,
}

impl Sound {
    pub(crate) fn from_file<P: AsRef<Path>>(device: &AudioDevice, path: P) -> AudioResult<Self> {
        let path = path.as_ref().to_path_buf();
        let runtime = device.runtime();
        let decoded_audio = device.get_or_decode_audio(&path)?;

        Ok(Self {
            runtime,
            path,
            decoded_audio,
        })
    }

    pub fn play(&self) -> AudioResult<SoundHandle> {
        self.play_with_config(SoundPlayConfig::default())
    }

    pub fn play_with_config(&self, config: SoundPlayConfig) -> AudioResult<SoundHandle> {
        let id = self.runtime.next_voice_id();
        let voice = Arc::new(SoundVoice::new(
            id,
            self.decoded_audio.clone(),
            config.volume,
            config.looping,
            PlaybackState::Playing,
        )?);

        // Publish the voice only after its settings have all been initialized.
        self.runtime.register_sound_voice(voice.clone())?;
        Ok(SoundHandle::new(voice))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}
