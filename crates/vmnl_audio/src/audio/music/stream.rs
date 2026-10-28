// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::error::{validate_gain, AudioError, AudioResult};
use crate::audio::stream_decoder::StreamingDecoder;
use crate::audio::PlaybackState;

use std::cell::UnsafeCell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const OUTPUT_CHANNELS: usize = 2;
const DECODE_CHUNK_FRAMES: usize = 4096;
const STREAM_BUFFER_MS: usize = 500;

struct SampleRing {
    data: Box<[UnsafeCell<f32>]>,
    capacity: usize,
    read_index: AtomicUsize,
    write_index: AtomicUsize,
}

// SAFETY: The ring owns its cells and atomics; moving it transfers that storage
// without exposing references to its contents.
unsafe impl Send for SampleRing {}
// SAFETY: pump() serializes the producer with its decoder/scratch mutexes; the
// CPAL callback is the sole consumer. Release/acquire publication of write_index
// protects initialized slots, and read_index prevents overwriting unread slots.
unsafe impl Sync for SampleRing {}

impl SampleRing {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        let data = (0..capacity)
            .map(|_| UnsafeCell::new(0.0))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            data,
            capacity,
            read_index: AtomicUsize::new(0),
            write_index: AtomicUsize::new(0),
        }
    }

    fn available_read(&self) -> usize {
        let write = self.write_index.load(Ordering::Acquire);
        let read = self.read_index.load(Ordering::Relaxed);
        write.wrapping_sub(read).min(self.capacity)
    }

    fn available_write(&self) -> usize {
        self.capacity - self.available_read()
    }

    /// Producer thread only.
    fn write(&self, samples: &[f32]) -> usize {
        let write = self.write_index.load(Ordering::Relaxed);
        let read = self.read_index.load(Ordering::Acquire);
        let available = self.capacity - write.wrapping_sub(read).min(self.capacity);
        let count = samples.len().min(available);

        for (offset, sample) in samples[..count].iter().copied().enumerate() {
            let index = write.wrapping_add(offset) % self.capacity;
            // SAFETY: count excludes unread slots. Only the serialized producer
            // writes these cells, before publishing them with a Release store.
            unsafe {
                *self.data[index].get() = sample;
            }
        }

        self.write_index
            .store(write.wrapping_add(count), Ordering::Release);
        count
    }

    /// CPAL consumer thread only.
    fn read(&self, output: &mut [f32]) -> usize {
        let read = self.read_index.load(Ordering::Relaxed);
        let write = self.write_index.load(Ordering::Acquire);
        let available = write.wrapping_sub(read).min(self.capacity);
        let count = output.len().min(available);

        for (offset, destination) in output[..count].iter_mut().enumerate() {
            let index = read.wrapping_add(offset) % self.capacity;
            // SAFETY: the Acquire load observes initialized slots. Only the
            // consumer reads them, before releasing them through read_index.
            unsafe {
                *destination = *self.data[index].get();
            }
        }

        self.read_index
            .store(read.wrapping_add(count), Ordering::Release);
        count
    }
}

pub(crate) struct MusicStream {
    /// Used only by `pump()` on the control thread.
    decoder: Mutex<StreamingDecoder>,
    decoder_scratch: Mutex<Vec<f32>>,
    /// Preallocated PCM data consumed by the realtime callback.
    ring: Arc<SampleRing>,
    cursor_frames: AtomicUsize,
    volume_bits: AtomicU32,
    looping: AtomicBool,
    state: AtomicU8,
    /// Means decoder EOF, not playback completion.
    decoder_eof: AtomicBool,
}

