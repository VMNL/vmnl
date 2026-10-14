////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
////////////////////////////////////////////////////////////////////////////////
mod bus;
mod decoder;
mod device;
mod error;
mod mixer;
mod music;
mod runtime;
mod sound;
mod stream_decoder;
mod worker;

pub use bus::AudioBus;
pub use device::{AudioConfig, AudioDevice, AudioDeviceBuilder};
pub use error::{AudioError, AudioResult};
pub use music::{Music, MusicHandle, MusicPlayConfig};
pub use sound::{PlaybackState, Sound, SoundHandle, SoundPlayConfig};
