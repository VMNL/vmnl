// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Decoder regression coverage using the same production source files and tiny fixtures.

use crate::common::{fixture_path, TempAudioFile};
use crate::internal_harness::audio::decoder::AudioDecoder;
use crate::internal_harness::audio::error::AudioError;
use crate::internal_harness::audio::stream_decoder::StreamingDecoder;
use std::fs::File;
use std::io::BufReader;

fn decode_to_vec(path: &std::path::Path, sample_rate: u32, chunk_sizes: &[usize]) -> Vec<f32> {
    let mut decoder = StreamingDecoder::open(path, sample_rate).expect("open decoder");
    assert_eq!(decoder.target_sample_rate(), sample_rate);

    let mut output = Vec::new();
    let mut chunk_index = 0;
    loop {
        assert_eq!(decoder.read_stereo(&mut []).expect("empty read"), 0);
        let size = chunk_sizes[chunk_index % chunk_sizes.len()];
        let mut chunk = vec![99.0_f32; size];
        let read = decoder
            .read_stereo(&mut chunk)
            .expect("read decoded frames");
        assert_eq!(read % 2, 0, "stereo reads must return whole frames");
        output.extend_from_slice(&chunk[..read]);
        chunk_index += 1;
        if read == 0 {
            break;
        }
    }
    for _ in 0..3 {
        let mut tail = [99.0; 4];
        assert_eq!(decoder.read_stereo(&mut tail).expect("stable EOF"), 0);
        assert_eq!(tail, [99.0; 4]);
    }
    output
}

#[test]
fn pcm16_mono_wav_decodes_to_stereo_with_expected_frames_and_samples() {
    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(1, 8_000, &[0, 8_192, -8_192, 16_384]);

    let decoded = AudioDecoder::decode_file(file.path()).expect("decode valid mono WAV");
    assert_eq!(decoded.channels(), 2);
    assert_eq!(decoded.frame_count(), 4);
    assert_eq!(decoded.samples().len(), 8);
    assert_eq!(
        decoded.samples(),
        &[0.0, 0.0, 0.25, 0.25, -0.25, -0.25, 0.5, 0.5]
    );
}

#[test]
fn float32_wav_decoding_preserves_finite_stereo_samples() {
    let file = TempAudioFile::new("wav");
    file.write_float_wav(2, 44_100, &[0.25, -0.5, 0.75, -1.0]);

    let decoded = AudioDecoder::decode_file(file.path()).expect("decode valid float WAV");
    assert_eq!(decoded.frame_count(), 2);
    assert_eq!(decoded.samples(), &[0.25, -0.5, 0.75, -1.0]);
}

#[test]
fn empty_wav_decodes_to_an_empty_signal_without_panicking() {
    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(2, 44_100, &[]);

    let decoded = AudioDecoder::decode_file(file.path()).expect("decode empty but valid WAV");
    assert!(decoded.is_empty());
    assert_eq!(decoded.frame_count(), 0);
}

fn assert_malformed_wav_is_rejected(path: &std::path::Path) {
    assert!(matches!(
        AudioDecoder::decode_file(path),
        Err(AudioError::DecoderFailed(_))
    ));
    let streamed = StreamingDecoder::open(path, 48_000)
        .and_then(|mut decoder| decoder.read_stereo(&mut [0.0; 4]));
    assert!(matches!(streamed, Err(AudioError::DecoderFailed(_))));
}

#[test]
fn zero_rate_wav_metadata_is_rejected_at_both_decoder_entries() {
    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(2, 8_000, &[8_192, -8_192]);
    let mut bytes = std::fs::read(file.path()).expect("read valid WAV fixture");
    assert_eq!(&bytes[12..16], b"fmt ");
    // Hound puts fmt first: set sample rate and its dependent byte rate to zero.
    bytes[24..28].copy_from_slice(&0_u32.to_le_bytes());
    bytes[28..32].copy_from_slice(&0_u32.to_le_bytes());
    file.write_bytes(&bytes);
    assert_malformed_wav_is_rejected(file.path());
}

#[test]
fn incomplete_stereo_wav_data_is_rejected_at_both_decoder_entries() {
    let file = TempAudioFile::new("wav");
    file.write_pcm16_samples(2, 8_000, &[8_192, -8_192]);
    let mut bytes = std::fs::read(file.path()).expect("read valid WAV fixture");
    let data = bytes
        .windows(4)
        .position(|chunk| chunk == b"data")
        .expect("data chunk");
    // Retain one complete i16 sample, but not one complete stereo frame.
    bytes.truncate(data + 8 + 2);
    let riff_length = u32::try_from(bytes.len() - 8).expect("small fixture length");
    bytes[4..8].copy_from_slice(&riff_length.to_le_bytes());
    bytes[data + 4..data + 8].copy_from_slice(&2_u32.to_le_bytes());
    file.write_bytes(&bytes);
    assert_malformed_wav_is_rejected(file.path());
}

