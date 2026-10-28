// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! State-transition regressions without a CPAL device.

use crate::audio::decoder::test_support::decoded_audio;
use crate::audio::music::MusicStream;
use crate::audio::runtime::AudioRuntime;
use crate::audio::sound::SoundVoice;
use crate::audio::PlaybackState;
use crate::common::TempAudioFile;
use std::sync::{Arc, Barrier};

const TRANSITIONS: [(PlaybackState, [PlaybackState; 3]); 3] = [
    (
        PlaybackState::Playing,
        [
            PlaybackState::Paused,
            PlaybackState::Playing,
            PlaybackState::Stopped,
        ],
    ),
    (
        PlaybackState::Paused,
        [
            PlaybackState::Paused,
            PlaybackState::Playing,
            PlaybackState::Stopped,
        ],
    ),
    (PlaybackState::Stopped, [PlaybackState::Stopped; 3]),
];

fn sound(state: PlaybackState) -> SoundVoice {
    let pcm = Arc::new(decoded_audio(2, 8_000, vec![0.5, -0.25, 0.25, -0.125]));
    SoundVoice::new(1, pcm, 1.0, false, state).expect("valid test voice")
}

fn music(looping: bool) -> (TempAudioFile, MusicStream) {
    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_samples(2, 8_000, &[16_384, -8_192, 8_192, -4_096]);
    let stream = MusicStream::new(
        wav.path().to_path_buf(),
        1.0,
        looping,
        PlaybackState::Playing,
        8_000,
    )
    .expect("valid test stream");
    // Keep the source file owned by the test so looping can reopen it.
    (wav, stream)
}

#[test]
fn sound_commands_follow_the_terminal_state_table() {
    let commands: [fn(&SoundVoice); 3] = [SoundVoice::pause, SoundVoice::resume, SoundVoice::stop];
    for (initial, expected) in TRANSITIONS {
        for (command, final_state) in commands.into_iter().zip(expected) {
            let voice = sound(initial);
            command(&voice);
            assert_eq!(voice.state(), final_state, "initial state: {initial:?}");
            command(&voice);
            assert_eq!(voice.state(), final_state, "command must be idempotent");
        }
    }
}

#[test]
fn music_commands_follow_the_terminal_state_table() {
    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_samples(2, 8_000, &[16_384, -8_192]);
    let commands: [fn(&MusicStream); 3] =
        [MusicStream::pause, MusicStream::resume, MusicStream::stop];
    for (initial, expected) in TRANSITIONS {
        for (command, final_state) in commands.into_iter().zip(expected) {
            let stream = MusicStream::new(wav.path().to_path_buf(), 1.0, false, initial, 8_000)
                .expect("valid test stream");
            command(&stream);
            assert_eq!(stream.state(), final_state, "initial state: {initial:?}");
            command(&stream);
            assert_eq!(stream.state(), final_state, "command must be idempotent");
        }
    }
}

#[test]
fn sound_stop_survives_commands_from_another_thread() {
    let voice = sound(PlaybackState::Playing);
    let stopped = Barrier::new(2);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            stopped.wait();
            for _ in 0..128 {
                voice.pause();
                voice.resume();
            }
        });
        voice.stop();
        stopped.wait();
    });
    assert!(voice.is_stopped());
}

#[test]
fn music_stop_survives_commands_from_another_thread() {
    let wav = TempAudioFile::new("wav");
    wav.write_pcm16_samples(2, 8_000, &[16_384, -8_192]);
    let stream = MusicStream::new(
        wav.path().to_path_buf(),
        1.0,
        false,
        PlaybackState::Playing,
        8_000,
    )
    .expect("valid test stream");
    let stopped = Barrier::new(2);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            stopped.wait();
            for _ in 0..128 {
                stream.pause();
                stream.resume();
            }
        });
        stream.stop();
        stopped.wait();
    });
    assert!(stream.is_stopped());
}

