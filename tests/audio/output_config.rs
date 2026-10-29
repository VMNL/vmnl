// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Selection-policy tests using advertised ranges, without a host or device.

use crate::audio::error::AudioError;
use crate::audio::output_config::select_output_config;
use cpal::{BufferSize, SampleFormat, SupportedBufferSize, SupportedStreamConfigRange};

fn range(
    channels: u16,
    format: SampleFormat,
    minimum: u32,
    maximum: u32,
) -> SupportedStreamConfigRange {
    SupportedStreamConfigRange::new(
        channels,
        minimum,
        maximum,
        SupportedBufferSize::Unknown,
        format,
    )
}

#[test]
fn supported_format_priority_is_independent_of_advertisement_order() {
    let priority = [
        SampleFormat::F32,
        SampleFormat::I16,
        SampleFormat::U16,
        SampleFormat::I32,
        SampleFormat::U32,
        SampleFormat::F64,
    ];
    for (index, expected) in priority.iter().enumerate() {
        let ranges = priority[index..]
            .iter()
            .rev()
            .map(|format| range(2, *format, 48_000, 48_000));
        let selected = select_output_config(ranges, 44_100).expect("supported output");
        assert_eq!(selected.sample_format(), *expected);
        assert_eq!(selected.sample_rate(), 48_000);
    }
}

#[test]
fn eight_bit_formats_remain_supported_as_fallbacks() {
    for format in [SampleFormat::I8, SampleFormat::U8] {
        let selected = select_output_config([range(2, format, 44_100, 44_100)].into_iter(), 44_100)
            .expect("supported eight-bit fallback");
        assert_eq!(selected.sample_format(), format);
    }
}

#[test]
fn requested_rate_is_preserved_or_clamped_to_supported_boundaries() {
    for (requested, expected) in [(8_000, 32_000), (44_100, 44_100), (192_000, 96_000)] {
        let selected = select_output_config(
            [range(2, SampleFormat::F32, 32_000, 96_000)].into_iter(),
            requested,
        )
        .expect("supported range");
        assert_eq!(selected.sample_rate(), expected);
        assert_eq!(selected.channels(), 2);
    }
}

#[test]
fn equal_format_priority_selects_the_closest_rate() {
    for reverse in [false, true] {
        let mut ranges = vec![
            range(2, SampleFormat::F32, 32_000, 32_000),
            range(2, SampleFormat::F32, 48_000, 48_000),
        ];
        if reverse {
            ranges.reverse();
        }
        let selected = select_output_config(ranges.into_iter(), 44_100).expect("closest rate");
        assert_eq!(selected.sample_rate(), 48_000);
    }
}

#[test]
fn preferred_format_precedes_an_exact_rate_in_a_lower_ranked_format() {
    let ranges = [
        range(2, SampleFormat::I16, 44_100, 44_100),
        range(2, SampleFormat::F32, 96_000, 96_000),
    ];
    let selected = select_output_config(ranges.into_iter(), 44_100).expect("preferred format");
    assert_eq!(selected.sample_format(), SampleFormat::F32);
    assert_eq!(selected.sample_rate(), 96_000);
}

#[test]
fn unsupported_channels_and_formats_do_not_override_a_valid_candidate() {
    let ranges = [
        range(1, SampleFormat::F32, 44_100, 44_100),
        range(6, SampleFormat::F32, 44_100, 44_100),
        range(2, SampleFormat::DsdU8, 44_100, 44_100),
        range(2, SampleFormat::I16, 44_100, 44_100),
    ];
    let selected = select_output_config(ranges.into_iter(), 44_100).expect("valid stereo fallback");
    assert_eq!(selected.channels(), 2);
    assert_eq!(selected.sample_format(), SampleFormat::I16);
}

#[test]
fn absence_of_a_supported_stereo_format_is_an_explicit_error() {
    let unsupported = [
        SampleFormat::I24,
        SampleFormat::U24,
        SampleFormat::I64,
        SampleFormat::U64,
        SampleFormat::DsdU8,
        SampleFormat::DsdU16,
        SampleFormat::DsdU32,
    ];
    for ranges in [
        Vec::new(),
        vec![range(1, SampleFormat::F32, 44_100, 44_100)],
        unsupported
            .into_iter()
            .map(|format| range(2, format, 44_100, 44_100))
            .collect(),
    ] {
        assert!(matches!(
            select_output_config(ranges.into_iter(), 44_100),
            Err(AudioError::UnsupportedFormat(_))
        ));
    }
}

#[test]
fn stream_configuration_keeps_the_backend_default_buffer_policy() {
    let range = SupportedStreamConfigRange::new(
        2,
        44_100,
        48_000,
        SupportedBufferSize::Range {
            min: 64,
            max: 2_048,
        },
        SampleFormat::F32,
    );
    let selected = select_output_config([range].into_iter(), 48_000).expect("supported range");
    assert!(matches!(
        selected.buffer_size(),
        SupportedBufferSize::Range {
            min: 64,
            max: 2_048
        }
    ));
    assert!(matches!(selected.config().buffer_size, BufferSize::Default));
}