impl MusicStream {
    pub(crate) fn new(
        path: PathBuf,
        volume: f32,
        looping: bool,
        state: PlaybackState,
        target_sample_rate: u32,
    ) -> AudioResult<Self> {
        let volume = validate_gain(volume)?;
        if target_sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "target sample rate must be greater than zero".to_owned(),
            ));
        }

        let decoder = StreamingDecoder::open(path, target_sample_rate)?;
        Ok(Self {
            decoder: Mutex::new(decoder),
            decoder_scratch: Mutex::new(vec![0.0; DECODE_CHUNK_FRAMES * OUTPUT_CHANNELS]),
            ring: Arc::new(SampleRing::new(
                ((target_sample_rate as usize * STREAM_BUFFER_MS) / 1000).max(1) * OUTPUT_CHANNELS,
            )),
            cursor_frames: AtomicUsize::new(0),
            volume_bits: AtomicU32::new(volume.to_bits()),
            looping: AtomicBool::new(looping),
            state: AtomicU8::new(state as u8),
            decoder_eof: AtomicBool::new(false),
        })
    }

    pub(crate) fn set_looping(&self, looping: bool) {
        self.looping.store(looping, Ordering::Relaxed);
    }

    pub(crate) fn set_volume(&self, volume: f32) -> AudioResult<()> {
        let volume = validate_gain(volume)?;
        self.volume_bits.store(volume.to_bits(), Ordering::Relaxed);
        Ok(())
    }

    fn volume(&self) -> f32 {
        f32::from_bits(self.volume_bits.load(Ordering::Relaxed))
    }

    fn set_state(&self, state: PlaybackState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }

    pub(crate) fn state(&self) -> PlaybackState {
        match self.state.load(Ordering::Relaxed) {
            0 => PlaybackState::Playing,
            1 => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        }
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.state() == PlaybackState::Stopped
    }

    fn is_decoder_eof(&self) -> bool {
        self.decoder_eof.load(Ordering::Acquire)
    }

    pub(crate) fn stop(&self) {
        self.decoder_eof.store(true, Ordering::Release);
        self.set_state(PlaybackState::Stopped);
    }

    pub(crate) fn pause(&self) {
        // Only Playing can become Paused; a concurrent stop must remain terminal.
        let _ = self.state.compare_exchange(
            PlaybackState::Playing as u8,
            PlaybackState::Paused as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }

    pub(crate) fn resume(&self) {
        let _ = self.state.compare_exchange(
            PlaybackState::Paused as u8,
            PlaybackState::Playing as u8,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }

    pub(crate) fn cursor_frames(&self) -> usize {
        self.cursor_frames.load(Ordering::Relaxed)
    }

    /// Control-thread function: decode only enough source data to refill the ring.
    pub(crate) fn pump(&self) -> AudioResult<()> {
        if self.is_stopped() {
            return Ok(());
        }
        if self.is_decoder_eof() {
            if !self.looping.load(Ordering::Relaxed) {
                if self.ring.available_read() == 0 {
                    self.set_state(PlaybackState::Stopped);
                }
                return Ok(());
            }
            // A live playback can request looping after prefill reached EOF.
            self.decoder_eof.store(false, Ordering::Release);
        }

        let mut decoder = self
            .decoder
            .lock()
            .map_err(|_| AudioError::MusicDecoderPoisoned)?;
        let mut scratch = self
            .decoder_scratch
            .lock()
            .map_err(|_| AudioError::MusicDecoderPoisoned)?;

        let target_samples = self.ring.capacity;

        while self.ring.available_read() < target_samples {
            let writable_samples = self.ring.available_write();
            let requested_frames = (writable_samples / OUTPUT_CHANNELS).min(DECODE_CHUNK_FRAMES);
            if requested_frames == 0 {
                break;
            }
            let requested_samples = requested_frames * OUTPUT_CHANNELS;
            let samples_read = decoder.read_stereo(&mut scratch[..requested_samples])?;

            if samples_read > 0 {
                let written = self.ring.write(&scratch[..samples_read]);
                if written < samples_read {
                    break;
                }
                continue;
            }

            if !self.looping.load(Ordering::Relaxed) {
                // Keep already decoded samples in the ring; playback ends only
                // after the realtime callback has consumed those samples.
                self.decoder_eof.store(true, Ordering::Release);
                break;
            }

            // EOF on a looping stream is a request to reopen, not a playback
            // completion. Probe the replacement before installing it so an
            // empty file cannot cause an infinite reopen loop.
            let mut replacement = decoder.reopen()?;
            let first_samples = replacement.read_stereo(&mut scratch[..requested_samples])?;
            if first_samples == 0 {
                self.decoder_eof.store(true, Ordering::Release);
                break;
            }

            let written = self.ring.write(&scratch[..first_samples]);
            *decoder = replacement;
            if written < first_samples {
                break;
            }
        }

        if self.is_decoder_eof() && self.ring.available_read() == 0 {
            self.set_state(PlaybackState::Stopped);
        }
        Ok(())
    }

    /// Realtime path. No locks, allocations, file access or decoding here.
    pub(crate) fn mix_into(&self, output: &mut [f32], gain: f32) {
        if self.state() != PlaybackState::Playing {
            return;
        }

        let output_samples = output.len() / OUTPUT_CHANNELS * OUTPUT_CHANNELS;
        if output_samples == 0 {
            return;
        }

        let volume = self.volume();
        let mut scratch = [0.0f32; 512];
        let mut output_position = 0usize;

        while output_position < output_samples {
            let chunk_len = (output_samples - output_position).min(scratch.len());
            let read = self.ring.read(&mut scratch[..chunk_len]);
            if read == 0 {
                break;
            }

            for index in 0..read {
                output[output_position + index] += scratch[index] * volume * gain;
            }
            output_position += read;
            self.cursor_frames
                .fetch_add(read / OUTPUT_CHANNELS, Ordering::Relaxed);
        }

        // EOF only means finished once all decoded samples have been consumed.
        if self.is_decoder_eof()
            && self.ring.available_read() == 0
            && !self.looping.load(Ordering::Relaxed)
        {
            self.set_state(PlaybackState::Stopped);
        }
    }
}
