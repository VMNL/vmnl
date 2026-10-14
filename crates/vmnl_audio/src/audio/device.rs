////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
////////////////////////////////////////////////////////////////////////////////
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

#[derive(Debug, Clone)]
pub struct AudioConfig {
    master_volume: f32,
    sample_rate: u32,
    channels: u32,
}

impl AudioConfig {
    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn set_master_volume(&mut self, volume: f32) -> AudioResult<()> {
        self.master_volume = validate_gain(volume)?;
        Ok(())
    }

    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) -> AudioResult<()> {
        if sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "sample_rate must be greater than zero".to_owned(),
            ));
        }
        self.sample_rate = sample_rate;
        Ok(())
    }

    #[must_use]
    pub fn channels(&self) -> u32 {
        self.channels
    }

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
    #[must_use]
    pub fn config(mut self, config: AudioConfig) -> Self {
        self.config = config;
        self
    }

    /// Enables the control worker. Defaults to `true`.
    /// The CPAL output callback always has its own realtime execution thread.
    #[must_use]
    pub fn thread(mut self, enabled: bool) -> Self {
        self.threaded = enabled;
        self
    }

    #[must_use]
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.config.sample_rate = sample_rate;
        self
    }

    #[must_use]
    pub fn channels(mut self, channels: u32) -> Self {
        self.config.channels = channels;
        self
    }

    #[must_use]
    pub fn master_volume(mut self, volume: f32) -> Self {
        self.config.master_volume = volume;
        self
    }

    pub fn build(self) -> AudioResult<AudioDevice> {
        AudioDevice::build(self.config, self.threaded)
    }
}

/// Owns the audio runtime and the real CPAL playback stream.
/// This type is deliberately not `Clone`; it owns external backend resources.
pub struct AudioDevice {
    runtime: Arc<AudioRuntime>,
    sample_rate: u32,
    channels: u32,
    worker: Option<ControlWorker>,
    background_error: Arc<ParkingMutex<Option<String>>>,
    _stream: cpal::Stream,
}

impl AudioDevice {
    #[must_use]
    pub fn builder() -> AudioDeviceBuilder {
        AudioDeviceBuilder::default()
    }

    /// Creates a device with the automatic control thread enabled.
    pub fn new(config: AudioConfig) -> AudioResult<Self> {
        AudioDeviceBuilder::default().config(config).build()
    }

    fn build(config: AudioConfig, threaded: bool) -> AudioResult<Self> {
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

    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[must_use]
    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn load_sound<P: AsRef<Path>>(&self, path: P) -> AudioResult<Sound> {
        Sound::from_file(self, path)
    }

    pub fn load_music<P: AsRef<Path>>(&self, path: P) -> AudioResult<Music> {
        Music::from_file(self, path)
    }

    pub fn set_master_volume(&self, volume: f32) -> AudioResult<()> {
        let volume = validate_gain(volume)?;
        self.runtime.master_bus().set_volume(volume)
    }

    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.runtime.master_bus().volume()
    }

    #[must_use]
    pub fn music_bus(&self) -> AudioBus {
        self.runtime.music_bus()
    }

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

    /// Advances control work only in manual mode (`thread(false)`). In automatic
    /// mode this is a harmless no-op; the worker owns the control cycle.
    pub fn update(&self) -> AudioResult<()> {
        self.process_if_manual()
    }

    fn process_if_manual(&self) -> AudioResult<()> {
        if self.worker.is_none() {
            self.runtime.tick()?;
        }
        Ok(())
    }

    pub fn stop_all(&self) -> AudioResult<()> {
        self.runtime.force_stop_all()?;
        self.process_if_manual()
    }

    pub fn pause_all(&self) -> AudioResult<()> {
        self.runtime.force_pause_all()?;
        self.process_if_manual()
    }

    pub fn resume_all(&self) -> AudioResult<()> {
        self.runtime.force_resume_all()?;
        self.process_if_manual()
    }

    pub fn set_max_sound_voices(&self, max: usize) -> AudioResult<()> {
        // This changes active voices immediately on the caller/control thread.
        self.runtime.set_max_sound_voices(max)
    }

    /// Returns and clears the last error reported by the control worker or
    /// CPAL's stream-error callback. Errors are not raised from the realtime
    /// data callback.
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
        SampleFormat::I8 => 6,
        SampleFormat::U8 => 6,
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