#[test]
fn float32_wav_decoding_rejects_non_finite_samples() {
    let file = TempAudioFile::new("wav");
    file.write_float_wav(1, 44_100, &[0.0, f32::NAN, 0.25]);

    assert!(matches!(
        AudioDecoder::decode_file(file.path()),
        Err(AudioError::DecoderFailed(_))
    ));
}

#[test]
fn wav_with_unsupported_channel_count_is_rejected() {
    let file = TempAudioFile::new("wav");
    file.write_pcm16_wav(3, 44_100, 8, 1_000);

    assert!(matches!(
        StreamingDecoder::open_native(file.path()),
        Err(AudioError::UnsupportedFormat(_))
    ));
}

#[test]
fn malformed_bit_depth_and_corrupt_input_return_errors_without_panicking() {
    let file = TempAudioFile::new("wav");
    file.write_invalid_33_bit_wav();
    let result = std::panic::catch_unwind(|| AudioDecoder::decode_file(file.path()));
    assert!(result.is_ok(), "malformed WAV metadata must not panic");
    assert!(result.expect("catch_unwind returned normally").is_err());

    let corrupt = TempAudioFile::new("wav");
    corrupt.write_bytes(b"RIFF-not-a-complete-WAVE-file");
    assert!(AudioDecoder::decode_file(corrupt.path()).is_err());
}

#[test]
fn unknown_extension_and_missing_file_return_specific_error_variants() {
    let unsupported = TempAudioFile::new("mystery");
    unsupported.write_bytes(b"not an audio file");
    assert!(matches!(
        StreamingDecoder::open_native(unsupported.path()),
        Err(AudioError::UnsupportedFormat(_))
    ));

    let missing =
        std::env::temp_dir().join(format!("vmnl-audio-missing-{}.mp3", std::process::id()));
    let _ = std::fs::remove_file(&missing);
    assert!(matches!(
        StreamingDecoder::open_native(&missing),
        Err(AudioError::Io(_))
    ));

    for extension in ["mp3", "ogg", "flac"] {
        let corrupt = TempAudioFile::new(extension);
        corrupt.write_bytes(b"not a valid compressed audio stream");
        assert!(
            StreamingDecoder::open_native(corrupt.path()).is_err(),
            "corrupt {extension} fixture should be rejected"
        );
    }
}

#[test]
fn ogg_empty_priming_packet_does_not_end_decoding() {
    let path = fixture_path("tiny.ogg");
    let file = File::open(&path).expect("open OGG fixture");
    let mut reference = lewton::inside_ogg::OggStreamReader::new(BufReader::new(file))
        .expect("read Vorbis headers");
    let priming = reference
        .read_dec_packet_itl()
        .expect("decode priming packet")
        .expect("priming packet exists before EOF");
    assert!(
        priming.is_empty(),
        "fixture must exercise an empty priming packet"
    );
    let first_pcm = reference
        .read_dec_packet_itl()
        .expect("decode first PCM packet")
        .expect("PCM packet follows priming");
    assert!(first_pcm.len() >= 14);
    let expected: Vec<f32> = first_pcm[..14]
        .iter()
        .map(|sample| f32::from(*sample) / f32::from(i16::MAX))
        .collect();

    let mut decoder = StreamingDecoder::open(&path, 44_100).expect("open OGG decoder");
    let mut output = [f32::NAN; 14];
    assert_eq!(
        decoder.read_stereo(&mut output).expect("read first PCM"),
        14
    );
    assert_eq!(output.as_slice(), expected);
}

#[test]
fn ogg_stream_preserves_reference_pcm_and_reports_stable_eof() {
    // Lewton 0.10.2 and FFmpeg independently emit 9,792 PCM frames here.
    // The container advertises 8,820 frames; timing/padding is a separate concern.
    const EXPECTED_FRAMES: usize = 9_792;
    let path = fixture_path("tiny.ogg");
    let mut decoder = StreamingDecoder::open(&path, 44_100).expect("open OGG decoder");
    let mut output = [0.0; 14];
    let mut samples = Vec::new();
    let mut reached_eof = false;
    // Bound the regression even if an erroneous decoder never reports EOF.
    for _ in 0..=EXPECTED_FRAMES {
        let count = decoder.read_stereo(&mut output).expect("read OGG chunk");
        assert_eq!(count % 2, 0);
        if count == 0 {
            reached_eof = true;
            break;
        }
        samples.extend_from_slice(&output[..count]);
    }
    assert!(
        reached_eof,
        "decoder must terminate at the physical end of stream"
    );
    assert_eq!(samples.len(), EXPECTED_FRAMES * 2);
    assert!(samples.iter().any(|sample| sample.abs() > 0.0001));
    for _ in 0..3 {
        assert_eq!(
            decoder.read_stereo(&mut output).expect("repeat EOF read"),
            0
        );
    }

    let resource = AudioDecoder::decode_file(&path).expect("decode complete OGG resource");
    assert_eq!(resource.frame_count(), EXPECTED_FRAMES);
    assert_same_audio(
        resource.samples(),
        &samples,
        "streamed OGG differs from full resource",
    );
}

