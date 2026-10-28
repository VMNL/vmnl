// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::bus::AudioBus;
use crate::audio::decoder::DecodedAudio;
use crate::audio::error::{validate_gain, AudioError, AudioResult};
use crate::audio::music::Music;
use crate::audio::runtime::AudioRuntime;
use crate::audio::sound::Sound;
use crate::audio::worker::ControlWorker;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use parking_lot::Mutex as ParkingMutex;
use std::path::Path;
use std::sync::Arc;

/// Requested output configuration, initially stereo at 44,100 Hz and gain 1.0.
///
/// The resolved device rate can differ from this request; inspect
/// [`AudioDevice::sample_rate`] after creation.
#[derive(Debug, Clone)]
pub struct AudioConfig {
    master_volume: f32,
    sample_rate: u32,
    channels: u32,
}

impl AudioConfig {
    /// Returns the requested master linear gain.
    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Sets the requested master gain.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::InvalidState`] for non-finite gains or values outside
    /// `0.0..=1.0`, without replacing the previous value.
    pub fn set_master_volume(&mut self, volume: f32) -> AudioResult<()> {
        self.master_volume = validate_gain(volume)?;
        Ok(())
    }

    /// Returns the requested output sample rate in Hz.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Sets the requested sample rate in Hz.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::InvalidState`] for zero; the previous value is preserved.
    pub fn set_sample_rate(&mut self, sample_rate: u32) -> AudioResult<()> {
        if sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "sample_rate must be greater than zero".to_owned(),
            ));
        }
        self.sample_rate = sample_rate;
        Ok(())
    }

    /// Returns the requested channel count, currently always two.
    #[must_use]
    pub fn channels(&self) -> u32 {
        self.channels
    }

    /// Sets the requested channel count.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::UnsupportedFormat`] for any value other than two,
    /// without changing the previous setting.
    pub fn set_channels(&mut self, channels: u32) -> AudioResult<()> {
        if channels != 2 {
            return Err(AudioError::UnsupportedFormat(
                "the current audio mixer requires exactly 2 output channels".to_owned(),
            ));
        }
        self.channels = channels;
        Ok(())
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sample_rate: 44_100,
            channels: 2,
        }
    }
}

/// Builder for the audio backend and its optional control thread.
#[derive(Debug, Clone)]
pub struct AudioDeviceBuilder {
    config: AudioConfig,
    threaded: bool,
}

impl Default for AudioDeviceBuilder {
    fn default() -> Self {
        Self {
            config: AudioConfig::default(),
            threaded: true,
        }
    }
}

impl AudioDeviceBuilder {
    /// Replaces the requested output configuration; validation happens in [`Self::build`].
    #[must_use]
    pub fn config(mut self, config: AudioConfig) -> Self {
        self.config = config;
        self
    }

    /// Enables the control worker. Defaults to `true`.
    /// Disabling this worker leaves backend-managed output execution active.
    /// The current default is scheduled to become opt-in before acceptance.
    #[must_use]
    pub fn thread(mut self, enabled: bool) -> Self {
        self.threaded = enabled;
        self
    }

    /// Requests a sample rate in Hz; zero is rejected when building.
    #[must_use]
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.config.sample_rate = sample_rate;
        self
    }

    /// Requests a channel count; only stereo is accepted when building.
    #[must_use]
    pub fn channels(mut self, channels: u32) -> Self {
        self.config.channels = channels;
        self
    }

    /// Requests a finite linear gain in `0.0..=1.0`; validation happens when building.
    #[must_use]
    pub fn master_volume(mut self, volume: f32) -> Self {
        self.config.master_volume = volume;
        self
    }

    /// Opens and starts the default output device with these settings.
    ///
    /// May allocate, query the backend, and start its stream and the optional
    /// control worker. See [`AudioDevice`] for rate selection and ownership.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::InvalidState`] for an invalid rate or gain,
    /// [`AudioError::UnsupportedFormat`] for unsupported channels or output formats,
    /// [`AudioError::BackendInitFailed`] for device/stream failures, or
    /// [`AudioError::ControlThreadStartFailed`] if the requested worker cannot start.
    pub fn build(self) -> AudioResult<AudioDevice> {
        AudioDevice::build(&self.config, self.threaded)
    }
}

