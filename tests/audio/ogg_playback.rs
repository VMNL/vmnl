// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Chained OGG failures after a successful music prefill, without an output device.

use crate::common::{fixture_path, ogg_chain, wait_until};
use crate::internal_harness::audio::decoder::AudioDecoder;
use crate::internal_harness::audio::error::AudioError;
use crate::internal_harness::audio::music::MusicStream;
use crate::internal_harness::audio::runtime::AudioRuntime;
use crate::internal_harness::audio::worker::{update_control, ControlWorker};
use crate::internal_harness::audio::PlaybackState;
use std::sync::Arc;
use std::time::Duration;

const LONG_START: &str = "chain-start-long-mono-8k.ogg";
const CHANGES: [&str; 3] = [
    "chain-next-mono-16k.ogg",
    "chain-next-stereo-8k.ogg",
    "chain-next-stereo-16k.ogg",
];

fn prefilled(path: &std::path::Path) -> (Arc<AudioRuntime>, Arc<MusicStream>) {
    let runtime = Arc::new(AudioRuntime::new());
    let stream = Arc::new(
        MusicStream::new(
            path.to_path_buf(),
            1.0,
            false,
            PlaybackState::Playing,
            8_000,
        )
        .expect("open chained OGG"),
    );
    stream
        .pump()
        .expect("format transition must be after prefill");
    assert_eq!(stream.state(), PlaybackState::Playing);
    assert_eq!(stream.cursor_frames(), 0);
    runtime
        .register_music_stream(stream.clone())
        .expect("publish prefilled stream");
    (runtime, stream)
}

fn consume_prefix(stream: &MusicStream, reference: &[f32]) {
    let before = stream.cursor_frames();
    let mut output = [0.0; 512];
    stream.mix_into(&mut output, 1.0);
    let after = stream.cursor_frames();
    assert!(
        after * 2 <= reference.len(),
        "incompatible PCM was consumed"
    );
    assert_eq!(
        &output[..(after - before) * 2],
        &reference[before * 2..after * 2],
        "consumed PCM must match the valid source prefix"
    );
}

fn assert_cleaned(runtime: &AudioRuntime, stream: &MusicStream) {
    assert!(stream.is_stopped());
    assert!(runtime
        .music_stream_snapshot
        .read()
        .expect("single snapshot reader")
        .is_empty());
    let cursor = stream.cursor_frames();
    stream.resume();
    stream.set_looping(true);
    stream
        .pump()
        .expect("stopped stream does not retry decoding");
    let mut output = [99.0; 4];
    runtime.mix_into(&mut output);
    assert_eq!(output, [0.0; 4]);
    assert!(stream.is_stopped());
    assert_eq!(stream.cursor_frames(), cursor);
}

#[test]
fn manual_chained_ogg_refill_error_is_preserved_and_cleanup_is_terminal() {
    let prefix = AudioDecoder::decode_file(fixture_path(LONG_START)).expect("decode valid prefix");
    assert!(
        prefix.frame_count() > 4_000,
        "prefix exceeds the 500 ms ring"
    );
    for second in CHANGES {
        let file = ogg_chain(LONG_START, second);
        let expected = AudioDecoder::decode_file(file.path())
            .expect_err("whole resource rejects the later format transition")
            .to_string();
        let (runtime, stream) = prefilled(file.path());
        let mut failure = None;
        for _ in 0..64 {
            consume_prefix(&stream, prefix.samples());
            if let Err(error) = runtime.tick() {
                failure = Some(error);
                break;
            }
        }
        let error = failure.expect("refill must reach the chained format transition");
        assert!(matches!(error, AudioError::UnsupportedFormat(_)));
        assert_eq!(
            error.to_string(),
            expected,
            "retain the original decoder error"
        );
        assert!(
            stream.cursor_frames() > 0,
            "failure occurs after playback began"
        );
        assert_cleaned(&runtime, &stream);
        runtime
            .tick()
            .expect("failed stream was removed, not retried");
    }
}

#[test]
fn automatic_chained_ogg_refill_failure_survives_taking_the_diagnostic() {
    let prefix = AudioDecoder::decode_file(fixture_path(LONG_START)).expect("decode valid prefix");
    for second in CHANGES {
        let file = ogg_chain(LONG_START, second);
        let expected = AudioDecoder::decode_file(file.path())
            .expect_err("whole resource rejects the later format transition")
            .to_string();
        let (runtime, stream) = prefilled(file.path());
        let errors = Arc::new(parking_lot::Mutex::new(None));
        let mut worker =
            ControlWorker::spawn(runtime.clone(), errors.clone()).expect("spawn worker");
        assert!(wait_until(Duration::from_secs(3), || {
            consume_prefix(&stream, prefix.samples());
            errors.lock().is_some()
        }));
        let diagnostic = errors.lock().take().expect("background diagnostic");
        assert!(
            diagnostic.contains(&expected),
            "retain the original decoder error"
        );
        assert!(matches!(
            update_control(&runtime, Some(&worker)),
            Err(AudioError::InvalidState(_))
        ));
        assert!(errors.lock().is_none());
        worker.stop();
        assert!(stream.cursor_frames() > 0);
        assert_cleaned(&runtime, &stream);
        assert_eq!(
            Arc::strong_count(&runtime),
            1,
            "joined worker releases runtime"
        );
    }
}

#[test]
fn same_format_chained_ogg_completes_through_manual_refills() {
    let file = ogg_chain(LONG_START, "chain-next-mono-8k.ogg");
    let reference = AudioDecoder::decode_file(file.path()).expect("decode same-format chain");
    let (runtime, stream) = prefilled(file.path());
    for _ in 0..64 {
        consume_prefix(&stream, reference.samples());
        runtime.tick().expect("same-format refill stays valid");
        if stream.is_stopped() {
            break;
        }
    }
    assert_eq!(stream.cursor_frames(), reference.frame_count());
    assert_cleaned(&runtime, &stream);
}
