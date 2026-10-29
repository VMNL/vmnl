// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::error::{AudioError, AudioResult};
use crate::audio::runtime::AudioRuntime;
use parking_lot::Mutex;
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Non-realtime worker for command processing, stream refill and cleanup.
pub(crate) struct ControlWorker {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    background_error: Arc<Mutex<Option<String>>>,
}

/// Select exactly one control producer. A failed automatic worker never falls
/// back to a caller-side tick, even after its diagnostic has been consumed.
pub(crate) fn update_control(
    runtime: &AudioRuntime,
    worker: Option<&ControlWorker>,
) -> AudioResult<()> {
    if let Some(worker) = worker {
        worker.ensure_running()
    } else {
        runtime.tick()
    }
}

impl ControlWorker {
    pub(crate) fn spawn(
        runtime: Arc<AudioRuntime>,
        background_error: Arc<Mutex<Option<String>>>,
    ) -> AudioResult<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let thread_error = background_error.clone();

        let join = thread::Builder::new()
            .name("vmnl-audio-control".to_owned())
            .spawn(move || {
                run_control(|| runtime.tick(), &stop_thread, &thread_error);
            })
            .map_err(|error| AudioError::ControlThreadStartFailed(error.to_string()))?;

        Ok(Self {
            stop,
            join: Some(join),
            background_error,
        })
    }

    pub(crate) fn ensure_running(&self) -> AudioResult<()> {
        if self.stop.load(Ordering::Acquire)
            || self.join.as_ref().is_none_or(JoinHandle::is_finished)
        {
            return Err(AudioError::InvalidState(
                "automatic audio control worker has stopped; inspect take_background_error()"
                    .to_owned(),
            ));
        }
        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            join.thread().unpark();
            // Joining also waits for any in-progress decoding or I/O in tick();
            // the sleep interval does not bound shutdown latency.
            if let Err(payload) = join.join() {
                *self.background_error.lock() = Some(panic_message(payload.as_ref()));
            }
        }
    }
}

fn run_control(
    mut tick: impl FnMut() -> AudioResult<()>,
    stop: &AtomicBool,
    background_error: &Mutex<Option<String>>,
) {
    // Failure is terminal: unwinding may poison runtime locks, so never retry
    // the cycle or silently fall back to another control producer.
    let result = catch_unwind(AssertUnwindSafe(|| -> AudioResult<()> {
        while !stop.load(Ordering::Acquire) {
            tick()?;
            thread::park_timeout(Duration::from_millis(5));
        }
        Ok(())
    }));
    let error = match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(format!(
            "audio control worker stopped after an error: {error}"
        )),
        Err(payload) => Some(panic_message(payload.as_ref())),
    };
    if let Some(error) = error {
        let mut diagnostic = background_error.lock();
        *diagnostic = Some(error);
        // A caller taking this diagnostic must also observe terminal control.
        stop.store(true, Ordering::Release);
    } else {
        stop.store(true, Ordering::Release);
    }
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic payload");
    format!("audio control worker stopped after a panic: {message}")
}