/// Owns the audio runtime and the real CPAL playback stream.
/// This type is deliberately not `Clone`; it owns external backend resources.
///
/// Creation chooses the default CPAL host and output device. It requires stereo,
/// ranks supported formats as F32, I16, U16, I32, U32, F64, then I8/U8, and clamps
/// the requested rate to a supported range. Format rank takes precedence over
/// distance from the requested rate. Inspect the actual rate after creation.
///
/// Dropping the device joins its control worker, then destroys its output stream.
/// Joining may block on control-side decoding or file I/O. Resources and handles
/// can retain runtime data after this owner is dropped, but cannot keep playback
/// running without the device. No graphics context is initialized.
pub struct AudioDevice {
    // Drop the stream's callback captures before releasing the owner's runtime.
    _stream: cpal::Stream,
    runtime: Arc<AudioRuntime>,
    sample_rate: u32,
    channels: u32,
    worker: Option<ControlWorker>,
    background_error: Arc<ParkingMutex<Option<String>>>,
}

impl AudioDevice {
    /// Returns a builder with the current default configuration and worker enabled.
    #[must_use]
    pub fn builder() -> AudioDeviceBuilder {
        AudioDeviceBuilder::default()
    }

    /// Creates and starts a device with the automatic control thread enabled.
    ///
    /// # Errors
    ///
    /// Reports the same configuration, backend, and worker-start errors as
    /// [`AudioDeviceBuilder::build`].
    pub fn new(config: AudioConfig) -> AudioResult<Self> {
        AudioDeviceBuilder::default().config(config).build()
    }

