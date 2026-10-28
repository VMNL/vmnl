// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::error::{AudioError, AudioResult};
use crate::audio::runtime::AudioRuntime;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Non-realtime worker for command processing, stream refill and cleanup.
pub(crate) struct ControlWorker {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl ControlWorker {
    pub(crate) fn spawn(
        runtime: Arc<AudioRuntime>,
        background_error: Arc<Mutex<Option<String>>>,
    ) -> AudioResult<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();

        let join = thread::Builder::new()
            .name("vmnl-audio-control".to_owned())
            .spawn(move || {
                while !stop_thread.load(Ordering::Acquire) {
                    if let Err(error) = runtime.tick() {
                        *background_error.lock() = Some(format!(
                            "audio control worker stopped after an error: {error}"
                        ));
                        break;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
            })
            .map_err(|error| AudioError::ControlThreadStartFailed(error.to_string()))?;

        Ok(Self {
            stop,
            join: Some(join),
        })
    }

    pub(crate) fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            // Joining also waits for any in-progress decoding or I/O in tick();
            // the sleep interval does not bound shutdown latency.
            let _ = join.join();
        }
    }
}

impl Drop for ControlWorker {
    fn drop(&mut self) {
        self.stop();
    }
}