impl Drop for ControlWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    // These fixtures intentionally panic or fail loudly on broken thread gates.
    #![allow(clippy::expect_used, clippy::panic)]

    use super::{run_control, AtomicBool, AudioError, ControlWorker, Mutex, Ordering};
    use std::cell::Cell;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::atomic::AtomicUsize;
    use std::sync::{mpsc, Arc};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn panicking_control_cycle_becomes_an_observable_failure() {
        type Cycle = fn() -> super::AudioResult<()>;
        let cases: [(Cycle, &str); 3] = [
            (|| panic!("controlled tick panic"), "controlled tick panic"),
            (
                || std::panic::panic_any("owned panic".to_owned()),
                "owned panic",
            ),
            (|| std::panic::panic_any(7_u8), "non-string panic payload"),
        ];
        for (tick, message) in cases {
            let stop = AtomicBool::new(false);
            let errors = Mutex::new(None);
            let result = catch_unwind(AssertUnwindSafe(|| run_control(tick, &stop, &errors)));
            assert!(result.is_ok(), "worker must catch its control-cycle panic");
            let error = errors.lock().take().expect("observable worker failure");
            assert!(error.contains(message));
            assert!(stop.load(Ordering::Acquire));
        }
    }

    #[test]
    fn failed_control_cycle_is_reported_once_and_never_retried() {
        let stop = AtomicBool::new(false);
        let errors = Mutex::new(None);
        let calls = Cell::new(0);
        run_control(
            || {
                calls.set(calls.get() + 1);
                Err(AudioError::InvalidState("controlled failure".to_owned()))
            },
            &stop,
            &errors,
        );
        assert_eq!(calls.get(), 1);
        assert!(errors
            .lock()
            .take()
            .expect("worker error")
            .contains("controlled failure"));
        assert!(stop.load(Ordering::Acquire));
    }

    #[test]
    fn automatic_update_does_not_run_a_second_control_cycle() {
        use crate::audio::{decoder::AudioDecoder, sound::SoundVoice, PlaybackState};
        let source = tempfile::Builder::new()
            .suffix(".wav")
            .tempfile()
            .expect("fixture");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 8_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(source.path(), spec).expect("WAV writer");
        writer.write_sample(8_192_i16).expect("left");
        writer.write_sample(-4_096_i16).expect("right");
        writer.finalize().expect("finish WAV");
        let pcm = Arc::new(AudioDecoder::decode_file(source.path()).expect("decode"));
        let voice =
            Arc::new(SoundVoice::new(1, pcm, 1.0, false, PlaybackState::Playing).expect("voice"));
        let runtime = super::AudioRuntime::new();
        runtime
            .register_sound_voice(voice.clone())
            .expect("register");
        runtime.force_pause_all().expect("queue pause");
        let (release_tx, release_rx) = mpsc::channel();
        let join = thread::spawn(move || {
            let _ = release_rx.recv();
        });
        let mut worker = ControlWorker {
            stop: Arc::new(AtomicBool::new(false)),
            join: Some(join),
            background_error: Arc::new(Mutex::new(None)),
        };
        // Ensure unwinding disconnects the gate before worker Drop joins it.
        let release = release_tx;
        super::update_control(&runtime, Some(&worker)).expect("automatic update");
        assert_eq!(
            voice.state(),
            PlaybackState::Playing,
            "queued pause was not applied by the caller"
        );
        super::update_control(&runtime, None).expect("manual update");
        assert_eq!(voice.state(), PlaybackState::Paused);
        release.send(()).expect("release worker");
        worker.stop();
        assert!(super::update_control(&runtime, Some(&worker)).is_err());
    }

    #[test]
    fn stop_reports_a_panic_outside_the_control_cycle_and_is_idempotent() {
        let errors = Arc::new(Mutex::new(None));
        let join: thread::JoinHandle<()> = thread::spawn(|| panic!("unhandled worker panic"));
        let mut worker = ControlWorker {
            stop: Arc::new(AtomicBool::new(false)),
            join: Some(join),
            background_error: errors.clone(),
        };
        worker.stop();
        assert!(worker.ensure_running().is_err());
        assert!(errors
            .lock()
            .take()
            .expect("join panic")
            .contains("unhandled worker panic"));
        worker.stop();
        drop(worker);
        assert!(
            errors.lock().is_none(),
            "a joined panic is not reported twice"
        );
    }

    #[test]
    fn stop_waits_for_the_in_flight_cycle_and_prevents_another_cycle() {
        let stop = Arc::new(AtomicBool::new(false));
        let errors = Arc::new(Mutex::new(None));
        let calls = Arc::new(AtomicUsize::new(0));
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let worker_stop = stop.clone();
        let worker_errors = errors.clone();
        let worker_calls = calls.clone();
        let join = thread::spawn(move || {
            run_control(
                || {
                    worker_calls.fetch_add(1, Ordering::Relaxed);
                    entered_tx.send(()).expect("announce running cycle");
                    release_rx.recv().expect("release in-flight cycle");
                    Ok(())
                },
                &worker_stop,
                &worker_errors,
            );
        });
        entered_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("worker entered cycle");
        let mut worker = ControlWorker {
            stop: stop.clone(),
            join: Some(join),
            background_error: errors.clone(),
        };
        thread::scope(|scope| {
            // Drop/disconnect this gate before joining the stopper on unwind.
            let release = release_tx;
            scope.spawn(move || {
                worker.stop();
                let _ = done_tx.send(());
            });
            let deadline = Instant::now() + Duration::from_secs(3);
            while !stop.load(Ordering::Acquire) && Instant::now() < deadline {
                thread::yield_now();
            }
            assert!(stop.load(Ordering::Acquire), "stop request was issued");
            assert!(matches!(done_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            release.send(()).expect("finish in-flight work");
            done_rx
                .recv_timeout(Duration::from_secs(3))
                .expect("stop joined worker");
        });
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert!(errors.lock().is_none());
    }
}
