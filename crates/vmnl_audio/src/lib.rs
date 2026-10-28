// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

//! Independent audio playback with a private CPAL output backend.
//!
//! [`Sound`] keeps decoded effects in memory; [`Music`] opens a streaming decoder
//! for each playback. [`AudioDevice`] owns the output stream and its resolved
//! stereo sample rate. Resource loading and manual control may allocate and do
//! file I/O; they must not run on the audio callback.
//!
//! The current builder enables a control worker by default. Select
//! [`AudioDeviceBuilder::thread`] with `false` and call [`AudioDevice::update`]
//! to drive control work manually. The backend still executes its audio callback.
//! Dropping the device stops output and joins its worker; resources and cloned
//! handles can retain shared runtime data but do not keep that output stream alive.
//!
//! # Development status
//!
//! This API is experimental. Playback transitions, muted cursor progression,
//! sound cursor bounds, and realtime reclamation still require
//! qualification or fixes. The required target is recorded in the
//! [audio acceptance contract](https://github.com/VMNL/vmnl/blob/main/docs/audio.md).
//! The planned facade feature and opt-in worker default are not implemented yet.
//!
//! # Configure without opening a device
//!
//! ```
//! use vmnl_audio::{AudioConfig, AudioResult};
//!
//! # fn main() -> AudioResult<()> {
//! let mut config = AudioConfig::default();
//! config.set_sample_rate(48_000)?;
//! config.set_master_volume(0.5)?;
//! assert_eq!(config.sample_rate(), 48_000);
//! # Ok(())
//! # }
//! ```
//!
//! # Manual control
//!
//! Opening a device needs a usable native output backend. This example compiles
//! without running that backend; the application must keep updating its control
//! work while playback is active.
//!
//! ```no_run
//! use vmnl_audio::{AudioConfig, AudioDevice, AudioResult};
//!
//! # fn main() -> AudioResult<()> {
//! let audio = AudioDevice::builder()
//!     .config(AudioConfig::default())
//!     .thread(false)
//!     .build()?;
//! audio.update()?; // Call again from the application's control loop.
//! # Ok(())
//! # }
//! ```

pub mod audio;

pub use audio::{
    AudioBus, AudioConfig, AudioDevice, AudioDeviceBuilder, AudioError, AudioResult, Music,
    MusicHandle, MusicPlayConfig, PlaybackState, Sound, SoundHandle, SoundPlayConfig,
};
