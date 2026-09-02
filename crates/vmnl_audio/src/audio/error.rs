////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AudioError {
    BackendInitFailed(String),
    DecoderFailed(String),
    InvalidState(String),
    UnsupportedFormat(String),
    CommandQueuePoisoned,
    SoundCachePoisoned,
    ActiveSoundVoicesPoisoned,
    ActiveMusicStreamsPoisoned,
    Io(std::io::Error),
}

pub type AudioResult<T> = Result<T, AudioError>;

impl Display for AudioError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendInitFailed(message) => write!(f, "Audio backend init failed: {message}"),
            Self::DecoderFailed(message) => write!(f, "Audio decoder failed: {message}"),
            Self::InvalidState(message) => write!(f, "Invalid audio state: {message}"),
            Self::UnsupportedFormat(message) => write!(f, "Unsupported audio format: {message}"),
            Self::CommandQueuePoisoned => write!(f, "Audio command queue lock was poisoned"),
            Self::SoundCachePoisoned => write!(f, "Audio sound cache lock was poisoned"),
            Self::ActiveSoundVoicesPoisoned => write!(f, "currently in test"),
            Self::ActiveMusicStreamsPoisoned => write!(f, "currently in test"),
            Self::Io(error) => write!(f, "IO error: {error}"),
        }
    }
}

impl std::error::Error for AudioError {}

impl From<std::io::Error> for AudioError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
