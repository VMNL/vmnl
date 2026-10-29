// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public playback-handle contracts. Tests require an explicitly selected native backend.

use crate::backend_support::{device, lock_backend_tests};
use crate::common::{ogg_chain, wait_until, TempAudioFile};
use std::time::Duration;
use vmnl_audio::{AudioError, MusicPlayConfig, SoundPlayConfig};

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn sound_handle_clones_control_the_same_playback_instance() {
    let _guard = lock_backend_tests();
    let device = device(true);

    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 44_100, 44_100 * 3, 4_000);
    let sound = device.load_sound(wav.path()).expect("load valid sound");

    let handle = sound
        .play_with_config(SoundPlayConfig {
            volume: 0.5,
            looping: true,
        })
        .expect("play looping sound");
    let clone = handle.clone();

    assert!(handle.is_playing());
    assert!(clone.set_volume(f32::NAN).is_err());
    clone.set_volume(0.25).expect("valid volume");

    clone.pause();
    assert!(handle.is_paused(), "clone must pause the original playback");
    handle.resume();
    assert!(
        clone.is_playing(),
        "resume must be visible through every handle clone"
    );

    clone.set_looping(false);
    handle.stop();
    assert!(clone.is_stopped());

    assert!(matches!(
        sound.play_with_config(SoundPlayConfig {
            volume: f32::INFINITY,
            looping: false
        }),
        Err(AudioError::InvalidState(_))
    ));
}

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn music_handle_clones_share_pause_stop_volume_and_looping_state() {
    let _guard = lock_backend_tests();
    let device = device(true);

    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 44_100, 44_100 * 3, 2_000);
    let music = device.load_music(wav.path()).expect("load valid music");
    assert!(matches!(
        music.play_with_config(MusicPlayConfig {
            volume: f32::NAN,
            looping: true
        }),
        Err(AudioError::InvalidState(_))
    ));
    let handle = music
        .play_with_config(MusicPlayConfig {
            volume: 0.6,
            looping: true,
        })
        .expect("play looping music");
    let clone = handle.clone();

    assert!(handle.is_playing());
    assert!(clone.set_volume(f32::INFINITY).is_err());
    clone.set_volume(0.3).expect("valid volume");
    clone.set_looping(false);

    handle.pause();
    assert!(clone.is_paused());
    clone.resume();
    assert!(handle.is_playing());

    clone.stop();
    assert!(handle.is_stopped());
}

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn device_load_methods_report_unknown_extensions_and_missing_files() {
    let _guard = lock_backend_tests();
    let device = device(false);

    let unknown = TempAudioFile::new("unsupported-audio");
    unknown.write_bytes(b"unknown format");
    assert!(matches!(
        device.load_sound(unknown.path()),
        Err(AudioError::UnsupportedFormat(_))
    ));

    let missing =
        std::env::temp_dir().join(format!("vmnl-audio-no-file-{}.mp3", std::process::id()));
    let _ = std::fs::remove_file(&missing);
    assert!(device.load_music(&missing).is_err());
}

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn chained_ogg_is_rejected_by_sound_loading_and_music_prefill() {
    let _guard = lock_backend_tests();
    let device = device(false);
    for second in [
        "chain-next-mono-16k.ogg",
        "chain-next-stereo-8k.ogg",
        "chain-next-stereo-16k.ogg",
    ] {
        let file = ogg_chain("chain-start-mono-8k.ogg", second);
        assert!(matches!(
            device.load_sound(file.path()),
            Err(AudioError::UnsupportedFormat(_))
        ));
        let music = device
            .load_music(file.path())
            .expect("initial metadata is valid");
        assert!(matches!(
            music.play(),
            Err(AudioError::UnsupportedFormat(_))
        ));
        device.update().expect("failed prefill published no stream");
    }
}

#[test]
#[ignore = "requires a usable CPAL stereo output device"]
fn chained_ogg_late_refill_error_stops_the_public_music_handle() {
    let _guard = lock_backend_tests();
    let device = device(false);
    for second in [
        "chain-next-mono-16k.ogg",
        "chain-next-stereo-8k.ogg",
        "chain-next-stereo-16k.ogg",
    ] {
        let file = ogg_chain("chain-start-long-mono-8k.ogg", second);
        let expected = device
            .load_sound(file.path())
            .err()
            .expect("whole sound rejects the later transition")
            .to_string();
        let music = device
            .load_music(file.path())
            .expect("load initial metadata");
        let handle = music
            .play()
            .expect("prefill precedes the format transition");
        assert!(handle.is_playing());
        let mut failure = None;
        assert!(wait_until(Duration::from_secs(3), || {
            failure = device.update().err();
            failure.is_some()
        }));
        let error = failure.expect("public update returned the refill error");
        assert!(matches!(error, AudioError::UnsupportedFormat(_)));
        assert_eq!(error.to_string(), expected);
        assert!(handle.cursor_frames() > 0);
        assert!(handle.is_stopped());
        handle.resume();
        assert!(handle.is_stopped());
        device.update().expect("failed stream was cleaned up");
    }
}
