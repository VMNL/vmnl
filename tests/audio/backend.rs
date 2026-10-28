// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Opt-in integration tests requiring a usable CPAL stereo output device.

// Test fixtures fail loudly with expect/panic; exact float checks verify
// round trips and controlled PCM samples, not approximate DSP measurements.
#![allow(clippy::expect_used, clippy::panic, clippy::float_cmp)]

mod backend_support;
mod backend_worker;
mod bus;
mod common;
mod playback;
