////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use crate::audio::bus::BusKind;
use crate::audio::error::{
    validate_gain,
    AudioError,
    AudioResult,
};
use crate::audio::PlaybackState;

use miniaudio::{
    Decoder,
    DecoderConfig,
    Format,
};

use std::cell::UnsafeCell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{
    AtomicBool,
    AtomicU32,
    AtomicU8,
    AtomicUsize,
    Ordering,
};
use std::sync::{Arc, Mutex};

const OUTPUT_CHANNELS: u32 = 2;
const DECODE_CHUNK_FRAMES: usize = 4096;

/*
 * 500 ms of stereo audio at 44.1 kHz.
 *
 * This is deliberately a fixed preallocated buffer.
 */
const STREAM_BUFFER_FRAMES: usize = 22_050;

struct SampleRing {
    data: Box<[UnsafeCell<f32>]>,
    capacity: usize,

    read_index: AtomicUsize,
    write_index: AtomicUsize,
}

unsafe impl Send for SampleRing {}
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

        write
            .wrapping_sub(read)
            .min(self.capacity)
    }

    fn available_write(&self) -> usize {
        self.capacity - self.available_read()
    }

    /*
     * Called ONLY by the producer.
     */
    fn write(&self, samples: &[f32]) -> usize {
        let write = self.write_index.load(Ordering::Relaxed);
        let read = self.read_index.load(Ordering::Acquire);

        let available = self.capacity
            - write
                .wrapping_sub(read)
                .min(self.capacity);

        let count = samples.len().min(available);

        for (offset, sample) in samples[..count]
            .iter()
            .copied()
            .enumerate()
        {
            let index =
                write.wrapping_add(offset) % self.capacity;

            unsafe {
                *self.data[index].get() = sample;
            }
        }

        self.write_index.store(
            write.wrapping_add(count),
            Ordering::Release,
        );

        count
    }

    /*
     * Called ONLY by the consumer.
     */
    fn read(&self, output: &mut [f32]) -> usize {
        let read = self.read_index.load(Ordering::Relaxed);
        let write = self.write_index.load(Ordering::Acquire);

        let available = write
            .wrapping_sub(read)
            .min(self.capacity);

        let count = output.len().min(available);

        for (offset, destination) in output[..count]
            .iter_mut()
            .enumerate()
        {
            let index =
                read.wrapping_add(offset) % self.capacity;

            unsafe {
                *destination = *self.data[index].get();
            }
        }

        self.read_index.store(
            read.wrapping_add(count),
            Ordering::Release,
        );

        count
    }
}

struct MixGuard<'a> {
    flag: &'a AtomicBool,
}

impl Drop for MixGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::Release);
    }
}

pub(crate) struct MusicStream {
    id: u64,
    path: PathBuf,

    /*
     * Decoder and scratch buffer are ONLY used by pump().
     */
    decoder: Mutex<Decoder>,
    decoder_scratch: Mutex<Vec<f32>>,

    /*
     * Realtime PCM buffer.
     */
    ring: Arc<SampleRing>,

    cursor_frames: AtomicUsize,

    volume_bits: AtomicU32,
    looping: AtomicBool,
    state: AtomicU8,
    finished: AtomicBool,

    /*
     * Prevent two concurrent render_into() calls from consuming
     * this stream simultaneously.
     *
     * This is a try-lock, never a blocking lock.
     */
    mix_guard: AtomicBool,

    bus: BusKind,
}

