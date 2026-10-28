// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Deterministic runtime, snapshot, voice-limit and mixer tests using the production sources.

use crate::common::TempAudioFile;
use crate::internal_harness::audio::bus::AudioBus;
use crate::internal_harness::audio::decoder::AudioDecoder;
use crate::internal_harness::audio::error::AudioError;
use crate::internal_harness::audio::music::MusicStream;
use crate::internal_harness::audio::runtime::AudioRuntime;
use crate::internal_harness::audio::sound::SoundVoice;
use crate::internal_harness::audio::PlaybackState;
use std::sync::Arc;

fn decoded_stereo(samples: &[i16]) -> Arc<crate::internal_harness::audio::decoder::DecodedAudio> {
    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(2, 8_000, samples);
    Arc::new(AudioDecoder::decode_file(file.path()).expect("decode controlled stereo samples"))
}

fn voice(
    id: u64,
    decoded: Arc<crate::internal_harness::audio::decoder::DecodedAudio>,
    looping: bool,
) -> Arc<SoundVoice> {
    Arc::new(
        SoundVoice::new(id, decoded, 1.0, looping, PlaybackState::Playing).expect("valid voice"),
    )
}

#[test]
fn bus_clones_share_state_without_opening_an_audio_device() {
    let bus = AudioBus::new();
    let clone = bus.clone();

    bus.set_volume(0.35).expect("valid gain");
    assert_eq!(clone.volume(), 0.35);
    assert_eq!(clone.gain(), 0.35);

    clone.mute();
    assert!(bus.is_muted());
    assert_eq!(bus.volume(), 0.35);
    assert_eq!(bus.gain(), 0.0);

    bus.unmute();
    clone.pause();
    assert!(bus.is_paused());
    assert_eq!(clone.gain(), 0.0);

    bus.resume();
    assert!(!clone.is_paused());
    assert_eq!(bus.gain(), 0.35);
}

#[test]
fn mixer_clears_output_and_returns_silence_when_nothing_is_playing() {
    let runtime = AudioRuntime::new();
    let mut output = [0.75_f32; 8];
    runtime.mix_into(&mut output);
    assert_eq!(output, [0.0; 8]);
}

#[test]
fn mixer_multiplies_master_and_sfx_gains_and_preserves_stereo_order() {
    let runtime = AudioRuntime::new();
    runtime
        .master_bus()
        .set_volume(0.5)
        .expect("valid master gain");
    runtime.sfx_bus().set_volume(0.25).expect("valid SFX gain");
    runtime
        .register_sound_voice(voice(
            1,
            decoded_stereo(&[16_384, -8_192, 8_192, -4_096]),
            false,
        ))
        .expect("register voice");

    let mut output = [8.0_f32; 8];
    runtime.mix_into(&mut output);
    assert_eq!(
        output,
        [0.0625, -0.03125, 0.03125, -0.015_625, 0.0, 0.0, 0.0, 0.0]
    );
}

#[test]
fn mixer_combines_music_and_sfx_using_their_own_bus_gains() {
    let runtime = AudioRuntime::new();
    runtime
        .master_bus()
        .set_volume(0.5)
        .expect("valid master gain");
    runtime.sfx_bus().set_volume(0.5).expect("valid SFX gain");
    runtime
        .music_bus()
        .set_volume(0.25)
        .expect("valid music gain");

    runtime
        .register_sound_voice(voice(
            1,
            decoded_stereo(&[16_384, -8_192, 8_192, -4_096]),
            false,
        ))
        .expect("register SFX voice");

    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(2, 8_000, &[16_384, -8_192, 8_192, -4_096]);
    let music = Arc::new(
        MusicStream::new(
            file.path().to_path_buf(),
            1.0,
            false,
            PlaybackState::Playing,
            8_000,
        )
        .expect("create test music stream"),
    );
    music.pump().expect("prefill music ring");
    runtime
        .register_music_stream(music)
        .expect("register music stream");

    let mut output = [0.0_f32; 4];
    runtime.mix_into(&mut output);
    assert_eq!(output, [0.1875, -0.09375, 0.09375, -0.046_875]);
}

#[test]
fn muted_or_paused_bus_silences_its_category_without_changing_volume() {
    let runtime = AudioRuntime::new();
    runtime.sfx_bus().set_volume(0.7).expect("valid SFX gain");
    runtime.sfx_bus().mute();
    runtime
        .register_sound_voice(voice(1, decoded_stereo(&[16_384, -8_192]), true))
        .expect("register voice");

    let mut muted_output = [1.0_f32; 2];
    runtime.mix_into(&mut muted_output);
    assert_eq!(muted_output, [0.0, 0.0]);
    assert_eq!(runtime.sfx_bus().volume(), 0.7);

    runtime.sfx_bus().unmute();
    runtime.sfx_bus().pause();
    let mut paused_output = [1.0_f32; 2];
    runtime.mix_into(&mut paused_output);
    assert_eq!(paused_output, [0.0, 0.0]);
}

