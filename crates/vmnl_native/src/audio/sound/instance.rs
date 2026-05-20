////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState
{
    Playing,
    Paused,
    Stopped,
}

#[derive(Debug)]
pub struct SoundInstance
{
    pub(crate) cursor: usize,
    pub(crate) volume: f32,
    pub(crate) looping: bool,
    pub(crate) state: PlaybackState,
}

impl Default for SoundInstance
{
    fn default() -> Self
    {
        Self {
            cursor: 0,
            volume: 1.0,
            looping: false,
            state: PlaybackState::Stopped,
        }
    }
}