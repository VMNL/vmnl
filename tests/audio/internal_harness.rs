// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Test-only source inclusion for private audio implementation contracts.
//!
//! The production types remain private. Including the same source files in this
//! integration-test target allows this suite to test private ring-buffer,
//! decoder and mixer invariants without widening the public API. These modules
//! are separate test copies; they are not linked to the corresponding types in
//! the `vmnl_audio` library.

#![allow(dead_code, unused_imports)]

// Load this module normally so its inner Rustdoc remains valid.
#[path = "../../crates/vmnl_audio/src/audio/stream_decoder.rs"]
pub(crate) mod stream_decoder;

#[path = "../../crates/vmnl_audio/src/audio/snapshot.rs"]
pub(crate) mod snapshot;

#[path = "../../crates/vmnl_audio/src/audio/output_config.rs"]
pub(crate) mod output_config;

pub(crate) mod audio {
    pub(crate) mod error {
        include!("../../crates/vmnl_audio/src/audio/error.rs");
    }

    pub(crate) mod bus {
        include!("../../crates/vmnl_audio/src/audio/bus.rs");
    }

    pub(crate) use super::output_config;
    pub(crate) use super::snapshot;
    pub(crate) use super::stream_decoder;

    pub(crate) mod decoder {
        include!("../../crates/vmnl_audio/src/audio/decoder.rs");

        #[cfg(test)]
        pub(crate) mod test_support {
            use super::DecodedAudio;

            pub(crate) fn decoded_audio(
                channels: u32,
                sample_rate: u32,
                samples: Vec<f32>,
            ) -> DecodedAudio {
                DecodedAudio {
                    channels,
                    sample_rate,
                    samples,
                }
            }
        }

        #[cfg(test)]
        mod decoded_audio_tests {
            use super::{test_support::decoded_audio, DecodedAudio};
            use crate::internal_harness::audio::error::AudioError;

            #[test]
            fn frame_count_and_empty_contract_handle_zero_channels_safely() {
                let stereo = decoded_audio(2, 44_100, vec![0.0; 10]);
                assert_eq!(stereo.frame_count(), 5);
                assert!(!stereo.is_empty());

                let empty = decoded_audio(2, 44_100, Vec::new());
                assert_eq!(empty.frame_count(), 0);
                assert!(empty.is_empty());
                let converted_empty = empty
                    .resample_to_stereo(48_000)
                    .expect("resample empty input");
                assert_eq!(converted_empty.frame_count(), 0);
                assert!(converted_empty.is_empty());

                let malformed = DecodedAudio {
                    channels: 0,
                    sample_rate: 44_100,
                    samples: vec![1.0],
                };
                assert_eq!(malformed.frame_count(), 0);
            }

            #[test]
            fn resampling_converts_mono_to_stereo_and_calculates_output_frames() {
                let mono = decoded_audio(1, 2, vec![0.0, 1.0]);
                let upsampled = mono.resample_to_stereo(4).expect("upsample mono data");
                assert_eq!(upsampled.frame_count(), 4);
                assert_eq!(upsampled.channels(), 2);
                assert_eq!(
                    upsampled.samples(),
                    &[0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0]
                );

                let downsampled = mono.resample_to_stereo(1).expect("downsample mono data");
                assert_eq!(downsampled.frame_count(), 1);
                assert_eq!(downsampled.samples(), &[0.0, 0.0]);
            }

            #[test]
            fn resampling_at_the_same_stereo_sample_rate_preserves_samples() {
                let stereo = decoded_audio(2, 48_000, vec![0.25, -0.5, 0.75, -1.0]);
                let converted = stereo
                    .resample_to_stereo(48_000)
                    .expect("same rate conversion");
                assert_eq!(converted.frame_count(), 2);
                assert_eq!(converted.samples(), stereo.samples());
            }