#[test]
fn mixer_sums_multiple_voices_and_clamps_output_to_unit_range() {
    let runtime = AudioRuntime::new();
    let decoded = decoded_stereo(&[i16::MAX, i16::MIN]);
    runtime
        .register_sound_voice(voice(1, decoded.clone(), true))
        .expect("register first voice");
    runtime
        .register_sound_voice(voice(2, decoded, true))
        .expect("register second voice");

    let mut output = [0.0_f32; 2];
    runtime.mix_into(&mut output);
    assert_eq!(output, [1.0, -1.0]);
}

#[test]
fn voice_limit_stops_oldest_voice_and_publishes_consistent_snapshot() {
    let runtime = AudioRuntime::new();
    runtime.set_max_sound_voices(2).expect("set limit to two");
    let decoded = decoded_stereo(&[1_000, -1_000]);
    let first = voice(runtime.next_voice_id(), decoded.clone(), true);
    let second = voice(runtime.next_voice_id(), decoded, true);
    runtime
        .register_sound_voice(first.clone())
        .expect("register first");
    runtime
        .register_sound_voice(second.clone())
        .expect("register second");
    assert_eq!(
        runtime
            .sound_voice_snapshot
            .read()
            .expect("exclusive sound reader")
            .len(),
        2
    );

    runtime
        .set_max_sound_voices(1)
        .expect("reduce active voice limit");
    assert!(first.is_stopped());
    assert_eq!(second.state(), PlaybackState::Playing);
    let snapshot = runtime
        .sound_voice_snapshot
        .read()
        .expect("exclusive sound reader");
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].id(), second.id());

    runtime
        .set_max_sound_voices(0)
        .expect("zero is clamped to the minimum");
    assert_eq!(runtime.max_sound_voices(), 1);
}

#[test]
fn runtime_commands_are_applied_in_order_and_cleanup_updates_snapshots() {
    let runtime = AudioRuntime::new();
    let current = voice(
        1,
        decoded_stereo(&[1_000, -1_000, 1_000, -1_000, 1_000, -1_000, 1_000, -1_000]),
        true,
    );
    runtime
        .register_sound_voice(current.clone())
        .expect("register voice");

    runtime.force_pause_all().expect("queue pause");
    runtime.tick().expect("apply pause command");
    assert_eq!(current.state(), PlaybackState::Paused);

    runtime.force_resume_all().expect("queue resume");
    runtime.tick().expect("apply resume command");
    assert_eq!(current.state(), PlaybackState::Playing);

    runtime.force_stop_all().expect("queue stop");
    runtime.tick().expect("apply stop command and clean up");
    assert!(current.is_stopped());
    assert!(runtime
        .sound_voice_snapshot
        .read()
        .expect("exclusive sound reader")
        .is_empty());
}

#[test]
fn repeated_loads_reuse_the_runtime_audio_cache() {
    let runtime = AudioRuntime::new();
    let file = TempAudioFile::new("wav");
    file.write_pcm16_wav(2, 8_000, 16, 1_000);

    let first = runtime
        .get_or_decode_audio(file.path())
        .expect("first decode");
    let second = runtime
        .get_or_decode_audio(file.path())
        .expect("cached decode");
    assert!(
        Arc::ptr_eq(&first, &second),
        "same path should reuse cached decoded data"
    );
}

#[test]
fn finished_music_is_removed_only_after_the_ring_is_drained() {
    let runtime = AudioRuntime::new();
    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(2, 8_000, &[16_384, -8_192, 8_192, -4_096]);
    let stream = Arc::new(
        MusicStream::new(
            file.path().to_path_buf(),
            1.0,
            false,
            PlaybackState::Playing,
            8_000,
        )
        .expect("create stream"),
    );
    stream.pump().expect("pre-fill stream");
    runtime
        .register_music_stream(stream.clone())
        .expect("register stream");
    assert_eq!(
        runtime
            .music_stream_snapshot
            .read()
            .expect("exclusive music reader")
            .len(),
        1
    );

    let mut first_frame = [0.0_f32; 2];
    runtime.mix_into(&mut first_frame);
    assert_eq!(first_frame, [0.5, -0.25]);
    runtime
        .cleanup()
        .expect("cleanup must retain buffered stream");
    assert_eq!(
        runtime
            .music_stream_snapshot
            .read()
            .expect("exclusive music reader")
            .len(),
        1
    );

    let mut second_frame = [0.0_f32; 2];
    runtime.mix_into(&mut second_frame);
    assert_eq!(second_frame, [0.25, -0.125]);
    assert!(stream.is_stopped());
    runtime.cleanup().expect("remove drained stream");
    assert!(runtime
        .music_stream_snapshot
        .read()
        .expect("exclusive music reader")
        .is_empty());
}

#[test]
fn runtime_propagates_invalid_gain_errors_instead_of_silently_accepting_them() {
    let runtime = AudioRuntime::new();
    assert!(matches!(
        runtime.master_bus().set_volume(f32::NAN),
        Err(AudioError::InvalidState(_))
    ));
}
