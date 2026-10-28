// SPDX-FileCopyrightText: 2026 Nathan Flachat
// SPDX-License-Identifier: MIT

use crate::audio::bus::{AudioBus, BusKind};
use crate::audio::decoder::{AudioDecoder, DecodedAudio};
use crate::audio::error::{AudioError, AudioResult};

use crate::audio::mixer::AudioMixer;
use crate::audio::music::MusicStream;
use crate::audio::snapshot::Snapshot;
use crate::audio::sound::SoundVoice;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone)]
// Each queued command applies to every active playback; retain that explicit scope.
#[allow(clippy::enum_variant_names)]
pub(crate) enum AudioCommand {
    PauseAll,
    ResumeAll,
    StopAll,
}

pub(crate) struct AudioRuntime {
    master_bus: AudioBus,
    music_bus: AudioBus,
    sfx_bus: AudioBus,
    sound_cache: RwLock<HashMap<PathBuf, Arc<DecodedAudio>>>,
    active_sound_voices: Mutex<Vec<Arc<SoundVoice>>>,
    active_music_streams: Mutex<Vec<Arc<MusicStream>>>,
    pub(crate) sound_voice_snapshot: Snapshot<SoundVoice>,
    pub(crate) music_stream_snapshot: Snapshot<MusicStream>,
    command_queue: Mutex<Vec<AudioCommand>>,
    next_voice_id: AtomicU64,
    max_sound_voices: AtomicUsize,
}

impl AudioRuntime {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            master_bus: AudioBus::new(),
            music_bus: AudioBus::new(),
            sfx_bus: AudioBus::new(),
            sound_cache: RwLock::new(HashMap::new()),
            active_sound_voices: Mutex::new(Vec::new()),
            active_music_streams: Mutex::new(Vec::new()),
            sound_voice_snapshot: Snapshot::new(),
            music_stream_snapshot: Snapshot::new(),
            command_queue: Mutex::new(Vec::new()),
            next_voice_id: AtomicU64::new(1),
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

    fn publish_sound_voice_snapshot(&self, voices: &[Arc<SoundVoice>]) {
        self.sound_voice_snapshot.publish(voices.to_vec());
    }

    fn publish_music_stream_snapshot(&self, streams: &[Arc<MusicStream>]) {
        self.music_stream_snapshot.publish(streams.to_vec());
    }

    pub(crate) fn next_voice_id(&self) -> u64 {
        self.next_voice_id.fetch_add(1, Ordering::Relaxed)
    }

    #[must_use]
    pub(crate) fn max_sound_voices(&self) -> usize {
        self.max_sound_voices.load(Ordering::Relaxed)
    }

    pub(crate) fn set_max_sound_voices(&self, max: usize) -> AudioResult<()> {
        self.max_sound_voices.store(max.max(1), Ordering::Relaxed);

        let mut voices = self
            .active_sound_voices
            .lock()
            .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

        self.enforce_voice_limit_locked(&mut voices);

        self.publish_sound_voice_snapshot(&voices);

        Ok(())
    }

    pub(crate) fn enqueue(&self, command: AudioCommand) -> AudioResult<()> {
        let mut queue = self
            .command_queue
            .lock()
            .map_err(|_| AudioError::CommandQueuePoisoned)?;

        queue.push(command);

        Ok(())
    }

    pub(crate) fn apply_commands(&self) -> AudioResult<()> {
        let commands = {
            let mut queue = self
                .command_queue
                .lock()
                .map_err(|_| AudioError::CommandQueuePoisoned)?;
            std::mem::take(&mut *queue)
        };

        for command in commands {
            match command {
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
            }
        }
        Ok(())
    }

    pub(crate) fn get_or_decode_audio<P>(&self, path: P) -> AudioResult<Arc<DecodedAudio>>
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

    pub(crate) fn register_sound_voice(&self, voice: Arc<SoundVoice>) -> AudioResult<()> {
        let mut voices = self
            .active_sound_voices
            .lock()
            .map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;

        voices.push(voice);
        self.enforce_voice_limit_locked(&mut voices);
        self.publish_sound_voice_snapshot(&voices);

        Ok(())
    }

    pub(crate) fn register_music_stream(&self, stream: Arc<MusicStream>) -> AudioResult<()> {
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

            streams.retain(|stream| !stream.is_stopped());

            if streams.len() != old_len {
                self.publish_music_stream_snapshot(&streams);
            }
        }

        self.sound_voice_snapshot.reclaim();
        self.music_stream_snapshot.reclaim();
        Ok(())
    }

    pub(crate) fn pump_music_streams(&self) -> AudioResult<()> {
        let streams = self
            .active_music_streams
            .lock()
            .map_err(|_| AudioError::ActiveMusicStreamsPoisoned)?;

        for stream in streams.iter() {
            stream.pump()?;
        }

        Ok(())
    }

    pub(crate) fn mix_into(&self, output: &mut [f32]) {
        AudioMixer::mix(self, output);
    }

    #[must_use]
    pub(crate) fn bus(&self, bus: BusKind) -> &AudioBus {
        match bus {
            BusKind::Master => &self.master_bus,
            BusKind::Music => &self.music_bus,
            BusKind::Sfx => &self.sfx_bus,
        }
    }

    pub(crate) fn force_stop_all(&self) -> AudioResult<()> {
        self.enqueue(AudioCommand::StopAll)
    }

    pub(crate) fn force_pause_all(&self) -> AudioResult<()> {
        self.enqueue(AudioCommand::PauseAll)
    }

    pub(crate) fn force_resume_all(&self) -> AudioResult<()> {
        self.enqueue(AudioCommand::ResumeAll)
    }

    pub(crate) fn tick(&self) -> AudioResult<()> {
        self.apply_commands()?;
        self.pump_music_streams()?;
        self.cleanup()?;
        Ok(())
    }
}

impl Default for AudioRuntime {
    fn default() -> Self {
        Self::new()
    }
}
