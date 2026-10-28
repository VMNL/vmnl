// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Device-independent selection from backend-advertised stereo output ranges.

use crate::audio::error::{AudioError, AudioResult};
use cpal::{SampleFormat, SupportedStreamConfig, SupportedStreamConfigRange};

/// The device validates the requested rate before calling. Ranges must be valid
/// backend advertisements; format rank takes precedence over rate distance.
pub(crate) fn select_output_config(
    ranges: impl Iterator<Item = SupportedStreamConfigRange>,
    requested_rate: u32,
) -> AudioResult<SupportedStreamConfig> {
    let (_, _, range, actual_rate) = ranges
        .filter(|range| range.channels() == 2)
        .filter_map(|range| {
            let rank = sample_format_rank(range.sample_format())?;
            let actual_rate =
                requested_rate.clamp(range.min_sample_rate(), range.max_sample_rate());
            Some((
                rank,
                requested_rate.abs_diff(actual_rate),
                range,
                actual_rate,
            ))
        })
        .min_by_key(|(rank, distance, _, _)| (*rank, *distance))
        .ok_or_else(|| {
            AudioError::UnsupportedFormat(
                "the default audio device has no supported stereo output format".to_owned(),
            )
        })?;
    Ok(range.with_sample_rate(actual_rate))
}

fn sample_format_rank(format: SampleFormat) -> Option<u8> {
    Some(match format {
        SampleFormat::F32 => 0,
        SampleFormat::I16 => 1,
        SampleFormat::U16 => 2,
        SampleFormat::I32 => 3,
        SampleFormat::U32 => 4,
        SampleFormat::F64 => 5,
        SampleFormat::I8 | SampleFormat::U8 => 6,
        _ => return None,
    })
}
