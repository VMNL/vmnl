// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Deterministic audio fixtures and helpers shared by audio integration tests.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::NamedTempFile;

#[allow(dead_code)]
pub struct TempAudioFile {
    file: NamedTempFile,
}

#[allow(dead_code)]
impl TempAudioFile {
    pub fn new(extension: &str) -> Self {
        let file = tempfile::Builder::new()
            .prefix("vmnl-audio-test-")
            .suffix(&format!(".{extension}"))
            .tempfile()
            .expect("create temporary audio fixture");
        Self { file }
    }

    pub fn path(&self) -> &Path {
        self.file.path()
    }

    pub fn write_bytes(&self, bytes: &[u8]) {
        std::fs::write(self.path(), bytes).expect("write temporary fixture");
    }

    pub fn write_pcm16_wav(&self, channels: u16, sample_rate: u32, frames: usize, sample: i16) {
        let count = frames
            .checked_mul(usize::from(channels))
            .expect("fixture sample count should fit usize");
        let samples = vec![sample; count];
        self.write_pcm16_samples(channels, sample_rate, &samples);
    }

    pub fn write_pcm16_samples(&self, channels: u16, sample_rate: u32, samples: &[i16]) {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let file = File::create(self.path()).expect("open WAV fixture for writing");
        let mut writer = hound::WavWriter::new(file, spec).expect("create PCM16 WAV fixture");
        for sample in samples {
            writer.write_sample(*sample).expect("write PCM16 sample");
        }
        writer.finalize().expect("finalize PCM16 WAV fixture");
    }

    pub fn write_float_wav(&self, channels: u16, sample_rate: u32, samples: &[f32]) {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let file = File::create(self.path()).expect("open float WAV fixture for writing");
        let mut writer = hound::WavWriter::new(file, spec).expect("create float WAV fixture");
        for sample in samples {
            writer.write_sample(*sample).expect("write float sample");
        }
        writer.finalize().expect("finalize float WAV fixture");
    }

    pub fn write_invalid_33_bit_wav(&self) {
        // Minimal RIFF/WAVE header with a PCM format claiming 33 bits/sample.
        // The decoder must reject the metadata instead of shifting by an
        // invalid amount or panicking while converting samples.
        let channels: u16 = 1;
        let sample_rate: u32 = 44_100;
        let bits_per_sample: u16 = 33;
        let block_align: u16 = channels * bits_per_sample.div_ceil(8);
        let byte_rate = sample_rate * u32::from(block_align);
        let data = [0_u8; 5];
        let data_len = u32::try_from(data.len()).expect("fixed fixture fits u32");
        let riff_size = 4_u32 + (8 + 16) + (8 + data_len);

        let mut bytes = Vec::with_capacity(44 + data.len());
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&riff_size.to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        bytes.extend_from_slice(&data);
        self.write_bytes(&bytes);
    }
}

#[allow(dead_code)]
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/audio/fixtures")
        .join(name)
}

pub fn ogg_chain(first: &str, second: &str) -> TempAudioFile {
    let file = TempAudioFile::new("ogg");
    let mut bytes = std::fs::read(fixture_path(first)).expect("read first OGG segment");
    bytes.extend_from_slice(&std::fs::read(fixture_path(second)).expect("read second OGG segment"));
    file.write_bytes(&bytes);
    file
}

pub fn wait_until(timeout: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        thread::yield_now();
        thread::sleep(Duration::from_millis(2));
    }
    condition()
}
