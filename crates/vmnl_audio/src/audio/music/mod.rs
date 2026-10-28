// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

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

/// Playback settings for one music instance; defaults to gain 1.0 without looping.
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

/// Reusable music resource bound to the creating device's runtime.
///
/// Clones share runtime and source path. Each play call opens a separate decoder
/// and PCM ring, with file I/O, allocation, and initial refill on the caller.
/// Resources retain runtime data but do not keep the device's output stream alive.
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

    /// Starts a new playback with [`MusicPlayConfig::default`].
    ///
    /// # Errors
    ///
    /// Reports the same failures as [`Self::play_with_config`].
    pub fn play(&self) -> AudioResult<MusicHandle> {
        self.play_with_config(MusicPlayConfig::default())
    }

    /// Starts an independent playback using these gain and looping settings.
    ///
    /// Opens and decodes initial source data into a new buffer before publishing
    /// the stream; this may block on file I/O and allocate on the caller.
    /// Call this outside the realtime callback. Dropping the returned handle alone
    /// does not stop the runtime-owned playback.
    ///
    /// # Errors
    ///
    /// Returns invalid-gain, source-file/decoder, unsupported-format, or
    /// active-stream-list synchronization errors.
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

    /// Returns the stored source path as supplied when loading, without canonicalization.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}
