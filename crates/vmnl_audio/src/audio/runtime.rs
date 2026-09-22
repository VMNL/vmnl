////////////////////////////////////////////////////////////////////////////////
use crate::audio::decoder::{AudioDecoder, DecodedAudio};
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use crate::audio::{AudioBus, AudioError, AudioResult, BusKind};

use crate::audio::mixer::AudioMixer;
use crate::audio::music::MusicStream;
use crate::audio::sound::SoundVoice;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use arc_swap::ArcSwap;

#[derive(Debug, Clone)]
pub(crate) enum AudioCommand {
    SetMasterVolume(f32),
    SetBusVolume(BusKind, f32),
    MuteBus(BusKind),
    UnmuteBus(BusKind),
    PauseAll,
    ResumeAll,
    StopAll,
    SetMaxVoices(usize),
}

pub(crate) struct AudioRuntime {
    master_bus: AudioBus,
    music_bus: AudioBus,
    sfx_bus: AudioBus,
    sound_cache: RwLock<HashMap<PathBuf, Arc<DecodedAudio>>>,
    active_sound_voices: Mutex<Vec<Arc<SoundVoice>>>,
    active_music_streams: Mutex<Vec<Arc<MusicStream>>>,
    sound_voice_snapshot: ArcSwap<Vec<Arc<SoundVoice>>>,
    music_stream_snapshot: ArcSwap<Vec<Arc<MusicStream>>>,
    command_queue: Mutex<Vec<AudioCommand>>,
    next_voice_id: AtomicU64,
    next_stream_id: AtomicU64,
    max_sound_voices: AtomicUsize,
}