            #[test]
            fn invalid_target_sample_rate_is_rejected_before_arithmetic() {
                let source = decoded_audio(1, 44_100, vec![0.5]);
                assert!(matches!(
                    source.resample_to_stereo(0),
                    Err(AudioError::InvalidState(_))
                ));
            }

            #[test]
            fn zero_source_sample_rate_is_rejected_instead_of_producing_misleading_audio() {
                // Private-seam defense, not a state constructible by library clients.
                for channels in [1, 2] {
                    for samples in [Vec::new(), vec![0.5, -0.5]] {
                        let source = decoded_audio(channels, 0, samples);
                        assert!(matches!(
                            source.resample_to_stereo(48_000),
                            Err(AudioError::DecoderFailed(_))
                        ));
                    }
                }
            }

            #[test]
            fn unsupported_source_channel_count_is_rejected() {
                let surround = decoded_audio(6, 48_000, vec![0.0; 12]);
                assert!(matches!(
                    surround.resample_to_stereo(48_000),
                    Err(AudioError::UnsupportedFormat(_))
                ));
            }

            #[test]
            fn incomplete_interleaved_frame_is_rejected() {
                for samples in [vec![0.25], vec![0.25, -0.5, 0.75]] {
                    let malformed = decoded_audio(2, 48_000, samples);
                    for rate in [44_100, 48_000] {
                        assert!(matches!(
                            malformed.resample_to_stereo(rate),
                            Err(AudioError::DecoderFailed(_))
                        ));
                    }
                }
            }
        }
    }

    pub(crate) mod sound {
        pub(crate) use voice::SoundVoice;

        pub(crate) mod voice {
            include!("../../crates/vmnl_audio/src/audio/sound/voice.rs");

            #[cfg(test)]
            mod voice_tests {
                use super::{PlaybackState, SoundVoice};
                use crate::common::TempAudioFile;
                use crate::internal_harness::audio::decoder::AudioDecoder;
                use std::sync::Arc;

                #[test]
                fn sound_voice_mixes_exact_stereo_samples_and_advances_by_consumed_frames() {
                    let wav = TempAudioFile::new("wav");
                    wav.write_pcm16_samples(2, 8_000, &[16_384, -8_192, 8_192, -4_096]);
                    let decoded =
                        Arc::new(AudioDecoder::decode_file(wav.path()).expect("decode fixture"));
                    let voice = SoundVoice::new(1, decoded, 0.5, false, PlaybackState::Playing)
                        .expect("valid voice");
                    let mut output = [0.0_f32; 8];

                    voice.mix_into(&mut output, 0.5);
                    assert_eq!(
                        output,
                        [0.125, -0.0625, 0.0625, -0.03125, 0.0, 0.0, 0.0, 0.0]
                    );
                    assert!(voice.is_stopped());
                    assert_eq!(
                        voice.cursor_frames(),
                        2,
                        "cursor should count frames actually consumed"
                    );
                }

                #[test]
                fn looping_sound_voice_repeats_source_frames_without_stopping() {
                    let wav = TempAudioFile::new("wav");
                    wav.write_pcm16_samples(2, 8_000, &[16_384, -8_192, 8_192, -4_096]);
                    let decoded =
                        Arc::new(AudioDecoder::decode_file(wav.path()).expect("decode fixture"));
                    let voice = SoundVoice::new(3, decoded, 0.5, true, PlaybackState::Playing)
                        .expect("valid looping voice");
                    let mut output = [0.0_f32; 8];

                    voice.mix_into(&mut output, 0.5);
                    assert_eq!(
                        output,
                        [0.125, -0.0625, 0.0625, -0.03125, 0.125, -0.0625, 0.0625, -0.03125]
                    );
                    assert_eq!(voice.state(), PlaybackState::Playing);
                    assert_eq!(voice.cursor_frames(), 4);
                }