#[test]
fn sound_exact_buffer_completion_stops_and_freezes_the_cursor() {
    let voice = sound(PlaybackState::Playing);
    let mut output = [0.0; 4];
    voice.mix_into(&mut output, 1.0);
    assert_eq!(output, [0.5, -0.25, 0.25, -0.125]);
    assert_eq!(voice.cursor_frames(), 2);
    assert!(
        voice.is_stopped(),
        "the last source frame fills this buffer exactly"
    );

    voice.pause();
    voice.resume();
    voice.set_looping(true);
    let mut after_completion = [0.0; 4];
    voice.mix_into(&mut after_completion, 1.0);
    assert_eq!(after_completion, [0.0; 4]);
    assert_eq!(voice.cursor_frames(), 2);
    assert!(voice.is_stopped());
}

#[test]
fn looping_sound_cursor_and_pcm_continue_across_callbacks() {
    let pcm = Arc::new(decoded_audio(2, 8_000, vec![0.5, -0.25, 0.25, -0.125]));
    let voice =
        SoundVoice::new(1, pcm, 1.0, true, PlaybackState::Playing).expect("valid looping voice");
    let mut first = [0.0; 6];
    voice.mix_into(&mut first, 1.0);
    assert_eq!(first, [0.5, -0.25, 0.25, -0.125, 0.5, -0.25]);
    assert_eq!(voice.cursor_frames(), 3);

    let mut second = [0.0; 6];
    voice.mix_into(&mut second, 1.0);
    assert_eq!(second, [0.25, -0.125, 0.5, -0.25, 0.25, -0.125]);
    assert_eq!(voice.cursor_frames(), 6);
    assert_eq!(voice.state(), PlaybackState::Playing);
}

#[test]
fn disabling_sound_looping_finishes_the_current_repetition() {
    let voice = sound(PlaybackState::Playing);
    voice.set_looping(true);
    voice.mix_into(&mut [0.0; 6], 1.0);
    assert_eq!(voice.cursor_frames(), 3);

    voice.set_looping(false);
    let mut tail = [0.0; 6];
    voice.mix_into(&mut tail, 1.0);
    assert_eq!(tail, [0.25, -0.125, 0.0, 0.0, 0.0, 0.0]);
    assert_eq!(
        voice.cursor_frames(),
        4,
        "earlier loop iterations remain counted"
    );
    assert!(voice.is_stopped());
}

#[test]
fn disabling_sound_looping_at_a_boundary_does_not_start_another_repetition() {
    let voice = sound(PlaybackState::Playing);
    voice.set_looping(true);
    voice.mix_into(&mut [0.0; 8], 1.0);
    assert_eq!(voice.cursor_frames(), 4);

    voice.set_looping(false);
    let mut next = [0.0; 4];
    voice.mix_into(&mut next, 1.0);
    assert_eq!(next, [0.0; 4]);
    assert_eq!(voice.cursor_frames(), 4);
    assert!(voice.is_stopped());
}

#[test]
fn music_looping_can_be_enabled_after_decoder_eof_with_pcm_buffered() {
    let (_fixture, stream) = music(false);
    stream
        .pump()
        .expect("prefill short track through decoder EOF");
    stream.set_looping(true);
    stream.pump().expect("refill after enabling looping");
    let mut output = [0.0; 12];
    stream.mix_into(&mut output, 1.0);
    assert_eq!(
        output,
        [0.5, -0.25, 0.25, -0.125, 0.5, -0.25, 0.25, -0.125, 0.5, -0.25, 0.25, -0.125]
    );
    assert_eq!(stream.cursor_frames(), 6);
    assert_eq!(stream.state(), PlaybackState::Playing);
}

