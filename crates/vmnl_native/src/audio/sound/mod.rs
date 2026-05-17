////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::decoder::{AudioDecoder, DecodedAudio};
use crate::audio::device::AudioDevice;
use crate::audio::error::AudioError;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct SoundState
{
    volume: f32,
    looping: bool,
    playing: bool,
}

#[derive(Clone)]
pub struct Sound
{
    device: AudioDevice,
    path: PathBuf,
    decoded_audio: Arc<DecodedAudio>,
}

#[derive(Clone)]
pub struct SoundHandle
{
    state: Arc<Mutex<SoundState>>,
}

impl Sound
{
    pub(crate) fn from_file<P>(device: AudioDevice, path: P) -> Result<Self, AudioError>
    where
        P: AsRef<Path>
    {
        let decoded_audio = AudioDecoder::decode_file(&path)?;

        Ok(Self {device, path: path.as_ref().to_path_buf(), decoded_audio: Arc::new(decoded_audio)})
    }

    pub fn play(&self) -> Result<SoundHandle, AudioError>
    {
        let state = SoundState {
            volume: 1.0,
            looping: false,
            playing: true,
        };

        // TODO:
        // Submit sound to Miniaudio backend

        Ok(SoundHandle { state: Arc::new(Mutex::new(state)) })
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
        let mut state = self.state.lock().unwrap();
        state.playing = false;
    }

    pub fn pause(&self)
    {
        let mut state = self.state.lock().unwrap();
        state.playing = false;
    }

    pub fn resume(&self)
    {
        let mut state = self.state.lock().unwrap();
        state.playing = true;
    }

    pub fn set_volume(&self, volume: f32)
    {
        let mut state = self.state.lock().unwrap();
        state.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_looping(&self, looping: bool)
    {
        let mut state = self.state.lock().unwrap();
        state.looping = looping;
    }

    pub fn is_playing(&self) -> bool
    {
        let state = self.state.lock().unwrap();
        state.playing
    }
}