                #[test]
                fn sound_voice_pause_resume_stop_and_invalid_volume_behave_consistently() {
                    let wav = TempAudioFile::new("wav");
                    wav.write_pcm16_wav(2, 8_000, 32, 1_000);
                    let decoded =
                        Arc::new(AudioDecoder::decode_file(wav.path()).expect("decode fixture"));
                    let voice = SoundVoice::new(2, decoded, 1.0, true, PlaybackState::Playing)
                        .expect("valid looping voice");

                    assert!(voice.set_volume(f32::NAN).is_err());
                    assert_eq!(voice.state(), PlaybackState::Playing);
                    let mut empty_output = [];
                    voice.mix_into(&mut empty_output, 1.0);
                    assert_eq!(
                        voice.state(),
                        PlaybackState::Playing,
                        "empty output must not stop a voice"
                    );
                    assert_eq!(voice.cursor_frames(), 0);
                    voice.pause();
                    assert_eq!(voice.state(), PlaybackState::Paused);
                    let cursor = voice.cursor_frames();
                    let mut output = [0.0_f32; 16];
                    voice.mix_into(&mut output, 1.0);
                    assert_eq!(
                        voice.cursor_frames(),
                        cursor,
                        "paused voice must not advance"
                    );

                    voice.resume();
                    assert_eq!(voice.state(), PlaybackState::Playing);
                    voice.mix_into(&mut output, 1.0);
                    assert!(voice.cursor_frames() > cursor);
                    voice.stop();
                    assert!(voice.is_stopped());
                }
            }
        }
    }

    pub(crate) use bus::BusKind;
    pub(crate) use sound::voice::PlaybackState;

    pub(crate) mod music {
        pub(crate) use stream::MusicStream;

        pub(crate) mod stream {
            include!("../../crates/vmnl_audio/src/audio/music/stream.rs");

            #[cfg(test)]
            mod sample_ring_tests {
                use super::SampleRing;
                use std::sync::atomic::{AtomicBool, Ordering};
                use std::sync::Arc;
                use std::thread;
                use std::time::{Duration, Instant};

                #[test]
                #[should_panic(expected = "capacity > 0")]
                fn ring_rejects_zero_capacity() {
                    let _ = SampleRing::new(0);
                }

                #[test]
                fn ring_reads_empty_buffer_and_reports_partial_write_and_read() {
                    let ring = SampleRing::new(4);
                    let mut empty = [7.0_f32; 2];
                    assert_eq!(ring.read(&mut empty), 0);
                    assert_eq!(
                        empty,
                        [7.0, 7.0],
                        "empty read must leave destination untouched"
                    );
                    assert_eq!(ring.available_read(), 0);
                    assert_eq!(ring.available_write(), 4);

                    assert_eq!(ring.write(&[1.0, 2.0, 3.0, 4.0, 5.0]), 4);
                    assert_eq!(
                        ring.write(&[6.0]),
                        0,
                        "full ring must reject further samples"
                    );
                    assert_eq!(ring.available_read(), 4);

                    let mut head = [0.0_f32; 2];
                    assert_eq!(ring.read(&mut head), 2);
                    assert_eq!(head, [1.0, 2.0]);
                    assert_eq!(ring.available_write(), 2);
                    assert_eq!(ring.write(&[5.0, 6.0]), 2);

                    let mut tail = [0.0_f32; 4];
                    assert_eq!(ring.read(&mut tail), 4);
                    assert_eq!(tail, [3.0, 4.0, 5.0, 6.0]);
                    assert_eq!(ring.available_read(), 0);
                    assert_eq!(ring.available_write(), 4);
                }

                #[test]
                // Fixture indices stay below 2,003 and are exact in f32.
                #[allow(clippy::cast_precision_loss)]
                fn ring_preserves_order_over_many_physical_wraps() {
                    let ring = SampleRing::new(7);
                    let mut produced = 0usize;
                    let mut received = Vec::new();
                    while received.len() < 2_003 {
                        let end = (produced + 5).min(2_003);
                        let source: Vec<f32> = (produced..end).map(|value| value as f32).collect();
                        let written = ring.write(&source);
                        produced += written;

                        let mut target = [f32::NAN; 3];
                        let read = ring.read(&mut target);
                        received.extend_from_slice(&target[..read]);
                    }
                    received.truncate(2_003);
                    let expected: Vec<f32> = (0..2_003).map(|value| value as f32).collect();
                    assert_eq!(received, expected);
                }

