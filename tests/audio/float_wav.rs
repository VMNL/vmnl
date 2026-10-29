// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Extreme finite WAV playback through cached PCM, streaming refills and bus mixing.

use crate::audio::bus::BusKind;
use crate::audio::decoder::AudioDecoder;
use crate::audio::error::AudioError;
use crate::audio::music::MusicStream;
use crate::audio::runtime::AudioRuntime;
use crate::audio::sound::SoundVoice;
use crate::audio::PlaybackState;
use crate::common::TempAudioFile;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GainMode {
    Active,
    MasterMute,
    MasterZero,
    RouteMute,
    RouteZero,
    HandleZero,
}

const GAIN_MODES: [GainMode; 6] = [
    GainMode::Active,
    GainMode::MasterMute,
    GainMode::MasterZero,
    GainMode::RouteMute,
    GainMode::RouteZero,
    GainMode::HandleZero,
];

fn configure_gain(runtime: &AudioRuntime, route: BusKind, mode: GainMode) {
    match mode {
        GainMode::MasterMute => runtime.master_bus().mute(),
        GainMode::MasterZero => runtime.master_bus().set_volume(0.0).expect("zero gain"),
        GainMode::RouteMute => runtime.bus(route).mute(),
        GainMode::RouteZero => runtime.bus(route).set_volume(0.0).expect("zero gain"),
        GainMode::Active | GainMode::HandleZero => {}
    }
}

fn source(prefix_frames: usize) -> TempAudioFile {
    let file = TempAudioFile::new("wav");
    let mut samples = [0.25, -0.125].repeat(prefix_frames);
    for value in [f32::MAX, -f32::MAX, f32::MAX, -f32::MAX, f32::MAX] {
        samples.extend_from_slice(&[value, -value]);
    }
    file.write_float_wav(2, 8_000, &samples);
    file
}

fn assert_mixed(actual: &[f32], pcm: &[f32], mode: GainMode) {
    assert!(actual.iter().all(|sample| sample.is_finite()), "{mode:?}");
    for (actual, sample) in actual.iter().zip(pcm) {
        let expected = if mode == GainMode::Active {
            sample.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        assert_eq!(*actual, expected, "{mode:?}");
    }
}

#[test]
fn extreme_float_sound_mixes_finitely_and_silence_still_consumes_pcm() {
    for prefix_frames in [0, 8_192] {
        let file = source(prefix_frames);
        let decoded = Arc::new(
            AudioDecoder::decode_file(file.path())
                .expect("decode extreme finite WAV")
                .resample_to_stereo(16_000)
                .expect("resample extreme sound"),
        );
        assert_eq!(decoded.frame_count(), (prefix_frames + 5) * 2);
        for mode in GAIN_MODES {
            let runtime = AudioRuntime::new();
            let sound = Arc::new(
                SoundVoice::new(1, decoded.clone(), 1.0, false, PlaybackState::Playing)
                    .expect("create sound"),
            );
            configure_gain(&runtime, BusKind::Sfx, mode);
            if mode == GainMode::HandleZero {
                sound.set_volume(0.0).expect("silence sound handle");
            }
            runtime
                .register_sound_voice(sound.clone())
                .expect("publish sound");
            for (block, pcm) in decoded.samples().chunks(514).enumerate() {
                let mut output = vec![99.0; pcm.len()];
                runtime.mix_into(&mut output);
                assert_mixed(&output, pcm, mode);
                assert_eq!(sound.cursor_frames(), (block * 257 + pcm.len() / 2));
            }
            assert!(sound.is_stopped());
            let mut tail = [99.0; 4];
            runtime.mix_into(&mut tail);
            assert_eq!(tail, [0.0; 4]);
            assert_eq!(sound.cursor_frames(), decoded.frame_count());
        }
    }
}

#[test]
fn extreme_float_music_refills_and_mixes_finitely_even_when_silent() {
    for prefix_frames in [0, 8_192] {
        let file = source(prefix_frames);
        let decoded = AudioDecoder::decode_file(file.path())
            .expect("decode reference PCM")
            .resample_to_stereo(16_000)
            .expect("resample reference PCM");
        for mode in GAIN_MODES {
            let runtime = AudioRuntime::new();
            let music = Arc::new(
                MusicStream::new(
                    file.path().to_path_buf(),
                    1.0,
                    false,
                    PlaybackState::Playing,
                    16_000,
                )
                .expect("open extreme float music"),
            );
            music.pump().expect("prefill music");
            configure_gain(&runtime, BusKind::Music, mode);
            if mode == GainMode::HandleZero {
                music.set_volume(0.0).expect("silence music handle");
            }
            runtime
                .register_music_stream(music.clone())
                .expect("publish music");
            for (block, pcm) in decoded.samples().chunks(514).enumerate() {
                let mut output = vec![99.0; pcm.len()];
                runtime.mix_into(&mut output);
                assert_mixed(&output, pcm, mode);
                assert_eq!(music.cursor_frames(), block * 257 + pcm.len() / 2);
                runtime.tick().expect("refill and clean up music");
            }
            assert!(music.is_stopped());
            let mut tail = [99.0; 4];
            runtime.mix_into(&mut tail);
            assert_eq!(tail, [0.0; 4]);
            assert_eq!(music.cursor_frames(), decoded.frame_count());
        }
    }
}

#[test]
fn non_finite_float_music_refill_stops_without_publishing_bad_pcm() {
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let file = TempAudioFile::new("wav");
        let mut samples = [0.25, -0.125].repeat(8_192);
        samples.extend_from_slice(&[invalid, 0.0]);
        file.write_float_wav(2, 8_000, &samples);
        let runtime = AudioRuntime::new();
        let music = Arc::new(
            MusicStream::new(
                file.path().to_path_buf(),
                1.0,
                false,
                PlaybackState::Playing,
                16_000,
            )
            .expect("open initial metadata"),
        );
        music.pump().expect("invalid PCM lies after prefill");
        runtime.master_bus().mute();
        runtime
            .register_music_stream(music.clone())
            .expect("publish music");
        let mut failed = false;
        for _ in 0..128 {
            let mut output = [99.0; 514];
            let before = music.cursor_frames();
            runtime.mix_into(&mut output);
            assert_eq!(output, [0.0; 514]);
            assert!(music.cursor_frames() > before, "valid prefix must advance");
            if let Err(error) = runtime.tick() {
                assert!(matches!(error, AudioError::DecoderFailed(_)));
                failed = true;
                break;
            }
        }
        assert!(failed, "later invalid PCM must fail refill");
        assert!(music.is_stopped());
        assert!(music.cursor_frames() <= 8_192 * 2);
        let mut output = [99.0; 514];
        runtime.mix_into(&mut output);
        assert_eq!(output, [0.0; 514]);
        runtime.tick().expect("failed music was cleaned up");
    }
}
