// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Device-independent public and implementation-conformance tests for `vmnl_audio`.
//!
//! This target is registered from `crates/vmnl_audio/Cargo.toml`, but every
//! test file remains in the repository-level `tests/audio/` directory.

// Test fixtures fail loudly with expect/panic; exact float checks verify
// round trips and controlled PCM samples, not approximate DSP measurements.
#![allow(clippy::expect_used, clippy::panic, clippy::float_cmp)]
#![allow(clippy::module_inception)]

mod allocation_probe;
mod common;
mod contracts;
mod decoding;
mod gain_semantics;
mod internal_harness;
mod ogg_chains;
mod output_config;
mod playback_states;
mod realtime;
mod runtime_mixer;
mod snapshots;
mod worker;

// Included production sources resolve their private paths through `crate::audio`.
pub(crate) use internal_harness::audio;
