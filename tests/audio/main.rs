// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Device-independent public and implementation-conformance tests for `vmnl_audio`.
//!
//! This target is registered from `crates/vmnl_audio/Cargo.toml`, but every
//! test file remains in the repository-level `tests/audio/` directory.

#![allow(clippy::module_inception)]

mod common;
mod contracts;
mod decoding;
mod internal_harness;
mod runtime_mixer;
mod worker;

// Included production sources resolve their private paths through `crate::audio`.
pub(crate) use internal_harness::audio;
