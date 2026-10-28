// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Snapshot lifetime and heap behaviour without starting a native output device.

use crate::allocation_probe::{measure, Counts};
use crate::audio::decoder::test_support::decoded_audio;
use crate::audio::music::MusicStream;
use crate::audio::runtime::AudioRuntime;
use crate::audio::sound::SoundVoice;
use crate::audio::PlaybackState;
use crate::common::TempAudioFile;
use std::sync::Arc;

fn playbacks() -> (
    TempAudioFile,
    AudioRuntime,
    Arc<SoundVoice>,
    Arc<MusicStream>,
) {
    let runtime = AudioRuntime::new();
    let pcm = Arc::new(decoded_audio(2, 8_000, vec![0.25, -0.125]));
    let sound =
        Arc::new(SoundVoice::new(1, pcm, 1.0, true, PlaybackState::Playing).expect("valid sound"));
    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(2, 8_000, &[8_192, -4_096]);
    let music = Arc::new(
        MusicStream::new(
            file.path().to_path_buf(),
            1.0,
            true,
            PlaybackState::Playing,
            8_000,
        )
        .expect("valid music"),
    );
    music.pump().expect("prefill music");
    runtime
        .register_sound_voice(sound.clone())
        .expect("register sound");
    runtime
        .register_music_stream(music.clone())
        .expect("register music");
    (file, runtime, sound, music)
}

#[test]
fn first_mix_on_a_fresh_thread_performs_no_heap_operations() {
    let (_file, runtime, _sound, _music) = playbacks();
    std::thread::spawn(move || {
        let mut output = [0.0; 4];
        let ((), counts) = measure(|| runtime.mix_into(&mut output));
        assert_eq!(output, [0.5, -0.25, 0.5, -0.25]);
        assert_eq!(counts, Counts::default(), "first data callback");
    })
    .join()
    .expect("reader thread");
}

#[test]
fn releasing_replaced_snapshots_does_not_free_on_the_reader() {
    let (_file, runtime, sound, music) = playbacks();
    let sound_weak = Arc::downgrade(&sound);
    let music_weak = Arc::downgrade(&music);
    let voices = runtime
        .sound_voice_snapshot
        .read()
        .expect("exclusive sound reader");
    let streams = runtime
        .music_stream_snapshot
        .read()
        .expect("exclusive music reader");
    sound.stop();
    music.stop();
    drop(sound);
    drop(music);
    runtime
        .cleanup()
        .expect("publish empty snapshots while reader holds old views");
    assert!(sound_weak.upgrade().is_some());
    assert!(music_weak.upgrade().is_some());

    let ((), counts) = measure(|| {
        drop(voices);
        drop(streams);
    });
    assert_eq!(counts, Counts::default(), "release must defer reclamation");
    let mut output = [1.0; 4];
    let ((), counts) = measure(|| runtime.mix_into(&mut output));
    assert_eq!(counts, Counts::default(), "adopt replacement snapshots");
    assert_eq!(output, [0.0; 4]);
    let (result, reclamation) = measure(|| runtime.cleanup());
    result.expect("reclaim on control thread");
    assert_eq!(reclamation.allocations, 0);
    assert_eq!(reclamation.reallocations, 0);
    assert!(
        reclamation.deallocations > 0,
        "control side frees retired resources"
    );
    assert!(sound_weak.upgrade().is_none());
    assert!(music_weak.upgrade().is_none());
}

#[test]
fn heap_probe_observes_allocations_and_releases() {
    let ((), counts) = measure(|| {
        let buffer = std::hint::black_box(vec![0_u8; 4_096]);
        std::hint::black_box(buffer.as_ptr());
        drop(buffer);
    });
    assert!(counts.allocations > 0);
    assert!(counts.deallocations > 0);
}