impl MusicStream {
    #[must_use]
    pub(crate) fn new(
        id: u64,
        path: PathBuf,
        bus: BusKind,
        volume: f32,
        looping: bool,
        state: PlaybackState,
        target_sample_rate: u32,
    ) -> AudioResult<Self> {
        let volume = validate_gain(volume)?;

        if target_sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "target sample rate must be greater than zero"
                    .to_string(),
            ));
        }

        /*
         * Ask miniaudio for the format consumed by the mixer.
         *
         * This also performs:
         * - sample-rate conversion
         * - channel conversion
         */
        let decoder_config = DecoderConfig::new(
            Format::F32,
            OUTPUT_CHANNELS,
            target_sample_rate,
        );

        let decoder =
            Decoder::from_file(
                &path,
                Some(&decoder_config),
            )
            .map_err(|error| {
                AudioError::DecoderFailed(error.to_string())
            })?;

        Ok(Self {
            id,
            path,

            decoder: Mutex::new(decoder),

            decoder_scratch: Mutex::new(
                vec![
                    0.0;
                    DECODE_CHUNK_FRAMES
                        * OUTPUT_CHANNELS as usize
                ],
            ),

            ring: Arc::new(
                SampleRing::new(
                    STREAM_BUFFER_FRAMES
                        * OUTPUT_CHANNELS as usize,
                ),
            ),

            cursor_frames: AtomicUsize::new(0),

            volume_bits: AtomicU32::new(
                volume.to_bits(),
            ),

            looping: AtomicBool::new(looping),

            state: AtomicU8::new(
                state as u8
            ),

            finished: AtomicBool::new(false),

            mix_guard: AtomicBool::new(false),

            bus,
        })
    }

    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn bus(&self) -> BusKind {
        self.bus
    }

    pub fn set_looping(&self, looping: bool) {
        self.looping.store(
            looping,
            Ordering::Relaxed,
        );
    }

    #[must_use]
    pub fn looping(&self) -> bool {
        self.looping.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, volume: f32) -> AudioResult<()> {
        let volume = validate_gain(volume)?;

        self.volume_bits.store(
            volume.to_bits(),
            Ordering::Relaxed,
        );

        Ok(())
    }

    #[must_use]
    pub fn volume(&self) -> f32 {
        f32::from_bits(
            self.volume_bits.load(Ordering::Relaxed)
        )
    }

    pub(crate) fn start(&self) {
        self.set_state(PlaybackState::Playing);
    }

    pub fn set_state(&self, state: PlaybackState) {
        self.state.store(
            state as u8,
            Ordering::Relaxed,
        );
    }

    pub fn state(&self) -> PlaybackState {
        match self.state.load(Ordering::Relaxed) {
            0 => PlaybackState::Playing,
            1 => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        }
    }

    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.state() == PlaybackState::Playing
    }

    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.state() == PlaybackState::Paused
    }

    #[must_use]
    pub fn is_stopped(&self) -> bool {
        self.state() == PlaybackState::Stopped
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.finished.store(
            true,
            Ordering::Release,
        );

        self.set_state(PlaybackState::Stopped);
    }

    pub fn pause(&self) {
        self.set_state(PlaybackState::Paused);
    }

    pub fn resume(&self) {
        if !self.is_stopped() {
            self.set_state(PlaybackState::Playing);
        }
    }

    #[must_use]
    pub fn cursor_frames(&self) -> usize {
        self.cursor_frames.load(Ordering::Relaxed)
    }

    pub fn set_cursor_frames(&self, cursor: usize) {
        self.cursor_frames.store(
            cursor,
            Ordering::Relaxed,
        );
    }

    /*
     * REAL STREAMING
     *
     * pump() runs on the control/update thread.
     * It reads only a few thousand frames from the decoder
     * and places them in the ring buffer.
     */
    pub fn pump(&self) -> AudioResult<()> {
        if self.is_stopped() {
            return Ok(());
        }

        if self.is_finished()
            && self.ring.available_read() == 0
        {
            return Ok(());
        }

        let mut decoder = self
            .decoder
            .lock()
            .map_err(|_| AudioError::MusicDecoderPoisoned)?;

        let mut scratch = self
            .decoder_scratch
            .lock()
            .map_err(|_| AudioError::MusicDecoderPoisoned)?;

        let target_samples =
            STREAM_BUFFER_FRAMES
                * OUTPUT_CHANNELS as usize;

        while self.ring.available_read() < target_samples {
            let writable_samples =
                self.ring.available_write();

            let requested_frames =
                (writable_samples
                    / OUTPUT_CHANNELS as usize)
                    .min(DECODE_CHUNK_FRAMES);

            if requested_frames == 0 {
                break;
            }

            let requested_samples =
                requested_frames
                    * OUTPUT_CHANNELS as usize;

            let mut frames =
                miniaudio::FramesMut::wrap(
                    &mut scratch[..requested_samples],
                    Format::F32,
                    OUTPUT_CHANNELS,
                );

            let frames_read =
                decoder
                    .read_pcm_frames(&mut frames)
                    as usize;

            if frames_read > 0 {
                let samples_read =
                    frames_read
                        * OUTPUT_CHANNELS as usize;

                let written =
                    self.ring.write(
                        &scratch[..samples_read]
                    );

                if written < samples_read {
                    break;
                }
            }

            /*
             * Decoder reached EOF.
             */
            if frames_read < requested_frames {
                if self.looping() {
                    decoder
                        .seek_to_pcm_frame(0)
                        .map_err(|error| {
                            AudioError::DecoderFailed(
                                error.to_string()
                            )
                        })?;

                    self.finished.store(
                        false,
                        Ordering::Release,
                    );

                    /*
                     * Empty/corrupt stream:
                     * seeking doesn't make progress.
                     */
                    if frames_read == 0 {
                        self.finished.store(
                            true,
                            Ordering::Release,
                        );

                        break;
                    }
                } else {
                    /*
                     * We don't stop immediately because
                     * there may still be PCM in the ring.
                     */
                    self.finished.store(
                        true,
                        Ordering::Release,
                    );

                    break;
                }
            }
        }

        Ok(())
    }

    /*
     * REALTIME PATH
     *
     * No Mutex.
     * No RwLock.
     * No Vec allocation.
     * No file IO.
     * No decoder call.
     */
    pub fn mix_into(
        &self,
        output: &mut [f32],
        gain: f32,
    ) {
        if self.state() != PlaybackState::Playing
            || gain <= 0.0
        {
            return;
        }

        /*
         * Two render_into() calls may happen concurrently.
         *
         * Never block the realtime callback.
         */
        if self
            .mix_guard
            .compare_exchange(
                false,
                true,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            return;
        }

        let _guard =
            MixGuard {
                flag: &self.mix_guard,
            };

        let volume = self.volume();

        /*
         * Fixed stack buffer.
         */
        let mut scratch = [0.0f32; 512];

        let mut output_position = 0usize;

        while output_position < output.len() {
            let chunk_len =
                (output.len() - output_position)
                    .min(scratch.len());

            let read =
                self.ring.read(
                    &mut scratch[..chunk_len]
                );

            if read == 0 {
                break;
            }

            for i in 0..read {
                output[
                    output_position + i
                ] += scratch[i]
                    * volume
                    * gain;
            }

            output_position += read;

            self.cursor_frames.fetch_add(
                read / OUTPUT_CHANNELS as usize,
                Ordering::Relaxed,
            );
        }

        /*
         * EOF + ring drained => playback is really finished.
         */
        if self.finished.load(Ordering::Acquire)
            && self.ring.available_read() == 0
        {
            self.set_state(
                PlaybackState::Stopped
            );
        }
    }
}