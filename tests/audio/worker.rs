// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Control-worker lifecycle without an audio device.

use crate::common::{wait_until, TempAudioFile};
use std::time::Duration;

use crate::internal_harness::audio::decoder::AudioDecoder;
use crate::internal_harness::audio::runtime::AudioRuntime;
use crate::internal_harness::audio::sound::SoundVoice;
use crate::internal_harness::audio::worker::ControlWorker;
use crate::internal_harness::audio::PlaybackState;
use std::sync::Arc;

#[test]
fn control_worker_processes_commands_and_joins_without_audio_hardware() {
    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 8_000, 80, 1_000);
    let decoded = Arc::new(AudioDecoder::decode_file(wav.path()).expect("decode worker fixture"));
    let voice = Arc::new(
        SoundVoice::new(1, decoded, 1.0, true, PlaybackState::Playing)
            .expect("create looping voice"),
    );
    let runtime = Arc::new(AudioRuntime::new());
    runtime
        .register_sound_voice(voice.clone())
        .expect("register worker test voice");
    runtime.force_pause_all().expect("queue pause command");

    let background_error = Arc::new(parking_lot::Mutex::new(None));
    let mut worker = ControlWorker::spawn(runtime, background_error.clone())
        .expect("spawn control worker without a CPAL device");

    assert!(
        wait_until(Duration::from_secs(1), || voice.state()
            == PlaybackState::Paused),
        "worker did not apply the queued pause command"
    );
    worker.stop();
    assert!(background_error.lock().is_none());

    // stop() joins the worker and is safe to call more than once; Drop is also
    // expected to be harmless after an explicit stop.
    worker.stop();
    drop(worker);
}
