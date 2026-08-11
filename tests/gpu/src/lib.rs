// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! GPU/display test crate for VMNL public behavior.

use std::sync::{Mutex, MutexGuard, OnceLock};

static GPU_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Serializes tests because GLFW and the windowing backend share process-global state.
pub fn gpu_test_guard() -> MutexGuard<'static, ()> {
    GPU_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
