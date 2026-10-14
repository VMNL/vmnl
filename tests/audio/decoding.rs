// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Decoder regression coverage using the same production source files and tiny fixtures.

use crate::common::{fixture_path, TempAudioFile};
use crate::internal_harness::audio::decoder::AudioDecoder;
use crate::internal_harness::audio::error::AudioError;
use crate::internal_harness::audio::stream_decoder::StreamingDecoder;

fn decode_to_vec(path: &std::path::Path, sample_rate: u32, chunk_sizes: &[usize]) -> Vec<f32> {
    let mut decoder = StreamingDecoder::open(path, sample_rate).expect("open decoder");
    assert_eq!(decoder.target_sample_rate(), sample_rate);

    let mut output = Vec::new();
    let mut chunk_index = 0;
    loop {
        let size = chunk_sizes[chunk_index % chunk_sizes.len()];
        let mut chunk = vec![99.0_f32; size];
        let read = decoder.read_stereo(&mut chunk).expect("read decoded frames");
        assert_eq!(read % 2, 0, "stereo reads must return whole frames");
        output.extend_from_slice(&chunk[..read]);
        chunk_index += 1;
        if read == 0 {
            break;
        }
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
    assert_eq!(decoded.samples(), &[0.0, 0.0, 0.25, 0.25, -0.25, -0.25, 0.5, 0.5]);
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

    let missing = std::env::temp_dir().join(format!("vmnl-audio-missing-{}.mp3", std::process::id()));
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
fn compressed_fixtures_decode_real_samples_for_mp3_ogg_and_flac() {
    for fixture in ["tiny.mp3", "tiny.ogg", "tiny.flac"] {
        let path = fixture_path(fixture);
        let decoded = AudioDecoder::decode_file(&path)
            .unwrap_or_else(|error| panic!("failed to decode {fixture}: {error}"));
        assert_eq!(decoded.channels(), 2, "{fixture} should convert to stereo");
        assert!(decoded.frame_count() > 0, "{fixture} must produce audio frames");
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
            assert_same_audio(&full, &partial, "WAV samples must not be lost at buffer boundaries");
            continue;
        } else {
            fixture_path(fixture)
        };

        let full = decode_to_vec(&path, 48_000, &[16_384]);
        let partial = decode_to_vec(&path, 48_000, &[2, 14, 6, 102, 18, 514]);
        assert_same_audio(&full, &partial, &format!("decoded samples changed across chunks for {fixture}"));
    }
}

fn assert_same_audio(expected: &[f32], actual: &[f32], message: &str) {
    assert_eq!(expected.len(), actual.len(), "{message}: sample count differs");
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
