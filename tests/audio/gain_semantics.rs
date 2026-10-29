// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! PCM consumption under mute, zero gain, and independent bus/handle pauses.

use crate::audio::bus::{AudioBus, BusKind};
use crate::audio::decoder::test_support::decoded_audio;
use crate::audio::music::MusicStream;
use crate::audio::runtime::AudioRuntime;
use crate::audio::sound::SoundVoice;
use crate::audio::PlaybackState;
use crate::common::TempAudioFile;
use std::sync::Arc;

const BUSES: [BusKind; 3] = [BusKind::Master, BusKind::Sfx, BusKind::Music];

struct Playback {
    runtime: AudioRuntime,
    sound: Arc<SoundVoice>,
    music: Arc<MusicStream>,
    // Looping music must be able to reopen its source for the entire scenario.
    _source: TempAudioFile,
}

impl Playback {
    fn new(looping: bool) -> Self {
        let runtime = AudioRuntime::new();
        let pcm = Arc::new(decoded_audio(2, 8_000, vec![0.5, -0.25, 0.25, -0.125]));
        let sound = Arc::new(
            SoundVoice::new(1, pcm, 1.0, looping, PlaybackState::Playing).expect("valid sound"),
        );
        let source = TempAudioFile::new("wav");
        source.write_pcm16_samples(2, 8_000, &[16_384, -8_192, 8_192, -4_096]);
        let music = Arc::new(
            MusicStream::new(
                source.path().to_path_buf(),
                1.0,
                looping,
                PlaybackState::Playing,
                8_000,
            )
            .expect("valid music"),
        );
        music.pump().expect("prefill PCM");
        runtime
            .register_sound_voice(sound.clone())
            .expect("register sound");
        runtime
            .register_music_stream(music.clone())
            .expect("register music");
        Self {
            runtime,
            sound,
            music,
            _source: source,
        }
    }

    fn bus(&self, kind: BusKind) -> AudioBus {
        match kind {
            BusKind::Master => self.runtime.master_bus().clone(),
            BusKind::Sfx => self.runtime.sfx_bus(),
            BusKind::Music => self.runtime.music_bus(),
        }
    }

    fn mix_frame(&self) -> [f32; 2] {
        let mut frame = [9.0; 2];
        self.runtime.mix_into(&mut frame);
        frame
    }

    fn assert_cursors(&self, sound: usize, music: usize) {
        assert_eq!(self.sound.cursor_frames(), sound, "sound cursor");
        assert_eq!(self.music.cursor_frames(), music, "music cursor");
    }
}

fn silence(bus: &AudioBus, muted: bool) {
    if muted {
        bus.mute();
        assert_eq!(bus.volume(), 1.0, "mute preserves the configured volume");
    } else {
        bus.set_volume(0.0).expect("valid silent gain");
    }
}

fn restore(bus: &AudioBus) {
    bus.unmute();
    bus.set_volume(1.0).expect("valid audible gain");
}

#[test]
fn silent_buses_consume_pcm_and_allow_natural_completion() {
    for kind in BUSES {
        for muted in [true, false] {
            let playback = Playback::new(false);
            silence(&playback.bus(kind), muted);
            let first = playback.mix_frame();
            assert_eq!(
                first,
                if kind == BusKind::Master {
                    [0.0; 2]
                } else {
                    [0.5, -0.25]
                }
            );
            playback.assert_cursors(1, 1);
            let last = playback.mix_frame();
            assert_eq!(
                last,
                if kind == BusKind::Master {
                    [0.0; 2]
                } else {
                    [0.25, -0.125]
                }
            );
            playback.assert_cursors(2, 2);
            assert!(playback.sound.is_stopped());
            assert!(playback.music.is_stopped());
            restore(&playback.bus(kind));
            assert_eq!(playback.mix_frame(), [0.0; 2]);
            playback.assert_cursors(2, 2);
        }
    }
}

#[test]
fn unmute_or_restoring_volume_continues_at_the_current_frame() {
    for kind in BUSES {
        for muted in [true, false] {
            let playback = Playback::new(false);
            let bus = playback.bus(kind);
            silence(&bus, muted);
            playback.mix_frame();
            restore(&bus);
            assert_eq!(playback.mix_frame(), [0.5, -0.25]);
            playback.assert_cursors(2, 2);
            assert!(playback.sound.is_stopped());
            assert!(playback.music.is_stopped());
        }
    }
}

