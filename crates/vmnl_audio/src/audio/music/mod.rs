////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
mod handle;
mod stream;

use crate::audio::bus::BusKind;
use crate::audio::decoder::DecodedAudio;
use crate::audio::device::AudioDevice;
use crate::audio::error::AudioError;
use crate::audio::runtime::AudioRuntime;

use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use handle::MusicHandle;
pub use stream::MusicStream;

#[derive(Clone)]
pub struct Music {
    runtime: Arc<AudioRuntime>,
    path: PathBuf,
    decoded_audio: Arc<DecodedAudio>,
}

impl Music {
    pub(crate) fn from_file<P>(device: AudioDevice, path: P) -> Result<Self, AudioError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let runtime = device.runtime();
        let decoded_audio = runtime.get_or_decode_audio(&path)?;

        Ok(Self {
            runtime,
            path,
            decoded_audio,
        })
    }

    pub fn play(&self) -> Result<MusicHandle, AudioError> {
        let id = self.runtime.next_stream_id();
        let stream = Arc::new(MusicStream::new(
            id,
            self.path.clone(),
            self.decoded_audio.clone(),
            BusKind::Music,
        ));
        self.runtime.register_music_stream(stream.clone());
        Ok(MusicHandle::new(stream))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn decoded_audio(&self) -> &DecodedAudio {
        self.decoded_audio.as_ref()
    }
}