                #[test]
                // Fixture indices stay below 120,000 and are exact in f32.
                #[allow(clippy::cast_precision_loss)]
                fn ring_spsc_concurrency_preserves_every_sample_in_order() {
                    const COUNT: usize = 120_000;
                    let ring = Arc::new(SampleRing::new(127));
                    let done = Arc::new(AtomicBool::new(false));
                    let timed_out = Arc::new(AtomicBool::new(false));
                    let deadline = Instant::now() + Duration::from_secs(8);

                    let producer_ring = ring.clone();
                    let producer_done = done.clone();
                    let producer_timeout = timed_out.clone();
                    let producer_deadline = deadline;
                    let producer = thread::spawn(move || {
                        let mut index = 0usize;
                        let mut source = [0.0_f32; 31];
                        while index < COUNT {
                            if Instant::now() >= producer_deadline {
                                producer_timeout.store(true, Ordering::Release);
                                break;
                            }
                            let count = (COUNT - index).min(source.len());
                            for (offset, sample) in source.iter_mut().take(count).enumerate() {
                                *sample = (index + offset) as f32;
                            }
                            let written = producer_ring.write(&source[..count]);
                            index += written;
                            if written == 0 {
                                thread::yield_now();
                            }
                        }
                        producer_done.store(true, Ordering::Release);
                    });

                    let consumer_ring = ring.clone();
                    let consumer_done = done.clone();
                    let consumer_timeout = timed_out.clone();
                    let consumer_deadline = deadline;
                    let consumer = thread::spawn(move || {
                        let mut received = Vec::with_capacity(COUNT);
                        let mut target = [0.0_f32; 23];
                        while received.len() < COUNT {
                            if Instant::now() >= consumer_deadline {
                                consumer_timeout.store(true, Ordering::Release);
                                break;
                            }
                            let read = consumer_ring.read(&mut target);
                            received.extend_from_slice(&target[..read]);
                            if read == 0 {
                                if consumer_done.load(Ordering::Acquire)
                                    && consumer_ring.available_read() == 0
                                {
                                    break;
                                }
                                thread::yield_now();
                            }
                        }
                        received
                    });

                    producer.join().expect("producer thread must terminate");
                    let received = consumer.join().expect("consumer thread must terminate");
                    assert!(
                        !timed_out.load(Ordering::Acquire),
                        "SPSC exercise timed out"
                    );
                    assert_eq!(received.len(), COUNT);
                    for (index, sample) in received.iter().enumerate() {
                        assert_eq!(
                            *sample, index as f32,
                            "sample lost, duplicated or reordered at {index}"
                        );
                    }
                }
            }

            #[cfg(test)]
            mod music_stream_tests {
                use super::MusicStream;
                use crate::common::TempAudioFile;
                use crate::internal_harness::audio::PlaybackState;

                #[test]
                fn decoder_eof_does_not_stop_music_until_ring_is_consumed() {
                    let wav = TempAudioFile::new("wav");
                    wav.write_pcm16_samples(2, 8_000, &[16_384, -8_192, 8_192, -4_096]);
                    let stream = MusicStream::new(
                        wav.path().to_path_buf(),
                        1.0,
                        false,
                        PlaybackState::Playing,
                        8_000,
                    )
                    .expect("create short music stream");
                    stream.pump().expect("pre-fill ring buffer");
                    assert!(
                        !stream.is_stopped(),
                        "EOF alone must not stop buffered playback"
                    );

                    let mut first = [0.0_f32; 2];
                    stream.mix_into(&mut first, 1.0);
                    assert_eq!(first, [0.5, -0.25]);
                    assert!(
                        !stream.is_stopped(),
                        "remaining buffered samples must stay playable"
                    );

                    let mut second = [0.0_f32; 6];
                    stream.mix_into(&mut second, 1.0);
                    assert_eq!(second, [0.25, -0.125, 0.0, 0.0, 0.0, 0.0]);
                    assert!(
                        stream.is_stopped(),
                        "stream should stop after EOF and full ring drain"
                    );
                    assert_eq!(stream.cursor_frames(), 2);
                }

