// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public configuration, validation, and error contracts that do not require audio hardware.

use std::io;
use vmnl_audio::{
    AudioConfig, AudioDevice, AudioDeviceBuilder, AudioError, ControlMode, MusicPlayConfig,
    PlaybackState, SoundPlayConfig,
};

#[test]
fn control_is_manual_by_default_and_automatic_only_when_requested() {
    assert_eq!(
        AudioDeviceBuilder::default().control_mode(),
        ControlMode::Manual
    );
    assert_eq!(AudioDevice::builder().control_mode(), ControlMode::Manual);
    assert_eq!(
        AudioDevice::builder().thread(true).control_mode(),
        ControlMode::Automatic
    );
    assert_eq!(
        AudioDevice::builder()
            .thread(true)
            .thread(false)
            .control_mode(),
        ControlMode::Manual
    );
}

#[test]
fn audio_config_defaults_to_stereo_at_44100_hz_and_unity_gain() {
    let config = AudioConfig::default();

    assert_eq!(config.master_volume(), 1.0);
    assert_eq!(config.sample_rate(), 44_100);
    assert_eq!(config.channels(), 2);
}

#[test]
fn audio_config_setters_keep_valid_values_and_reject_invalid_updates() {
    let mut config = AudioConfig::default();

    config
        .set_master_volume(0.25)
        .expect("intermediate gain is valid");
    assert_eq!(config.master_volume(), 0.25);

    for invalid in [-0.01, 1.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            config.set_master_volume(invalid),
            Err(AudioError::InvalidState(_))
        ));
        assert_eq!(
            config.master_volume(),
            0.25,
            "invalid gain must not overwrite state"
        );
    }

    config.set_master_volume(0.0).expect("mute gain is valid");
    assert_eq!(config.master_volume(), 0.0);
    config.set_master_volume(1.0).expect("unity gain is valid");
    assert_eq!(config.master_volume(), 1.0);

    config
        .set_sample_rate(48_000)
        .expect("positive sample rate is valid");
    assert_eq!(config.sample_rate(), 48_000);
    assert!(matches!(
        config.set_sample_rate(0),
        Err(AudioError::InvalidState(_))
    ));
    assert_eq!(config.sample_rate(), 48_000);

    config.set_channels(2).expect("stereo output is supported");
    for channels in [0, 1, 4, 6] {
        assert!(matches!(
            config.set_channels(channels),
            Err(AudioError::UnsupportedFormat(_))
        ));
        assert_eq!(config.channels(), 2);
    }
}

#[test]
fn builder_rejects_invalid_configuration_before_backend_initialization() {
    assert!(matches!(
        AudioDevice::builder().master_volume(f32::NAN).build(),
        Err(AudioError::InvalidState(_))
    ));
    assert!(matches!(
        AudioDevice::builder().sample_rate(0).build(),
        Err(AudioError::InvalidState(_))
    ));
    assert!(matches!(
        AudioDevice::builder().channels(1).build(),
        Err(AudioError::UnsupportedFormat(_))
    ));
}

#[test]
fn playback_configs_have_stable_defaults_and_keep_custom_values() {
    let sound_default = SoundPlayConfig::default();
    assert_eq!(sound_default.volume, 1.0);
    assert!(!sound_default.looping);

    let music_default = MusicPlayConfig::default();
    assert_eq!(music_default.volume, 1.0);
    assert!(!music_default.looping);

    let sound_custom = SoundPlayConfig {
        volume: 0.3,
        looping: true,
    };
    assert_eq!(sound_custom.volume, 0.3);
    assert!(sound_custom.looping);

    let music_custom = MusicPlayConfig {
        volume: 0.7,
        looping: true,
    };
    assert_eq!(music_custom.volume, 0.7);
    assert!(music_custom.looping);
}

#[test]
fn playback_state_variants_are_distinct_without_relying_on_numeric_discriminants() {
    assert_ne!(PlaybackState::Playing, PlaybackState::Paused);
    assert_ne!(PlaybackState::Paused, PlaybackState::Stopped);
    assert_ne!(PlaybackState::Playing, PlaybackState::Stopped);
}

#[test]
fn public_audio_errors_have_useful_display_messages_and_io_conversion() {
    let io_error: AudioError = io::Error::new(io::ErrorKind::NotFound, "fixture missing").into();
    assert!(matches!(io_error, AudioError::Io(_)));
    assert!(io_error.to_string().contains("fixture missing"));

    assert!(AudioError::InvalidState("bad gain".to_owned())
        .to_string()
        .contains("bad gain"));
    assert!(AudioError::UnsupportedFormat("channels=6".to_owned())
        .to_string()
        .contains("channels=6"));
    assert!(AudioError::DecoderFailed("corrupt WAV".to_owned())
        .to_string()
        .contains("corrupt WAV"));
}
