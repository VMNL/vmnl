// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Native audio setup; an explicitly requested backend test must fail on setup errors.

use std::sync::{Mutex, MutexGuard};
use vmnl_audio::{AudioDevice, ControlMode};

static BACKEND_TEST_LOCK: Mutex<()> = Mutex::new(());

pub fn lock_backend_tests() -> MutexGuard<'static, ()> {
    BACKEND_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn device(threaded: bool) -> AudioDevice {
    let device = AudioDevice::builder()
        .thread(threaded)
        .build()
        .expect("backend test requires a usable CPAL stereo output device");
    assert_eq!(
        device.control_mode(),
        if threaded {
            ControlMode::Automatic
        } else {
            ControlMode::Manual
        }
    );
    device
}
