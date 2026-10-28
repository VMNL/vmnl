// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Native audio setup; an explicitly requested backend test must fail on setup errors.

use std::sync::{Mutex, MutexGuard};
use vmnl_audio::AudioDevice;

static BACKEND_TEST_LOCK: Mutex<()> = Mutex::new(());

pub fn lock_backend_tests() -> MutexGuard<'static, ()> {
    BACKEND_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn device(threaded: bool) -> AudioDevice {
    AudioDevice::builder()
        .thread(threaded)
        .build()
        .expect("backend test requires a usable CPAL stereo output device")
}
