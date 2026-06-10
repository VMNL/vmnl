////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
pub mod bus;
pub mod decoder;
pub mod device;
pub mod error;
pub mod mixer;
pub mod music;
pub mod runtime;
pub mod sound;

pub use bus::{AudioBus, BusKind};
pub use device::{AudioConfig, AudioDevice};
pub use error::AudioError;
pub use mixer::AudioMixer;
pub use music::{Music, MusicHandle, MusicStream};
pub use runtime::{AudioCommand, AudioRuntime};
pub use sound::{PlaybackState, Sound, SoundHandle, SoundVoice};
