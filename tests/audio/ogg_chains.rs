// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Chained Vorbis format boundaries through real OGG files, without an output device.

use crate::common::{fixture_path, ogg_chain, TempAudioFile};
use crate::internal_harness::audio::decoder::AudioDecoder;
use crate::internal_harness::audio::error::{AudioError, AudioResult};
use crate::internal_harness::audio::stream_decoder::StreamingDecoder;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

const START: &str = "chain-start-mono-8k.ogg";
const SAME_FORMAT: &str = "chain-next-mono-8k.ogg";
const RATE_CHANGE: &str = "chain-next-mono-16k.ogg";
const CHANNEL_CHANGE: &str = "chain-next-stereo-8k.ogg";
const BOTH_CHANGE: &str = "chain-next-stereo-16k.ogg";

struct ReferenceAudio {
    formats: Vec<(u8, u32)>,
    serials: Vec<u32>,
    samples: Vec<f32>,
    empty_packets: usize,
}

fn reference_audio(path: &Path) -> ReferenceAudio {
    let mut reader = lewton::inside_ogg::OggStreamReader::new(BufReader::new(
        File::open(path).expect("open OGG fixture"),
    ))
    .expect("read Vorbis headers");
    let mut audio = ReferenceAudio {
        formats: Vec::new(),
        serials: Vec::new(),
        samples: Vec::new(),
        empty_packets: 0,
    };

    // These short segments need only a few packets; bound a broken EOF oracle.
    for _ in 0..128 {
        let packet = reader.read_dec_packet_itl().expect("decode reference OGG");
        let channels = reader.ident_hdr.audio_channels;
        if audio.serials.last() != Some(&reader.stream_serial()) {
            audio.serials.push(reader.stream_serial());
            audio
                .formats
                .push((channels, reader.ident_hdr.audio_sample_rate));
        }
        let Some(packet) = packet else {
            return audio;
        };
        assert!(channels == 1 || channels == 2, "mono/stereo fixture");
        assert_eq!(packet.len() % usize::from(channels), 0);
        if packet.is_empty() {
            audio.empty_packets += 1;
        }
        audio.samples.extend(
            packet
                .chunks_exact(usize::from(channels))
                .flat_map(|frame| {
                    let left = f32::from(frame[0]) / f32::from(i16::MAX);
                    let right = if channels == 1 {
                        left
                    } else {
                        f32::from(frame[1]) / f32::from(i16::MAX)
                    };
                    [left, right]
                }),
        );
    }
    panic!("reference OGG did not reach EOF within the fixture bound");
}

fn chained_file(second: &str, second_format: (u8, u32)) -> TempAudioFile {
    let file = ogg_chain(START, second);

    // Check the actual chain, rather than assuming concatenated files were decoded.
    let reference = reference_audio(file.path());
    assert_eq!(reference.formats, [(1, 8_000), second_format]);
    assert_eq!(reference.serials.len(), 2);
    assert_ne!(reference.serials[0], reference.serials[1]);
    assert!(!reference.samples.is_empty());
    file
}

fn streamed_audio(path: &Path, read_sizes: &[usize]) -> AudioResult<Vec<f32>> {
    let mut decoder = StreamingDecoder::open(path, 8_000)?;
    let mut samples = Vec::new();
    for read_index in 0..65_536 {
        assert_eq!(decoder.read_stereo(&mut [])?, 0);
        let mut output = vec![99.0; read_sizes[read_index % read_sizes.len()]];
        let count = decoder.read_stereo(&mut output)?;
        assert_eq!(count % 2, 0);
        samples.extend_from_slice(&output[..count]);
        if count == 0 {
            for _ in 0..3 {
                let mut tail = [99.0; 4];
                assert_eq!(decoder.read_stereo(&mut tail)?, 0);
                assert_eq!(tail, [99.0; 4]);
            }
            return Ok(samples);
        }
    }
    panic!("streaming OGG did not reach EOF within the fixture bound");
}

fn assert_stream_rejected(second: &str, second_format: (u8, u32)) {
    let file = chained_file(second, second_format);
    for sizes in [&[2][..], &[1_024], &[8_192], &[2, 14, 6, 102, 18, 514]] {
        let error = streamed_audio(file.path(), sizes).err();
        assert!(
            matches!(error, Some(AudioError::UnsupportedFormat(_))),
            "OGG format transition to {second_format:?}, reads={sizes:?}: expected UnsupportedFormat, got {error:?}"
        );
    }
}

fn assert_resource_rejected(second: &str, second_format: (u8, u32)) {
    let file = chained_file(second, second_format);
    let error = AudioDecoder::decode_file(file.path()).err();
    assert!(
        matches!(error, Some(AudioError::UnsupportedFormat(_))),
        "OGG resource transition to {second_format:?}: expected UnsupportedFormat, got {error:?}"
    );
}

#[test]
fn fixtures_have_distinct_serials_and_the_expected_formats() {
    let mut serials = HashSet::new();
    for (name, channels, rate, serial) in [
        (START, 1, 8_000, 84),
        (SAME_FORMAT, 1, 8_000, 85),
        (RATE_CHANGE, 1, 16_000, 86),
        (CHANNEL_CHANGE, 2, 8_000, 87),
        (BOTH_CHANGE, 2, 16_000, 88),
        ("chain-start-long-mono-8k.ogg", 1, 8_000, 89),
    ] {
        let audio = reference_audio(&fixture_path(name));
        assert_eq!(audio.formats, [(channels, rate)], "{name}");
        assert_eq!(audio.serials, [serial], "{name}");
        assert!(serials.insert(serial), "duplicate stream serial in {name}");
        assert!(audio.empty_packets > 0, "priming packet in {name}");
        assert!(audio.samples.iter().any(|sample| sample.abs() > 0.0001));
    }
}

#[test]
fn same_format_chain_preserves_reference_pcm_across_read_sizes() {
    let file = chained_file(SAME_FORMAT, (1, 8_000));
    // Lewton's packets from the chain are the oracle, not the durations or the
    // sum of independently decoded segments: priming/trimming can differ.
    let reference = reference_audio(file.path());
    let resource = AudioDecoder::decode_file(file.path()).expect("decode same-format chain");
    assert_eq!(resource.samples(), reference.samples);
    for sizes in [&[2][..], &[1_024], &[8_192], &[2, 14, 6, 102, 18, 514]] {
        let streamed = streamed_audio(file.path(), sizes).expect("decode same-format chain");
        assert_eq!(streamed, reference.samples, "reads={sizes:?}");
    }
}

#[test]
fn sample_rate_change_is_rejected_by_streaming() {
    assert_stream_rejected(RATE_CHANGE, (1, 16_000));
}

#[test]
fn channel_change_is_rejected_by_streaming() {
    assert_stream_rejected(CHANNEL_CHANGE, (2, 8_000));
}

#[test]
fn sample_rate_and_channel_changes_are_rejected_by_streaming() {
    assert_stream_rejected(BOTH_CHANGE, (2, 16_000));
}

#[test]
fn sample_rate_change_is_rejected_by_full_resource_decoding() {
    assert_resource_rejected(RATE_CHANGE, (1, 16_000));
}

#[test]
fn channel_change_is_rejected_by_full_resource_decoding() {
    assert_resource_rejected(CHANNEL_CHANGE, (2, 8_000));
}

#[test]
fn sample_rate_and_channel_changes_are_rejected_by_full_resource_decoding() {
    assert_resource_rejected(BOTH_CHANGE, (2, 16_000));
}