    // Keep backend/worker creation and its fallible cleanup order together.
    #[allow(clippy::too_many_lines)]
    fn build(config: &AudioConfig, threaded: bool) -> AudioResult<Self> {
        validate_gain(config.master_volume)?;
        if config.sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "sample_rate must be greater than zero".to_owned(),
            ));
        }
        if config.channels != 2 {
            return Err(AudioError::UnsupportedFormat(
                "the current audio mixer requires exactly 2 output channels".to_owned(),
            ));
        }

        let runtime = Arc::new(AudioRuntime::new());
        runtime.master_bus().set_volume(config.master_volume)?;

        let host = cpal::default_host();
        let output_device = host.default_output_device().ok_or_else(|| {
            AudioError::BackendInitFailed("no default output audio device is available".to_owned())
        })?;

        // Choose a supported stereo format. Prefer F32; support common integer
        // and F64 formats too. If the requested rate is outside a supported
        // range, choose the nearest boundary and expose the actual rate below.
        let supported_configs = output_device
            .supported_output_configs()
            .map_err(|error| AudioError::BackendInitFailed(error.to_string()))?;

        let selected = supported_configs
            .filter(|range| range.channels() == 2)
            .filter_map(|range| {
                let rank = sample_format_rank(range.sample_format())?;
                let minimum = range.min_sample_rate();
                let maximum = range.max_sample_rate();
                let actual_rate = config.sample_rate.clamp(minimum, maximum);
                let rate_distance = config.sample_rate.abs_diff(actual_rate);
                Some((rank, rate_distance, range, actual_rate))
            })
            .min_by_key(|(rank, distance, _, _)| (*rank, *distance))
            .ok_or_else(|| {
                AudioError::UnsupportedFormat(
                    "the default audio device has no supported stereo output format".to_owned(),
                )
            })?;

        let (_, _, supported_config, actual_sample_rate) = selected;
        let sample_format = supported_config.sample_format();
        let output_config = supported_config
            .with_sample_rate(actual_sample_rate)
            .config();

        let background_error = Arc::new(ParkingMutex::new(None::<String>));
        let stream = match sample_format {
            SampleFormat::F32 => build_output_stream::<f32>(
                &output_device,
                output_config,
                runtime.clone(),
                background_error.clone(),
            ),
            SampleFormat::F64 => build_output_stream::<f64>(
                &output_device,
                output_config,
                runtime.clone(),
                background_error.clone(),
            ),
            SampleFormat::I8 => build_output_stream::<i8>(
                &output_device,
                output_config,
                runtime.clone(),
                background_error.clone(),
            ),
            SampleFormat::I16 => build_output_stream::<i16>(
                &output_device,
                output_config,
                runtime.clone(),
                background_error.clone(),
            ),
            SampleFormat::U8 => build_output_stream::<u8>(
                &output_device,
                output_config,
                runtime.clone(),
                background_error.clone(),
            ),
            SampleFormat::U16 => build_output_stream::<u16>(
                &output_device,
                output_config,
                runtime.clone(),
                background_error.clone(),
            ),
            SampleFormat::I32 => build_output_stream::<i32>(
                &output_device,
                output_config,
                runtime.clone(),
                background_error.clone(),
            ),
            SampleFormat::U32 => build_output_stream::<u32>(
                &output_device,
                output_config,
                runtime.clone(),
                background_error.clone(),
            ),
            unsupported => {
                return Err(AudioError::UnsupportedFormat(format!(
                    "CPAL output sample format {unsupported:?} is not supported by this backend"
                )));
            }
        }
        .map_err(|error| AudioError::BackendInitFailed(error.to_string()))?;

        // Spawn the non-realtime worker before starting playback. If stream.play
        // fails, dropping the worker stops and joins it automatically.
        let worker = if threaded {
            Some(ControlWorker::spawn(
                runtime.clone(),
                background_error.clone(),
            )?)
        } else {
            None
        };

        stream
            .play()
            .map_err(|error| AudioError::BackendInitFailed(error.to_string()))?;

        Ok(Self {
            runtime,
            sample_rate: actual_sample_rate,
            channels: 2,
            worker,
            background_error,
            _stream: stream,
        })
    }

    #[must_use]
    pub(crate) fn runtime(&self) -> Arc<AudioRuntime> {
        self.runtime.clone()
    }

    /// Returns the resolved output sample rate in Hz.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the resolved output channel count, currently two.
    #[must_use]
    pub fn channels(&self) -> u32 {
        self.channels
    }

    /// Loads and caches a complete sound, then converts it to stereo at the device rate.
    ///
    /// This performs file I/O and allocates in proportion to the decoded sound;
    /// call it outside the audio callback.
    ///
    /// # Errors
    ///
    /// Returns file/decoder errors, [`AudioError::UnsupportedFormat`] for unsupported
    /// sources, or [`AudioError::SoundCachePoisoned`] if the cache is poisoned.
    pub fn load_sound<P: AsRef<Path>>(&self, path: P) -> AudioResult<Sound> {
        Sound::from_file(self, path)
    }

    /// Validates a music file and creates a reusable streaming resource.
    ///
    /// Reads source metadata without loading the complete track. Playback opens
    /// its own decoder and preallocated PCM buffer.
    ///
    /// # Errors
    ///
    /// Returns file/decoder errors or [`AudioError::UnsupportedFormat`] for
    /// unsupported source formats or channel counts.
    pub fn load_music<P: AsRef<Path>>(&self, path: P) -> AudioResult<Music> {
        Music::from_file(self, path)
    }

    /// Sets the shared master linear gain without changing individual volumes.
    ///
    /// Zero silences both categories without pausing them. Individual and bus
    /// pauses still freeze the corresponding playback positions.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::InvalidState`] for non-finite gains or values outside
    /// `0.0..=1.0`; the previous value is preserved.
    pub fn set_master_volume(&self, volume: f32) -> AudioResult<()> {
        let volume = validate_gain(volume)?;
        self.runtime.master_bus().set_volume(volume)
    }

    /// Returns the shared master linear gain in `0.0..=1.0`.
    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.runtime.master_bus().volume()
    }

    /// Returns shared controls for all music playback on this runtime.
    #[must_use]
    pub fn music_bus(&self) -> AudioBus {
        self.runtime.music_bus()
    }

    /// Returns shared controls for all sound-effect playback on this runtime.
    #[must_use]
    pub fn sfx_bus(&self) -> AudioBus {
        self.runtime.sfx_bus()
    }

    /// Decodes/caches a sound and converts it to stereo at the device rate.
    pub(crate) fn get_or_decode_audio<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> AudioResult<Arc<DecodedAudio>> {
        let decoded = self.runtime.get_or_decode_audio(path)?;
        Ok(Arc::new(decoded.resample_to_stereo(self.sample_rate)?))
    }

    /// Advances control work only in manual mode (`thread(false)`). With a live
    /// automatic worker this is a no-op; a stopped worker is reported as an error.
    ///
    /// Manual updates may decode, perform file I/O, lock, and allocate. The caller
    /// must update often enough to keep music buffered; backend output continues
    /// independently of this call.
    /// Automatic mode never falls back to a manual cycle after worker failure.
    ///
    /// # Errors
    ///
    /// Propagates poisoned control-state locks and music decoding/file failures.
    /// Returns [`AudioError::InvalidState`] if the automatic worker has stopped,
    /// including after its diagnostic has been taken. Worker or stream diagnostics
    /// are retrieved through [`Self::take_background_error`].
    pub fn update(&self) -> AudioResult<()> {
        self.process_if_manual()
    }

    fn process_if_manual(&self) -> AudioResult<()> {
        self.check_control_worker()?;
        if self.worker.is_none() {
            self.runtime.tick()?;
        }
        Ok(())
    }

    fn check_control_worker(&self) -> AudioResult<()> {
        if let Some(worker) = &self.worker {
            worker.ensure_running()?;
        }
        Ok(())
    }

    /// Queues a stop for all active playbacks.
    ///
    /// Manual mode also runs the control cycle on the caller; automatic mode
    /// leaves application of the command to the worker.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::CommandQueuePoisoned`] if queueing fails. Manual mode
    /// also propagates control-cycle lock, decoding, or file errors.
    /// A stopped automatic worker returns [`AudioError::InvalidState`] before
    /// queueing; a concurrent failure can still leave an accepted command unapplied.
    pub fn stop_all(&self) -> AudioResult<()> {
        self.check_control_worker()?;
        self.runtime.force_stop_all()?;
        self.process_if_manual()
    }

    /// Queues a pause for all active playbacks, with scheduling as in [`Self::stop_all`].
    ///
    /// # Errors
    ///
    /// Returns queue or manual control-cycle errors as described by [`Self::stop_all`].
    pub fn pause_all(&self) -> AudioResult<()> {
        self.check_control_worker()?;
        self.runtime.force_pause_all()?;
        self.process_if_manual()
    }

    /// Queues a resume for all active playbacks, with scheduling as in [`Self::stop_all`].
    ///
    /// # Errors
    ///
    /// Returns queue or manual control-cycle errors as described by [`Self::stop_all`].
    pub fn resume_all(&self) -> AudioResult<()> {
        self.check_control_worker()?;
        self.runtime.force_resume_all()?;
        self.process_if_manual()
    }

    /// Sets the active sound limit, clamping zero to one; the initial limit is 64.
    ///
    /// Immediately stops the oldest excess voices and republishes the snapshot
    /// on the caller. May lock and allocate; never call it from the audio callback.
    ///
    /// # Errors
    ///
    /// Returns [`AudioError::ActiveSoundVoicesPoisoned`] if the voice list is poisoned.
    pub fn set_max_sound_voices(&self, max: usize) -> AudioResult<()> {
        // This changes active voices immediately on the caller/control thread.
        self.runtime.set_max_sound_voices(max)
    }

    /// Returns and clears the last error reported by the control worker or
    /// CPAL's stream-error callback. Errors are not raised from the realtime
    /// data callback.
    ///
    /// This is one last-error slot, not an error history: a later report can
    /// overwrite an unread diagnostic. Taking it does not restart a failed worker.
    /// Control-cycle panics are reported when unwinding is enabled; aborts are not caught.
    #[must_use]
    pub fn take_background_error(&self) -> Option<String> {
        self.background_error.lock().take()
    }
}

