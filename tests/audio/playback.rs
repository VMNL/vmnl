// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public playback-handle contracts. Hardware-dependent tests are skippable on headless CI.

use crate::common::{device_or_skip, lock_backend_tests, TempAudioFile};
use vmnl_audio::{AudioError, MusicPlayConfig, SoundPlayConfig};

#[test]
fn sound_handle_clones_control_the_same_playback_instance() {
    let _guard = lock_backend_tests();
    let Some(device) = device_or_skip(true) else {
        return;
    };

    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 44_100, 44_100 * 3, 4_000);
    let sound = device.load_sound(wav.path()).expect("load valid sound");

    let handle = sound
        .play_with_config(SoundPlayConfig { volume: 0.5, looping: true })
        .expect("play looping sound");
    let clone = handle.clone();

    assert!(handle.is_playing());
    assert!(clone.set_volume(f32::NAN).is_err());
    clone.set_volume(0.25).expect("valid volume");

    clone.pause();
    assert!(handle.is_paused(), "clone must pause the original playback");
    handle.resume();
    assert!(clone.is_playing(), "resume must be visible through every handle clone");

    clone.set_looping(false);
    handle.stop();
    assert!(clone.is_stopped());

    assert!(matches!(
        sound.play_with_config(SoundPlayConfig { volume: f32::INFINITY, looping: false }),
        Err(AudioError::InvalidState(_))
    ));
}

#[test]
fn music_handle_clones_share_pause_stop_volume_and_looping_state() {
    let _guard = lock_backend_tests();
    let Some(device) = device_or_skip(true) else {
        return;
    };

    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_wav(2, 44_100, 44_100 * 3, 2_000);
    let music = device.load_music(wav.path()).expect("load valid music");
    assert!(matches!(
        music.play_with_config(MusicPlayConfig { volume: f32::NAN, looping: true }),
        Err(AudioError::InvalidState(_))
    ));
    let handle = music
        .play_with_config(MusicPlayConfig { volume: 0.6, looping: true })
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
fn device_load_methods_report_unknown_extensions_and_missing_files() {
    let _guard = lock_backend_tests();
    let Some(device) = device_or_skip(false) else {
        return;
    };

    let unknown = TempAudioFile::new("unsupported-audio");
    unknown.write_bytes(b"unknown format");
    assert!(matches!(
        device.load_sound(unknown.path()),
        Err(AudioError::UnsupportedFormat(_))
    ));

    let missing = std::env::temp_dir().join(format!("vmnl-audio-no-file-{}.mp3", std::process::id()));
    let _ = std::fs::remove_file(&missing);
    assert!(device.load_music(&missing).is_err());
}
