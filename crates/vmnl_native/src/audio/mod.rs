////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////

pub mod bus;
pub mod device;
pub mod decoder;
pub mod error;
pub mod instance;
pub mod mixer;
pub mod music;
pub mod sound;
pub mod voice;

pub use bus::AudioBus;
pub use device::{AudioConfig, AudioDevice};
pub use error::AudioError;
pub use music::{Music, MusicHandle};
pub use sound::{Sound, SoundHandle};
