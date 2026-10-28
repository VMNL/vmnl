// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Native automatic/manual control-worker behavior.

use std::time::Duration;

use crate::backend_support::{device, lock_backend_tests};
use crate::common::{wait_until, TempAudioFile};
use vmnl_audio::MusicPlayConfig;

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn automatic_worker_refills_music_without_client_update_calls() {
    let _guard = lock_backend_tests();
    let device = device(true);

    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 44_100, 44_100 * 4, 2_000);
    let music = device.load_music(wav.path()).expect("load test music");
    let handle = music
        .play_with_config(MusicPlayConfig {
            volume: 0.5,
            looping: true,
        })
        .expect("start looping music");

    // The ring buffer initially holds about 500 ms. Moving significantly past
    // that capacity without calling update() proves the control worker refills it.
    let refill_threshold = device.sample_rate() as usize * 65 / 100;
    assert!(
        wait_until(Duration::from_secs(3), || handle.cursor_frames()
            >= refill_threshold),
        "automatic worker failed to refill the music buffer without update()"
    );
    assert!(device.take_background_error().is_none());
    handle.stop();
}

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn manual_mode_requires_update_to_refill_an_exhausted_music_buffer() {
    let _guard = lock_backend_tests();
    let device = device(false);

    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 44_100, 44_100 * 4, 2_000);
    let music = device.load_music(wav.path()).expect("load test music");
    let handle = music
        .play_with_config(MusicPlayConfig {
            volume: 0.5,
            looping: true,
        })
        .expect("start looping music");

    let sample_rate = device.sample_rate() as usize;
    let prefill_threshold = sample_rate * 45 / 100;
    assert!(
        wait_until(Duration::from_secs(2), || handle.cursor_frames()
            >= prefill_threshold),
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

    device
        .update()
        .expect("manual update should refill the stream");
    assert!(
        wait_until(Duration::from_secs(2), || handle.cursor_frames()
            >= progress_threshold),
        "manual update did not allow the music stream to resume"
    );

    handle.stop();
    assert!(device.take_background_error().is_none());
}