#[test]
fn music_enabled_loop_survives_buffer_drain_before_control_refill() {
    let (_fixture, stream) = music(false);
    stream
        .pump()
        .expect("prefill short track through decoder EOF");
    stream.set_looping(true);
    let mut first = [0.0; 4];
    stream.mix_into(&mut first, 1.0);
    assert_eq!(first, [0.5, -0.25, 0.25, -0.125]);
    assert_eq!(stream.state(), PlaybackState::Playing);
    assert_eq!(stream.cursor_frames(), 2);

    let mut underrun = [0.0; 4];
    stream.mix_into(&mut underrun, 1.0);
    assert_eq!(underrun, [0.0; 4]);
    assert_eq!(stream.cursor_frames(), 2);
    stream.pump().expect("restart the decoder after draining");
    let mut next = [0.0; 4];
    stream.mix_into(&mut next, 1.0);
    assert_eq!(next, first);
    assert_eq!(stream.cursor_frames(), 4);
    assert_eq!(stream.state(), PlaybackState::Playing);
}

#[test]
fn music_looping_cannot_restart_completed_playback() {
    let (_fixture, stream) = music(false);
    stream.pump().expect("prefill short track");
    let mut output = [0.0; 4];
    stream.mix_into(&mut output, 1.0);
    assert!(stream.is_stopped());

    stream.set_looping(true);
    stream.pause();
    stream.resume();
    stream.pump().expect("stopped stream performs no decoding");
    let mut after_completion = [0.0; 4];
    stream.mix_into(&mut after_completion, 1.0);
    assert_eq!(after_completion, [0.0; 4]);
    assert_eq!(stream.cursor_frames(), 2);
    assert!(stream.is_stopped());
}

#[test]
fn disabling_music_looping_preserves_buffered_pcm_and_eventually_stops() {
    let (_fixture, stream) = music(true);
    stream.pump().expect("prefill looping track");
    stream.set_looping(false);
    // The stereo ring holds half a second at 8 kHz, including prefetched loops.
    let mut output = vec![0.0; 8_000];
    stream.mix_into(&mut output, 1.0);
    assert_eq!(stream.cursor_frames(), 4_000);
    assert!(output
        .chunks_exact(4)
        .all(|chunk| chunk == [0.5, -0.25, 0.25, -0.125]));
    stream.pump().expect("observe EOF without reopening");
    assert!(stream.is_stopped());
}

#[test]
fn queued_pause_and_resume_cannot_resurrect_stopped_sound_or_music() {
    let runtime = AudioRuntime::new();
    let voice = Arc::new(sound(PlaybackState::Playing));
    let (_fixture, stream) = music(false);
    let stream = Arc::new(stream);
    runtime
        .register_sound_voice(voice.clone())
        .expect("register voice");
    runtime
        .register_music_stream(stream.clone())
        .expect("register stream");
    runtime.force_stop_all().expect("queue stop");
    runtime.force_pause_all().expect("queue pause");
    runtime.force_resume_all().expect("queue resume");
    runtime.tick().expect("apply commands and clean up");

    assert!(voice.is_stopped());
    assert!(stream.is_stopped());
    assert!(runtime
        .sound_voice_snapshot
        .read()
        .expect("exclusive sound reader")
        .is_empty());
    assert!(runtime
        .music_stream_snapshot
        .read()
        .expect("exclusive music reader")
        .is_empty());
}

#[test]
fn music_consumes_only_complete_stereo_frames() {
    let (_fixture, stream) = music(false);
    stream.pump().expect("prefill short track");
    let mut partial = [0.0; 1];
    stream.mix_into(&mut partial, 1.0);
    assert_eq!(partial, [0.0]);
    assert_eq!(stream.cursor_frames(), 0);

    let mut first = [0.0; 3];
    stream.mix_into(&mut first, 1.0);
    assert_eq!(first, [0.5, -0.25, 0.0]);
    assert_eq!(stream.cursor_frames(), 1);
    let mut last = [0.0; 2];
    stream.mix_into(&mut last, 1.0);
    assert_eq!(last, [0.25, -0.125]);
    assert_eq!(stream.cursor_frames(), 2);
    assert!(stream.is_stopped());
}
