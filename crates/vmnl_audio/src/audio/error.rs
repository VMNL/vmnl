// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use std::fmt::{Display, Formatter};

/// Failures from configuration, control synchronization, decoding, and device setup.
#[derive(Debug)]
pub enum AudioError {
    /// The default output device or stream could not be queried, built, or started.
    BackendInitFailed(String),
    /// The optional VMNL control worker could not be spawned.
    ControlThreadStartFailed(String),
    /// Source metadata, decoding, or format conversion failed.
    DecoderFailed(String),
    /// A requested gain, rate, or buffer shape is invalid.
    InvalidState(String),
    /// The codec, channel count, or output sample format is unsupported.
    UnsupportedFormat(String),
    /// The global command queue mutex was poisoned.
    CommandQueuePoisoned,
    /// The decoded sound cache lock was poisoned.
    SoundCachePoisoned,
    /// The active sound list mutex was poisoned.
    ActiveSoundVoicesPoisoned,
    /// The active music list mutex was poisoned.
    ActiveMusicStreamsPoisoned,
    /// A streaming decoder or its scratch-buffer mutex was poisoned.
    MusicDecoderPoisoned,
    /// A source-file operation failed with an I/O error.
    Io(std::io::Error),
}

/// Result type returned by fallible audio operations.
pub type AudioResult<T> = Result<T, AudioError>;

impl Display for AudioError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendInitFailed(message) => write!(f, "Audio backend init failed: {message}"),
            Self::ControlThreadStartFailed(message) => {
                write!(f, "Audio control thread start failed: {message}")
            }
            Self::DecoderFailed(message) => write!(f, "Audio decoder failed: {message}"),
            Self::InvalidState(message) => write!(f, "Invalid audio state: {message}"),
            Self::UnsupportedFormat(message) => write!(f, "Unsupported audio format: {message}"),
            Self::CommandQueuePoisoned => write!(f, "Audio command queue lock was poisoned"),
            Self::SoundCachePoisoned => write!(f, "Audio sound cache lock was poisoned"),
            Self::ActiveSoundVoicesPoisoned => write!(f, "Active sound voices lock was poisoned"),
            Self::ActiveMusicStreamsPoisoned => write!(f, "Active music streams lock was poisoned"),
            Self::MusicDecoderPoisoned => write!(f, "Music decoder lock was poisoned"),
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

pub fn validate_gain(gain: f32) -> AudioResult<f32> {
    if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
        return Err(AudioError::InvalidState(
            "gain must be finite and between 0.0 and 1.0".to_string(),
        ));
    }

    Ok(gain)
}
