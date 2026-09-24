////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use crate::audio::bus::AudioBus;
use crate::audio::decoder::DecodedAudio;
use crate::audio::error::{validate_gain, AudioError, AudioResult};
use crate::audio::music::Music;
use crate::audio::runtime::{AudioCommand, AudioRuntime};
use crate::audio::sound::Sound;

use miniaudio::{DeviceConfig as MiniaudioDeviceConfig, DeviceType, Format};
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
                "sample_rate must be greater than zero".to_string(),
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
                "the current mixer/backend requires exactly 2 output channels".to_string(),
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

pub struct AudioDevice {
    runtime: Arc<AudioRuntime>,
    sample_rate: u32,
    channels: u32,
    _backend: miniaudio::Device,
}

impl AudioDevice {
    pub fn new(config: AudioConfig) -> AudioResult<Self> {
        if config.channels != 2 {
            return Err(AudioError::UnsupportedFormat(
                "the current mixer/backend requires exactly 2 output channels".to_string(),
            ));
        }

        if config.sample_rate == 0 {
            return Err(AudioError::InvalidState(
                "sample_rate must be greater than zero".to_string(),
            ));
        }

        let runtime = Arc::new(AudioRuntime::new());

        runtime
            .master_bus()
            .set_volume(config.master_volume())?;

        /*
         * Create the real OS playback device.
         *
         * The callback is executed by miniaudio on the realtime audio thread.
         * It MUST NOT call update(), lock Mutex/RwLock, decode files or allocate.
         */
        let runtime_for_callback = runtime.clone();

        let mut backend_config =
            MiniaudioDeviceConfig::new(DeviceType::Playback);

        backend_config.set_sample_rate(config.sample_rate());

        backend_config
            .playback_mut()
            .set_format(Format::F32);

        backend_config
            .playback_mut()
            .set_channels(config.channels());

        backend_config.set_data_callback(
            move |_device, output, _input| {
                let output_samples = output.as_samples_mut::<f32>();

                runtime_for_callback.mix_into(output_samples);
            },
        );

        let backend = miniaudio::Device::new(None, &backend_config)
            .map_err(|error| {
                AudioError::BackendInitFailed(error.to_string())
            })?;

        backend.start().map_err(|error| {
            AudioError::BackendInitFailed(error.to_string())
        })?;

        Ok(Self {
            runtime,
            sample_rate: config.sample_rate(),
            channels: config.channels(),
            _backend: backend,
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

    pub fn load_sound<P>(&self, path: P) -> AudioResult<Sound>
    where
        P: AsRef<Path>,
    {
        Sound::from_file(self, path)
    }

    pub fn load_music<P>(&self, path: P) -> AudioResult<Music>
    where
        P: AsRef<Path>,
    {
        Music::from_file(self, path)
    }

    pub fn set_master_volume(&self, volume: f32) -> AudioResult<()> {
        let volume = validate_gain(volume)?;

        self.runtime
            .enqueue(AudioCommand::SetMasterVolume(volume))?;

        self.update()?;

        Ok(())
    }

    #[must_use]
    pub fn master_volume(&self) -> f32 {
        self.runtime.master_bus().volume()
    }

    #[must_use]
    pub fn music_bus(&self) -> AudioBus {
        self.runtime.music_bus().clone()
    }

    #[must_use]
    pub fn sfx_bus(&self) -> AudioBus {
        self.runtime.sfx_bus().clone()
    }

    /*
     * Sounds are still cached/decoded entirely because SoundVoice needs
     * random access to PCM samples.
     *
     * Conversion to stereo + resampling happen OUTSIDE the realtime callback.
     */
    pub(crate) fn get_or_decode_audio<P>(
        &self,
        path: P,
    ) -> AudioResult<Arc<DecodedAudio>>
    where
        P: AsRef<Path>,
    {
        let decoded = self.runtime.get_or_decode_audio(path)?;

        let output = decoded.resample_to_stereo(self.sample_rate)?;

        Ok(Arc::new(output))
    }

    /*
     * Control thread only.
     *
     * This function is intentionally NOT called from render_into().
     */
    pub fn update(&self) -> AudioResult<()> {
        self.runtime.apply_commands()?;
        self.runtime.pump_music_streams()?;
        self.runtime.cleanup()?;

        Ok(())
    }

    /*
     * Manual rendering API.
     *
     * No update(), no Mutex/RwLock and no allocation here.
     */
    pub fn render_into(&self, output: &mut [f32]) -> AudioResult<()> {
        if output.len() % self.channels as usize != 0 {
            return Err(AudioError::InvalidState(
                "output buffer length must be a multiple of the device channel count"
                    .to_string(),
            ));
        }

        self.runtime.mix_into(output);

        Ok(())
    }

    pub fn stop_all(&self) -> AudioResult<()> {
        self.runtime.force_stop_all()?;
        self.update()?;

        Ok(())
    }

    pub fn pause_all(&self) -> AudioResult<()> {
        self.runtime.force_pause_all()?;
        self.update()?;

        Ok(())
    }

    pub fn resume_all(&self) -> AudioResult<()> {
        self.runtime.force_resume_all()?;
        self.update()?;

        Ok(())
    }

    pub fn set_max_sound_voices(&self, max: usize) -> AudioResult<()> {
        self.runtime
            .enqueue(AudioCommand::SetMaxVoices(max))?;

        self.update()?;

        Ok(())
    }
}