// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Integration and implementation-conformance tests for `vmnl_audio`.
//!
//! This target is registered from `crates/vmnl_audio/Cargo.toml`, but every
//! test file remains in the repository-level `tests/audio/` directory.

#![allow(clippy::module_inception)]

mod bus;
mod common;
mod contracts;
mod decoding;
mod internal_harness;
mod playback;
mod runtime_mixer;
mod worker;