                #[test]
                fn music_stream_validates_volume_and_pause_resume_preserve_cursor() {
                    let wav = TempAudioFile::new("wav");
                    wav.write_pcm16_samples(2, 8_000, &[16_384, -8_192, 8_192, -4_096]);
                    let stream = MusicStream::new(
                        wav.path().to_path_buf(),
                        1.0,
                        true,
                        PlaybackState::Playing,
                        8_000,
                    )
                    .expect("create looping stream");
                    stream.pump().expect("prefill looping stream");

                    assert!(stream.set_volume(f32::NAN).is_err());
                    assert_eq!(stream.volume(), 1.0, "invalid gain must not replace volume");
                    stream.set_volume(0.25).expect("valid stream gain");
                    stream.pause();

                    let mut paused_output = [0.0_f32; 2];
                    stream.mix_into(&mut paused_output, 0.5);
                    assert_eq!(paused_output, [0.0, 0.0]);
                    assert_eq!(stream.cursor_frames(), 0);

                    stream.resume();
                    let mut empty_output = [];
                    stream.mix_into(&mut empty_output, 1.0);
                    assert_eq!(stream.state(), PlaybackState::Playing);
                    assert_eq!(stream.cursor_frames(), 0);

                    let mut output = [0.0_f32; 2];
                    stream.mix_into(&mut output, 0.5);
                    assert_eq!(output, [0.0625, -0.03125]);
                    assert_eq!(stream.cursor_frames(), 1);
                    stream.stop();
                    assert!(stream.is_stopped());
                }

                #[test]
                fn empty_file_with_looping_does_not_reopen_forever() {
                    let wav = TempAudioFile::new("wav");
                    wav.write_pcm16_samples(2, 8_000, &[]);
                    let stream = MusicStream::new(
                        wav.path().to_path_buf(),
                        1.0,
                        true,
                        PlaybackState::Playing,
                        8_000,
                    )
                    .expect("create empty looping stream");
                    stream
                        .pump()
                        .expect("empty looping stream should terminate pump");
                    assert!(stream.is_stopped());
                    assert_eq!(stream.cursor_frames(), 0);
                }

                #[test]
                fn looping_stream_can_consume_more_than_one_source_pass() {
                    let wav = TempAudioFile::new("wav");
                    // A 12-frame ring at 24 Hz forces several file passes while
                    // keeping the test small and deterministic.
                    wav.write_pcm16_samples(2, 24, &[16_384, -8_192, 8_192, -4_096]);
                    let stream = MusicStream::new(
                        wav.path().to_path_buf(),
                        1.0,
                        true,
                        PlaybackState::Playing,
                        24,
                    )
                    .expect("create looping stream");
                    stream.pump().expect("fill looping ring");
                    let mut output = vec![0.0_f32; 24];
                    stream.mix_into(&mut output, 1.0);
                    assert_eq!(stream.state(), PlaybackState::Playing);
                    assert_eq!(stream.cursor_frames(), 12);
                }
            }
        }
    }

    pub(crate) mod mixer {
        include!("../../crates/vmnl_audio/src/audio/mixer.rs");
    }

    pub(crate) mod runtime {
        include!("../../crates/vmnl_audio/src/audio/runtime.rs");
    }

    pub(crate) mod worker {
        include!("../../crates/vmnl_audio/src/audio/worker.rs");
    }
}
