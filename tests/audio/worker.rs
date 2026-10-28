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

use crate::audio::decoder::test_support::decoded_audio;
use crate::audio::error::AudioError;
use crate::audio::music::MusicStream;

fn failing_runtime() -> (TempAudioFile, Arc<AudioRuntime>, Arc<MusicStream>) {
    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(2, 8_000, &[8_192, -4_096]);
    let stream = Arc::new(
        MusicStream::new(
            file.path().to_path_buf(),
            1.0,
            true,
            PlaybackState::Playing,
            8_000,
        )
        .expect("open valid source before its removal"),
    );
    let runtime = Arc::new(AudioRuntime::new());
    runtime
        .register_music_stream(stream.clone())
        .expect("register stream");
    std::fs::remove_file(file.path()).expect("force a reopen error at looping EOF");
    (file, runtime, stream)
}

#[test]
fn refill_error_stops_the_failed_stream_and_is_returned_to_manual_control() {
    let (file, runtime, stream) = failing_runtime();
    let expected = hound::WavReader::open(file.path())
        .err()
        .expect("source is missing")
        .to_string();
    assert!(
        matches!(runtime.tick(), Err(AudioError::DecoderFailed(message)) if message == expected),
        "original refill error must propagate"
    );
    assert!(
        stream.is_stopped(),
        "a failed stream must not remain Playing"
    );
    assert!(runtime
        .music_stream_snapshot
        .read()
        .expect("exclusive reader")
        .is_empty());
    runtime
        .tick()
        .expect("failed stream was removed, not retried");
}

#[test]
fn refill_failure_does_not_skip_cleanup_of_stopped_sound() {
    let (_file, runtime, _stream) = failing_runtime();
    let pcm = Arc::new(decoded_audio(2, 8_000, vec![0.25, -0.125]));
    let voice =
        Arc::new(SoundVoice::new(1, pcm, 1.0, false, PlaybackState::Playing).expect("valid voice"));
    runtime
        .register_sound_voice(voice.clone())
        .expect("register voice");
    voice.stop();
    assert!(runtime.tick().is_err());
    assert!(runtime
        .sound_voice_snapshot
        .read()
        .expect("exclusive reader")
        .is_empty());
}

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
    worker.ensure_running().expect("healthy automatic worker");

    assert!(
        wait_until(Duration::from_secs(1), || voice.state()
            == PlaybackState::Paused),
        "worker did not apply the queued pause command"
    );
    worker.stop();
    assert!(worker.ensure_running().is_err());
    assert!(background_error.lock().is_none());

    // stop() joins the worker and is safe to call more than once; Drop is also
    // expected to be harmless after an explicit stop.
    worker.stop();
    drop(worker);
}

#[test]
fn automatic_refill_failure_remains_observable_after_taking_the_error() {
    let (file, runtime, stream) = failing_runtime();
    let expected = hound::WavReader::open(file.path())
        .err()
        .expect("source is missing")
        .to_string();
    let errors = Arc::new(parking_lot::Mutex::new(None));
    let mut worker = ControlWorker::spawn(runtime.clone(), errors.clone()).expect("spawn worker");
    assert!(wait_until(Duration::from_secs(3), || errors
        .lock()
        .is_some()));
    let error = errors.lock().take().expect("background failure");
    assert!(error.contains("stopped after an error"));
    assert!(
        error.contains(&expected),
        "retain the decoder's original diagnostic"
    );
    assert!(stream.is_stopped());
    assert!(
        worker.ensure_running().is_err(),
        "clearing the diagnostic cannot revive control"
    );
    assert!(errors.lock().is_none());
    worker.stop();
    worker.stop();
    drop(worker);
    assert_eq!(
        Arc::strong_count(&runtime),
        1,
        "joined worker releases its runtime"
    );
}

#[test]
fn manual_control_can_refill_a_healthy_stream_after_removing_the_failure() {
    let (_failed_file, runtime, failed) = failing_runtime();
    let healthy_file = TempAudioFile::new("wav");
    healthy_file.write_pcm16_samples(2, 8_000, &[16_384, -8_192]);
    let healthy = Arc::new(
        MusicStream::new(
            healthy_file.path().to_path_buf(),
            1.0,
            false,
            PlaybackState::Playing,
            8_000,
        )
        .expect("valid healthy stream"),
    );
    runtime
        .register_music_stream(healthy.clone())
        .expect("register healthy stream");
    assert!(runtime.tick().is_err());
    assert!(failed.is_stopped());
    runtime
        .tick()
        .expect("next manual cycle refills the remaining stream");
    let mut output = [0.0; 2];
    runtime.mix_into(&mut output);
    assert_eq!(output, [0.5, -0.25]);
    assert_eq!(healthy.cursor_frames(), 1);
    assert!(healthy.is_stopped());
}
