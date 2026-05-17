////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

pub mod device;
pub mod decoder;
pub mod error;
pub mod music;
pub mod sound;

pub use device::{AudioConfig, AudioDevice};
pub use error::AudioError;
pub use music::{Music, MusicHandle};
pub use sound::{Sound, SoundHandle};