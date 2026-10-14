// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Observable automatic/manual control-worker behaviour. These tests need CPAL output hardware.

use std::time::Duration;

use crate::common::{default_device_or_skip, device_or_skip, lock_backend_tests, wait_until, TempAudioFile};
use vmnl_audio::MusicPlayConfig;

use crate::internal_harness::audio::decoder::AudioDecoder;
use crate::internal_harness::audio::runtime::AudioRuntime;
use crate::internal_harness::audio::sound::SoundVoice;
use crate::internal_harness::audio::worker::ControlWorker;
use crate::internal_harness::audio::PlaybackState;
use std::sync::Arc;

#[test]
fn automatic_worker_refills_music_without_client_update_calls() {
    let _guard = lock_backend_tests();
    let Some(device) = default_device_or_skip() else {
        return;
    };

    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 44_100, 44_100 * 4, 2_000);
    let music = device.load_music(wav.path()).expect("load test music");
    let handle = music
        .play_with_config(MusicPlayConfig { volume: 0.5, looping: true })
        .expect("start looping music");

    // The ring buffer initially holds about 500 ms. Moving significantly past
    // that capacity without calling update() proves the control worker refills it.
    let refill_threshold = device.sample_rate() as usize * 65 / 100;
    assert!(
        wait_until(Duration::from_secs(3), || handle.cursor_frames() >= refill_threshold),
        "automatic worker failed to refill the music buffer without update()"
    );
    assert!(device.take_background_error().is_none());
    handle.stop();
}

#[test]
fn manual_mode_requires_update_to_refill_an_exhausted_music_buffer() {
    let _guard = lock_backend_tests();
    let Some(device) = device_or_skip(false) else {
        return;
    };

    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 44_100, 44_100 * 4, 2_000);
    let music = device.load_music(wav.path()).expect("load test music");
    let handle = music
        .play_with_config(MusicPlayConfig { volume: 0.5, looping: true })
        .expect("start looping music");

    let sample_rate = device.sample_rate() as usize;
    let prefill_threshold = sample_rate * 45 / 100;
    assert!(
        wait_until(Duration::from_secs(2), || handle.cursor_frames() >= prefill_threshold),
        "manual stream did not consume its initial prefetched buffer"
    );

    let before_update = handle.cursor_frames();
    let progress_threshold = before_update + sample_rate / 10;
    let advanced_without_update = wait_until(Duration::from_millis(250), || {
        handle.cursor_frames() >= progress_threshold
    });
    assert!(
        !advanced_without_update,
        "manual mode unexpectedly refilled a drained ring without update()"
    );

    device.update().expect("manual update should refill the stream");
    assert!(
        wait_until(Duration::from_secs(2), || handle.cursor_frames() >= progress_threshold),
        "manual update did not allow the music stream to resume"
    );

    handle.stop();
    assert!(device.take_background_error().is_none());
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
    runtime.register_sound_voice(voice.clone()).expect("register worker test voice");
    runtime.force_pause_all().expect("queue pause command");

    let background_error = Arc::new(parking_lot::Mutex::new(None));
    let mut worker = ControlWorker::spawn(runtime, background_error.clone())
        .expect("spawn control worker without a CPAL device");

    assert!(
        wait_until(Duration::from_secs(1), || voice.state() == PlaybackState::Paused),
        "worker did not apply the queued pause command"
    );
    worker.stop();
    assert!(background_error.lock().is_none());

    // stop() joins the worker and is safe to call more than once; Drop is also
    // expected to be harmless after an explicit stop.
    worker.stop();
    drop(worker);
}
