// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

//! Audio devices, resources, playback handles, buses, and errors.

mod bus;
mod decoder;
mod device;
mod error;
mod mixer;
mod music;
mod output_config;
mod runtime;
mod snapshot;
mod sound;
mod stream_decoder;
mod worker;

pub use bus::AudioBus;
pub use device::{AudioConfig, AudioDevice, AudioDeviceBuilder};
pub use error::{AudioError, AudioResult};
pub use music::{Music, MusicHandle, MusicPlayConfig};
pub use sound::{PlaybackState, Sound, SoundHandle, SoundPlayConfig};
