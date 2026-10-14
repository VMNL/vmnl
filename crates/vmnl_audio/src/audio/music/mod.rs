////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
////////////////////////////////////////////////////////////////////////////////
mod handle;
mod stream;

use crate::audio::device::AudioDevice;
use crate::audio::error::AudioResult;
use crate::audio::runtime::AudioRuntime;
use crate::audio::stream_decoder::StreamingDecoder;

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use handle::MusicHandle;
pub(crate) use stream::MusicStream;

#[derive(Debug, Clone, Copy)]
pub struct MusicPlayConfig {
    /// Linear gain between 0.0 (silent) and 1.0 (full volume).
    pub volume: f32,
    /// Rewind and continue playing when the end of the source is reached.
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
    pub(crate) fn from_file<P: AsRef<Path>>(device: &AudioDevice, path: P) -> AudioResult<Self> {
        let path = path.as_ref().to_path_buf();
        let sample_rate = device.sample_rate();

        // Validate the file/codec without decoding the complete track into RAM.
        // The actual streaming decoder is created for each playback instance.
        drop(StreamingDecoder::open(&path, sample_rate)?);

        Ok(Self {
            runtime: device.runtime(),
            path,
            sample_rate,
        })
    }

    pub fn play(&self) -> AudioResult<MusicHandle> {
        self.play_with_config(MusicPlayConfig::default())
    }

    pub fn play_with_config(&self, config: MusicPlayConfig) -> AudioResult<MusicHandle> {
        let stream = Arc::new(MusicStream::new(
            self.path.clone(),
            config.volume,
            config.looping,
            crate::audio::PlaybackState::Playing,
            self.sample_rate,
        )?);

        // Prefill before publishing the stream to the realtime mixer.
        stream.pump()?;
        self.runtime.register_music_stream(stream.clone())?;

        Ok(MusicHandle::new(stream))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}
