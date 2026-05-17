////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use crate::audio::device::AudioDevice;
use crate::audio::error::AudioError;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct MusicState
{
    volume: f32,
    looping: bool,
    playing: bool,
}

#[derive(Clone)]
pub struct Music
{
    device: AudioDevice,
    path: PathBuf,
}

#[derive(Clone)]
pub struct MusicHandle
{
    state: Arc<Mutex<MusicState>>,
}

impl Music
{
    pub(crate) fn from_file<P>(device: AudioDevice, path: P) -> Result<Self, AudioError>
    where
        P: AsRef<Path>
    {
        let path = path.as_ref();

        if !path.exists()
        {
            return Err(AudioError::DecoderFailed(format!("Music file does not exist: {:}", path)));
        }

        Ok(Self { device, path: path.to_path_buf() })
    }

    pub fn play(&self) -> Result<MusicHandle, AudioError>
    {
        let state = MusicState {
            volume: 1.0,
            looping: false,
            playing: true,
        };

        // TODO:
        // Create streaming decoder
        // Feed Miniaudio rolling buffer

        Ok(MusicHandle { state: Arc::new(Mutex::new(state)) })
    }

    pub fn path(&self) -> &Path
    {
        &self.path
    }
}

impl MusicHandle
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