#[test]
fn compressed_fixtures_decode_real_samples_for_mp3_ogg_and_flac() {
    for fixture in ["tiny.mp3", "tiny.ogg", "tiny.flac"] {
        let path = fixture_path(fixture);
        let decoded = AudioDecoder::decode_file(&path)
            .unwrap_or_else(|error| panic!("failed to decode {fixture}: {error}"));
        assert_eq!(decoded.channels(), 2, "{fixture} should convert to stereo");
        assert!(
            decoded.frame_count() > 0,
            "{fixture} must produce audio frames"
        );
        assert_eq!(decoded.samples().len(), decoded.frame_count() * 2);
        assert!(decoded.samples().iter().all(|sample| sample.is_finite()));
        assert!(decoded.samples().iter().any(|sample| sample.abs() > 0.0001));
    }
}

#[test]
fn varied_read_sizes_match_continuous_reads_across_codec_chunk_boundaries() {
    for fixture in ["tiny.wav", "tiny.mp3", "tiny.ogg", "tiny.flac"] {
        let path = if fixture == "tiny.wav" {
            // This generated file contains a known, deterministic waveform.
            let file = TempAudioFile::new("wav");
            let samples: Vec<i16> = (0..12_000)
                .flat_map(|frame| {
                    let sample = ((frame % 128) as i16 - 64) * 300;
                    [sample, -sample]
                })
                .collect();
            file.write_pcm16_samples(2, 44_100, &samples);
            let full = decode_to_vec(file.path(), 48_000, &[8_192]);
            let partial = decode_to_vec(file.path(), 48_000, &[2, 14, 6, 102, 18, 514]);
            assert_same_audio(
                &full,
                &partial,
                "WAV samples must not be lost at buffer boundaries",
            );
            continue;
        } else {
            fixture_path(fixture)
        };

        let full = decode_to_vec(&path, 48_000, &[16_384]);
        let partial = decode_to_vec(&path, 48_000, &[2, 14, 6, 102, 18, 514]);
        assert_same_audio(
            &full,
            &partial,
            &format!("decoded samples changed across chunks for {fixture}"),
        );
    }
}

#[test]
fn resampling_frame_count_and_pcm_are_independent_of_read_sizes() {
    for (source_rate, target_rate) in [
        (44_100, 48_000),
        (22_050, 48_000),
        (48_000, 44_100),
        (48_000, 22_050),
        (44_100, 44_100),
    ] {
        for channels in [1_u16, 2] {
            for frames in [0_usize, 1, 2, 147, 441, 4_410, 44_100] {
                let file = TempAudioFile::new("wav");
                let samples: Vec<i16> = (0..frames * usize::from(channels))
                    .map(|index| (i16::try_from(index % 97).expect("bounded pattern") - 48) * 500)
                    .collect();
                file.write_pcm16_samples(channels, source_rate, &samples);
                let expected_frames = usize::try_from(
                    (frames as u128 * u128::from(target_rate)).div_ceil(u128::from(source_rate)),
                )
                .expect("small fixture");
                let full = AudioDecoder::decode_file(file.path())
                    .expect("decode WAV")
                    .resample_to_stereo(target_rate)
                    .expect("resample sound");
                assert_eq!(full.frame_count(), expected_frames);
                let reference = decode_to_vec(file.path(), target_rate, &[8_192]);
                for sizes in [&[2][..], &[1_024], &[8_192], &[2, 14, 6, 102, 18, 514]] {
                    let streamed = decode_to_vec(file.path(), target_rate, sizes);
                    let context = format!(
                        "{frames} frames, {channels} channels, {source_rate}->{target_rate}, reads={sizes:?}"
                    );
                    assert_eq!(streamed.len() / 2, expected_frames, "{context}");
                    assert!(
                        streamed.iter().eq(&reference),
                        "chunk-dependent PCM: {context}"
                    );
                    assert_same_audio(full.samples(), &streamed, &context);
                }
            }
        }
    }
}

fn assert_same_audio(expected: &[f32], actual: &[f32], message: &str) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "{message}: sample count differs"
    );
    for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        assert!(
            (expected - actual).abs() <= 1.0e-5,
            "{message}: sample {index} differs ({expected} vs {actual})"
        );
    }
}

#[test]
fn decoder_rejects_odd_sized_stereo_output_buffer() {
    let fixture = TempAudioFile::new("wav");
    fixture.write_pcm16_wav(1, 44_100, 20, 1_000);
    let mut decoder = StreamingDecoder::open_native(fixture.path()).expect("open WAV decoder");
    let mut odd = [0.0_f32; 3];
    assert!(matches!(
        decoder.read_stereo(&mut odd),
        Err(AudioError::InvalidState(_))
    ));
}