#[test]
fn bus_pause_freezes_its_routes_even_with_zero_effective_gain() {
    for kind in BUSES {
        for muted in [true, false] {
            let playback = Playback::new(false);
            let bus = playback.bus(kind);
            silence(&bus, muted);
            bus.pause();
            let first = playback.mix_frame();
            assert_eq!(
                first,
                if kind == BusKind::Master {
                    [0.0; 2]
                } else {
                    [0.5, -0.25]
                }
            );
            playback.assert_cursors(
                usize::from(kind == BusKind::Music),
                usize::from(kind == BusKind::Sfx),
            );
            assert_eq!(playback.sound.state(), PlaybackState::Playing);
            assert_eq!(playback.music.state(), PlaybackState::Playing);

            bus.resume();
            assert_eq!(bus.is_muted(), muted, "resume preserves mute");
            assert_eq!(bus.gain(), 0.0, "resume preserves zero gain");
            let second = playback.mix_frame();
            assert_eq!(
                second,
                if kind == BusKind::Master {
                    [0.0; 2]
                } else {
                    [0.25, -0.125]
                }
            );
            playback.assert_cursors(
                if kind == BusKind::Music { 2 } else { 1 },
                if kind == BusKind::Sfx { 2 } else { 1 },
            );

            restore(&bus);
            let last = playback.mix_frame();
            assert_eq!(
                last,
                if kind == BusKind::Master {
                    [0.5, -0.25]
                } else {
                    [0.25, -0.125]
                }
            );
            playback.assert_cursors(2, 2);
            assert!(playback.sound.is_stopped());
            assert!(playback.music.is_stopped());
        }
    }
}

#[test]
fn bus_controls_preserve_individually_paused_or_stopped_handles() {
    for kind in BUSES {
        for state in [PlaybackState::Paused, PlaybackState::Stopped] {
            let playback = Playback::new(false);
            if state == PlaybackState::Paused {
                playback.sound.pause();
                playback.music.pause();
            } else {
                playback.sound.stop();
                playback.music.stop();
            }
            let bus = playback.bus(kind);
            bus.pause();
            silence(&bus, true);
            bus.set_volume(0.0).expect("valid silent gain");
            assert_eq!(playback.mix_frame(), [0.0; 2]);
            playback.assert_cursors(0, 0);
            bus.resume();
            restore(&bus);
            assert_eq!(playback.mix_frame(), [0.0; 2]);
            playback.assert_cursors(0, 0);
            assert_eq!(playback.sound.state(), state);
            assert_eq!(playback.music.state(), state);
        }
    }
}

#[test]
fn zero_handle_volumes_consume_pcm_until_eof() {
    let playback = Playback::new(false);
    playback.sound.set_volume(0.0).expect("valid sound volume");
    playback.music.set_volume(0.0).expect("valid music volume");
    assert_eq!(playback.mix_frame(), [0.0; 2]);
    playback.assert_cursors(1, 1);
    assert_eq!(playback.mix_frame(), [0.0; 2]);
    playback.assert_cursors(2, 2);
    assert!(playback.sound.is_stopped());
    assert!(playback.music.is_stopped());
}

#[test]
fn silent_looping_playbacks_keep_cumulative_progress() {
    for kind in BUSES {
        for muted in [true, false] {
            let playback = Playback::new(true);
            let bus = playback.bus(kind);
            silence(&bus, muted);
            let mut output = [9.0; 6];
            playback.runtime.mix_into(&mut output);
            assert_eq!(
                output,
                if kind == BusKind::Master {
                    [0.0; 6]
                } else {
                    [0.5, -0.25, 0.25, -0.125, 0.5, -0.25]
                }
            );
            playback.assert_cursors(3, 3);
            assert_eq!(playback.sound.state(), PlaybackState::Playing);
            assert_eq!(playback.music.state(), PlaybackState::Playing);
            restore(&bus);
            assert_eq!(playback.mix_frame(), [0.5, -0.25]);
            playback.assert_cursors(4, 4);
        }
    }
}

#[test]
fn resuming_master_preserves_a_category_pause() {
    let playback = Playback::new(false);
    playback.runtime.master_bus().pause();
    playback.runtime.sfx_bus().pause();
    assert_eq!(playback.mix_frame(), [0.0; 2]);
    playback.assert_cursors(0, 0);

    playback.runtime.master_bus().resume();
    assert_eq!(playback.mix_frame(), [0.5, -0.25]);
    playback.assert_cursors(0, 1);
    assert!(playback.runtime.sfx_bus().is_paused());
    playback.runtime.sfx_bus().resume();
    assert_eq!(playback.mix_frame(), [0.75, -0.375]);
    playback.assert_cursors(1, 2);
}

#[test]
fn muted_music_underrun_does_not_count_inserted_silence() {
    let playback = Playback::new(true);
    playback.runtime.master_bus().mute();
    // Music prefill holds 4,000 frames at 8 kHz. Request two frames beyond it.
    let mut output = vec![9.0; 8_004];
    playback.runtime.mix_into(&mut output);
    assert!(output.iter().all(|sample| *sample == 0.0));
    playback.assert_cursors(4_002, 4_000);
    assert_eq!(playback.music.state(), PlaybackState::Playing);
    playback.music.pump().expect("refill after underrun");
    assert_eq!(playback.mix_frame(), [0.0; 2]);
    playback.assert_cursors(4_003, 4_001);
}
