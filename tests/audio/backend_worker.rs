// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Native automatic/manual control-worker behavior.

use std::time::Duration;

use crate::backend_support::{device, lock_backend_tests};
use crate::common::{wait_until, TempAudioFile};
use vmnl_audio::{AudioConfig, AudioDevice, AudioError, ControlMode, MusicPlayConfig};

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn default_device_constructors_resolve_to_manual_control() {
    let _guard = lock_backend_tests();
    let built = AudioDevice::builder().build().expect("default builder");
    assert_eq!(built.control_mode(), ControlMode::Manual);
    drop(built);
    let direct = AudioDevice::new(AudioConfig::default()).expect("default constructor");
    assert_eq!(direct.control_mode(), ControlMode::Manual);
}

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

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn failed_automatic_worker_rejects_public_control_calls() {
    let _guard = lock_backend_tests();
    let device = device(true);
    let file = TempAudioFile::new("wav");
    file.write_pcm16_wav(2, 44_100, 512, 0);
    let music = device.load_music(file.path()).expect("load source");
    let handle = music
        .play_with_config(MusicPlayConfig {
            volume: 0.0,
            looping: true,
        })
        .expect("prefill before removing the source");
    std::fs::remove_file(file.path()).expect("force looping refill failure");
    assert!(
        wait_until(Duration::from_secs(3), || matches!(
            device.update(),
            Err(AudioError::InvalidState(_))
        )),
        "failed automatic worker must be observable through update"
    );
    assert!(handle.is_stopped());
    assert!(device.take_background_error().is_some());
    assert!(device.take_background_error().is_none());
    assert!(matches!(
        device.stop_all(),
        Err(AudioError::InvalidState(_))
    ));
    assert!(matches!(
        device.pause_all(),
        Err(AudioError::InvalidState(_))
    ));
    assert!(matches!(
        device.resume_all(),
        Err(AudioError::InvalidState(_))
    ));
    assert!(matches!(device.update(), Err(AudioError::InvalidState(_))));
}
