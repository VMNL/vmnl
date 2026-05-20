////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct AudioBusState
{
    pub volume: f32,
    pub muted: bool,
    pub paused: bool,
}

#[derive(Clone, Debug)]
pub struct AudioBus
{
    state: Arc<Mutex<AudioBusState>>,
}

impl AudioBus
{
    pub fn new() -> Self
    {
        Self {
            state: Arc::new(Mutex::new(AudioBusState {
                volume: 1.0,
                muted: false,
                paused: false,
            }))
        }
    }

    pub fn set_volume(&self, volume: f32)
    {
        let mut state = self.state.lock().unwrap();

        state.volume = volume.clamp(0.0, 1.0);
    }

    pub fn volume(&self) -> f32
    {
        let state = self.state.lock().unwrap();

        state.volume
    }

    pub fn mute(&self)
    {
        let mut state = self.state.lock().unwrap();

        state.muted = true;
    }

    pub fn unmute(&self)
    {
        let mut state = self.state.lock().unwrap();

        state.muted = false;
    }

    pub fn is_muted(&self) -> bool
    {
        let state = self.state.lock().unwrap();

        state.muted
    }

    pub fn pause(&self)
    {
        let mut state = self.state.lock().unwrap();

        state.paused = true;
    }

    pub fn resume(&self)
    {
        let mut state = self.state.lock().unwrap();

        state.paused = false;
    }

    pub fn is_paused(&self) -> bool
    {
        let state = self.state.lock().unwrap();

        state.paused
    }
}