impl AudioRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            master_bus: AudioBus::new(BusKind::Master),
            music_bus: AudioBus::new(BusKind::Music),
            sfx_bus: AudioBus::new(BusKind::Sfx),
            sound_cache: RwLock::new(HashMap::new()),
            active_sound_voices: Mutex::new(Vec::new()),
            active_music_streams: Mutex::new(Vec::new()),
            sound_voice_snapshot: ArcSwap::from_pointee(Vec::new()),
            music_stream_snapshot: ArcSwap::from_pointee(Vec::new()),
            command_queue: Mutex::new(Vec::new()),
            next_voice_id: AtomicU64::new(1),
            next_stream_id: AtomicU64::new(1),
            max_sound_voices: AtomicUsize::new(64),
        }
    }

    pub(crate) fn master_bus(&self) -> &AudioBus {
        &self.master_bus
    }

    pub(crate) fn music_bus(&self) -> AudioBus {
        self.music_bus.clone()
    }

    pub(crate) fn sfx_bus(&self) -> AudioBus {
        self.sfx_bus.clone()
    }


    fn publish_sound_voice_snapshot(&self, voices: &[Arc<SoundVoice>],) {
        self.sound_voice_snapshot
            .store(Arc::new(voices.to_vec()));
    }

    fn publish_music_stream_snapshot(&self, streams: &[Arc<MusicStream>],) {
        self.music_stream_snapshot
            .store(Arc::new(streams.to_vec()));
    }

    pub fn next_voice_id(&self) -> u64 {
        self.next_voice_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn next_stream_id(&self) -> u64 {
        self.next_stream_id.fetch_add(1, Ordering::Relaxed)
    }

    #[must_use]
    pub fn max_sound_voices(&self) -> usize {
        self.max_sound_voices.load(Ordering::Relaxed)
    }

    pub fn set_max_sound_voices(&self, max: usize) -> AudioResult<()> {
        self.max_sound_voices
            .store(max.max(1), Ordering::Relaxed);

        let mut voices = self
            .active_sound_voices
            .lock()
            .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

        self.enforce_voice_limit_locked(&mut voices);

        self.publish_sound_voice_snapshot(&voices);

        Ok(())
    }

    pub fn enqueue(&self, command: AudioCommand) -> AudioResult<()> {
        let mut queue = self
            .command_queue
            .lock()
            .map_err(|_| AudioError::CommandQueuePoisoned)?;

        queue.push(command);

        Ok(())
    }

    pub fn apply_commands(&self) -> AudioResult<()> {
        let commands = {
            let mut queue = self
                .command_queue
                .lock()
                .map_err(|_| AudioError::CommandQueuePoisoned)?;
            std::mem::take(&mut *queue)
        };

        for command in commands {
            match command {
                AudioCommand::SetMasterVolume(volume) => self.master_bus.set_volume(volume)?,
                AudioCommand::SetBusVolume(BusKind::Master, volume) => {
                    self.master_bus.set_volume(volume)?;
                }
                AudioCommand::SetBusVolume(BusKind::Music, volume) => {
                    self.music_bus.set_volume(volume)?;
                }
                AudioCommand::SetBusVolume(BusKind::Sfx, volume) => {
                    self.sfx_bus.set_volume(volume)?;
                }
                AudioCommand::MuteBus(BusKind::Master) => self.master_bus.mute(),
                AudioCommand::MuteBus(BusKind::Music) => self.music_bus.mute(),
                AudioCommand::MuteBus(BusKind::Sfx) => self.sfx_bus.mute(),
                AudioCommand::UnmuteBus(BusKind::Master) => self.master_bus.unmute(),
                AudioCommand::UnmuteBus(BusKind::Music) => self.music_bus.unmute(),
                AudioCommand::UnmuteBus(BusKind::Sfx) => self.sfx_bus.unmute(),
                AudioCommand::PauseAll => {
                    let voices = self
                        .active_sound_voices
                        .lock()
                        .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

                    for voice in voices.iter() {
                        voice.pause();
                    }

                    let streams = self
                        .active_music_streams
                        .lock()
                        .map_err(|_| AudioError::ActiveMusicStreamsPoisoned)?;

                    for stream in streams.iter() {
                        stream.pause();
                    }
                }

                AudioCommand::ResumeAll => {
                    let voices = self
                        .active_sound_voices
                        .lock()
                        .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

                    for voice in voices.iter() {
                        voice.resume();
                    }

                    let streams = self
                        .active_music_streams
                        .lock()
                        .map_err(|_| AudioError::ActiveMusicStreamsPoisoned)?;

                    for stream in streams.iter() {
                        stream.resume();
                    }
                }

                AudioCommand::StopAll => {
                    let voices = self
                        .active_sound_voices
                        .lock()
                        .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

                    for voice in voices.iter() {
                        voice.stop();
                    }

                    let streams = self
                        .active_music_streams
                        .lock()
                        .map_err(|_| AudioError::ActiveMusicStreamsPoisoned)?;

                    for stream in streams.iter() {
                        stream.stop();
                    }
                }

                AudioCommand::SetMaxVoices(max) => self.set_max_sound_voices(max)?,
            }
        }
        Ok(())
    }

    pub fn get_or_decode_audio<P>(&self, path: P) -> AudioResult<Arc<DecodedAudio>>
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref().to_path_buf();

        let cached_audio = {
            let cache = self
                .sound_cache
                .read()
                .map_err(|_| AudioError::SoundCachePoisoned)?;

            cache.get(&path).cloned()
        };

        if let Some(decoded_audio) = cached_audio {
            return Ok(decoded_audio);
        }

        let decoded_audio = Arc::new(AudioDecoder::decode_file(&path)?);

        let mut cache = self
            .sound_cache
            .write()
            .map_err(|_| AudioError::SoundCachePoisoned)?;

        cache.insert(path, decoded_audio.clone());

        Ok(decoded_audio)
    }

    pub fn register_sound_voice(&self, voice: Arc<SoundVoice>) -> AudioResult<()> {
        let mut voices = self
            .active_sound_voices
            .lock()
            .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

        voices.push(voice);
        self.enforce_voice_limit_locked(&mut voices);
        self.publish_sound_voice_snapshot(&voices);

        Ok(())
    }

    pub fn register_music_stream(&self, stream: Arc<MusicStream>) -> AudioResult<()> {
        let mut streams = self
            .active_music_streams
            .lock()
            .map_err(|_| AudioError::ActiveMusicStreamsPoisoned)?;

        streams.push(stream);
        self.publish_music_stream_snapshot(&streams);
        Ok(())
    }

    fn enforce_voice_limit_locked(&self, voices: &mut Vec<Arc<SoundVoice>>) {
        let max = self.max_sound_voices();
        while voices.len() > max {
            let mut oldest_index = None;
            let mut oldest_id = u64::MAX;
            for (index, voice) in voices.iter().enumerate() {
                let id = voice.id();
                if id < oldest_id {
                    oldest_id = id;
                    oldest_index = Some(index);
                }
            }

            if let Some(index) = oldest_index {
                voices[index].stop();
                voices.remove(index);
            } else {
                break;
            }
        }
    }

    pub(crate) fn cleanup(&self) -> AudioResult<()> {
        {
            let mut voices = self
                .active_sound_voices
                .lock()
                .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

            let old_len = voices.len();

            voices.retain(|voice| !voice.is_stopped());

            if voices.len() != old_len {
                self.publish_sound_voice_snapshot(&voices);
            }
        }

        {
            let mut streams = self
                .active_music_streams
                .lock()
                .map_err(|_| AudioError::ActiveMusicStreamsPoisoned)?;

            let old_len = streams.len();

            streams.retain(|stream| {
                !stream.is_stopped() && !stream.is_finished()
            });

            if streams.len() != old_len {
                self.publish_music_stream_snapshot(&streams);
            }
        }

        Ok(())
    }

    pub fn pump_music_streams(&self) -> AudioResult<()> {
        let streams = self
            .active_music_streams
            .lock()
            .map_err(|_| AudioError::ActiveMusicStreamsPoisoned)?;

        for stream in streams.iter() {
            stream.pump();
        }

        Ok(())
    }

    pub fn mix_into(&self, output: &mut [f32]) -> AudioResult<()> {
        AudioMixer::mix(self, output)
    }

    #[must_use]
    pub fn bus_gain(&self, bus: BusKind) -> f32 {
        match bus {
            BusKind::Master => self.master_bus.gain(),
            BusKind::Music => self.music_bus.gain(),
            BusKind::Sfx => self.sfx_bus.gain(),
        }
    }

    #[must_use]
    pub fn is_anything_playing(&self) -> AudioResult<bool> {
        let voices = self
            .active_sound_voices
            .lock()
            .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

        if voices.iter().any(|voice| !voice.is_stopped()) {
            return Ok(true);
        }

        let streams = self
            .active_music_streams
            .lock()
            .map_err(|_| AudioError::ActiveMusicStreamsPoisoned)?;

        Ok(streams
            .iter()
            .any(|stream| !stream.is_stopped() && !stream.is_finished()))
    }

    pub fn force_stop_all(&self) -> AudioResult<()> {
        self.enqueue(AudioCommand::StopAll)
    }

    pub fn force_pause_all(&self) -> AudioResult<()> {
        self.enqueue(AudioCommand::PauseAll)
    }

    pub fn force_resume_all(&self) -> AudioResult<()> {
        self.enqueue(AudioCommand::ResumeAll)
    }
}

impl Default for AudioRuntime {
    fn default() -> Self {
        Self::new()
    }
}