impl Drop for AudioDevice {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.as_mut() {
            worker.stop();
        }
    }
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

fn build_output_stream<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    runtime: Arc<AudioRuntime>,
    background_error: Arc<ParkingMutex<Option<String>>>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample + FromSample<f32>,
{
    device.build_output_stream(
        config,
        move |output: &mut [T], _callback_info: &cpal::OutputCallbackInfo| {
            render_output(&runtime, output);
        },
        move |error| {
            *background_error.lock() = Some(format!("CPAL output stream error: {error}"));
        },
        None,
    )
}

/// Converts the mixer's f32 output to the sample format selected by CPAL.
/// The fixed stack buffer avoids allocation in the realtime callback.
fn render_output<T>(runtime: &AudioRuntime, output: &mut [T])
where
    T: Sample + FromSample<f32>,
{
    const SCRATCH_SAMPLES: usize = 512;
    let mut scratch = [0.0f32; SCRATCH_SAMPLES];
    let mut offset = 0usize;

    while offset < output.len() {
        let remaining = output.len() - offset;
        let mut count = remaining.min(SCRATCH_SAMPLES);
        // The mixer consumes stereo interleaved samples in pairs.
        if !count.is_multiple_of(2) {
            count -= 1;
        }
        if count == 0 {
            output[offset] = T::EQUILIBRIUM;
            offset += 1;
            continue;
        }

        runtime.mix_into(&mut scratch[..count]);
        for (destination, source) in output[offset..offset + count]
            .iter_mut()
            .zip(scratch[..count].iter().copied())
        {
            *destination = T::from_sample(source.clamp(-1.0, 1.0));
        }
        offset += count;
    }
}
