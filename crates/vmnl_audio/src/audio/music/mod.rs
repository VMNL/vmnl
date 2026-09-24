////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
mod handle;
mod stream;

use crate::audio::bus::BusKind;
use crate::audio::device::AudioDevice;
use crate::audio::error::{AudioError, AudioResult};
use crate::audio::runtime::AudioRuntime;

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use handle::MusicHandle;
pub(crate) use stream::MusicStream;

#[derive(Debug, Clone, Copy)]
pub struct MusicPlayConfig {
    pub volume: f32,
    pub looping: bool,
}

impl Default for MusicPlayConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            looping: false,
        }
    }
}

#[derive(Clone)]
pub struct Music {
    runtime: Arc<AudioRuntime>,
    path: PathBuf,
    sample_rate: u32,
}

impl Music {
    pub(crate) fn from_file<P>(
        device: &AudioDevice,
        path: P,
    ) -> AudioResult<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let sample_rate = device.sample_rate();

        /*
         * Validate the file using a streaming decoder.
         *
         * Nothing is fully decoded into memory here.
         */
        let decoder_config = miniaudio::DecoderConfig::new(
            miniaudio::Format::F32,
            2,
            sample_rate,
        );

        miniaudio::Decoder::from_file(
            &path,
            Some(&decoder_config),
        )
        .map_err(|error| {
            AudioError::DecoderFailed(error.to_string())
        })?;

        Ok(Self {
            runtime: device.runtime(),
            path,
            sample_rate,
        })
    }

    pub fn play(&self) -> AudioResult<MusicHandle> {
        self.play_with_config(MusicPlayConfig::default())
    }

    pub fn play_with_config(
        &self,
        config: MusicPlayConfig,
    ) -> AudioResult<MusicHandle> {
        let id = self.runtime.next_stream_id();

        let stream = Arc::new(
            MusicStream::new(
                id,
                self.path.clone(),
                BusKind::Music,
                config.volume,
                config.looping,
                crate::audio::PlaybackState::Playing,
                self.sample_rate,
            )?,
        );

        /*
         * Fill the streaming buffer BEFORE exposing the stream to the mixer.
         */
        stream.pump()?;

        self.runtime
            .register_music_stream(stream.clone())?;

        Ok(MusicHandle::new(stream))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}