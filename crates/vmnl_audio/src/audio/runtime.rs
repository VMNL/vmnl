////////////////////////////////////////////////////////////////////////////////
use crate::audio::decoder::{AudioDecoder, DecodedAudio};
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////
use crate::audio::{AudioBus, AudioError, AudioMixer, BusKind, MusicStream, SoundVoice};

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone)]
pub enum AudioCommand {
    SetMasterVolume(f32),
    SetBusVolume(BusKind, f32),
    MuteBus(BusKind),
    UnmuteBus(BusKind),
    PauseAll,
    ResumeAll,
    StopAll,
    SetMaxVoices(usize),
}

pub struct AudioRuntime {
    pub master_bus: AudioBus,
    pub music_bus: AudioBus,
    pub sfx_bus: AudioBus,
    pub sound_cache: RwLock<HashMap<PathBuf, Arc<DecodedAudio>>>,
    pub active_sound_voices: RwLock<Vec<Arc<SoundVoice>>>,
    pub active_music_streams: RwLock<Vec<Arc<MusicStream>>>,
    pub command_queue: Mutex<Vec<AudioCommand>>,
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
            active_sound_voices: RwLock::new(Vec::new()),
            active_music_streams: RwLock::new(Vec::new()),
            command_queue: Mutex::new(Vec::new()),
            next_voice_id: AtomicU64::new(1),
            next_stream_id: AtomicU64::new(1),
            max_sound_voices: AtomicUsize::new(64),
        }
    }

    pub fn next_voice_id(&self) -> u64 {
        self.next_voice_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn next_stream_id(&self) -> u64 {
        self.next_stream_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn max_sound_voices(&self) -> usize {
        self.max_sound_voices.load(Ordering::Relaxed)
    }

    pub fn set_max_sound_voices(&self, max: usize) {
        self.max_sound_voices.store(max.max(1), Ordering::Relaxed);
    }

    pub fn enqueue(&self, command: AudioCommand) {
        let mut queue = self.command_queue.lock().unwrap();
        queue.push(command);
    }

    pub fn apply_commands(&self) {
        let commands = {
            let mut queue = self.command_queue.lock().unwrap();
            std::mem::take(&mut *queue)
        };

        for command in commands {
            match command {
                AudioCommand::SetMasterVolume(volume) => self.master_bus.set_volume(volume),
                AudioCommand::SetBusVolume(BusKind::Master, volume) => {
                    self.master_bus.set_volume(volume);
                }
                AudioCommand::SetBusVolume(BusKind::Music, volume) => {
                    self.music_bus.set_volume(volume);
                }
                AudioCommand::SetBusVolume(BusKind::Sfx, volume) => self.sfx_bus.set_volume(volume),
                AudioCommand::MuteBus(BusKind::Master) => self.master_bus.mute(),
                AudioCommand::MuteBus(BusKind::Music) => self.music_bus.mute(),
                AudioCommand::MuteBus(BusKind::Sfx) => self.sfx_bus.mute(),
                AudioCommand::UnmuteBus(BusKind::Master) => self.master_bus.unmute(),
                AudioCommand::UnmuteBus(BusKind::Music) => self.music_bus.unmute(),
                AudioCommand::UnmuteBus(BusKind::Sfx) => self.sfx_bus.unmute(),
                AudioCommand::PauseAll => {
                    if let Ok(voices) = self.active_sound_voices.read() {
                        for voice in voices.iter() {
                            voice.pause();
                        }
                    }
                    if let Ok(streams) = self.active_music_streams.read() {
                        for stream in streams.iter() {
                            stream.pause();
                        }
                    }
                }
                AudioCommand::ResumeAll => {
                    if let Ok(voices) = self.active_sound_voices.read() {
                        for voice in voices.iter() {
                            voice.resume();
                        }
                    }
                    if let Ok(streams) = self.active_music_streams.read() {
                        for stream in streams.iter() {
                            stream.resume();
                        }
                    }
                }
                AudioCommand::StopAll => {
                    if let Ok(voices) = self.active_sound_voices.read() {
                        for voice in voices.iter() {
                            voice.stop();
                        }
                    }
                    if let Ok(streams) = self.active_music_streams.read() {
                        for stream in streams.iter() {
                            stream.stop();
                        }
                    }
                }
                AudioCommand::SetMaxVoices(max) => self.set_max_sound_voices(max),
            }
        }
    }

    pub fn get_or_decode_audio<P>(&self, path: P) -> Result<Arc<DecodedAudio>, AudioError>
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref().to_path_buf();

        if let Ok(cache) = self.sound_cache.read() {
            if let Some(decoded_audio) = cache.get(&path) {
                return Ok(decoded_audio.clone());
            }
        }

        let decoded_audio = Arc::new(AudioDecoder::decode_file(&path)?);

        let mut cache = self.sound_cache.write().unwrap();
        cache.insert(path, decoded_audio.clone());
        Ok(decoded_audio)
    }

    pub fn register_sound_voice(&self, voice: Arc<SoundVoice>) {
        let mut voices = self.active_sound_voices.write().unwrap();
        voices.push(voice);
        self.enforce_voice_limit_locked(&mut voices);
    }

    pub fn register_music_stream(&self, stream: Arc<MusicStream>) {
        let mut streams = self.active_music_streams.write().unwrap();
        streams.push(stream);
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

    pub fn cleanup(&self) {
        if let Ok(mut voices) = self.active_sound_voices.write() {
            voices.retain(|voice| !voice.is_stopped());
        }
        if let Ok(mut streams) = self.active_music_streams.write() {
            streams.retain(|stream| !stream.is_stopped() && !stream.is_finished());
        }
    }

    pub fn pump_music_streams(&self) {
        if let Ok(streams) = self.active_music_streams.read() {
            for stream in streams.iter() {
                stream.pump();
            }
        }
    }

    pub fn mix_into(&self, output: &mut [f32]) {
        AudioMixer::mix(self, output);
    }

    pub fn bus_gain(&self, bus: BusKind) -> f32 {
        match bus {
            BusKind::Master => self.master_bus.gain(),
            BusKind::Music => self.music_bus.gain(),
            BusKind::Sfx => self.sfx_bus.gain(),
        }
    }

    pub fn is_anything_playing(&self) -> bool {
        let voices = self
            .active_sound_voices
            .read()
            .ok()
            .is_some_and(|v| v.iter().any(|x| x.is_playing()));
        let streams = self
            .active_music_streams
            .read()
            .ok()
            .is_some_and(|s| s.iter().any(|x| x.is_playing()));
        voices || streams
    }

    pub fn force_stop_all(&self) {
        self.enqueue(AudioCommand::StopAll);
    }

    pub fn force_pause_all(&self) {
        self.enqueue(AudioCommand::PauseAll);
    }

    pub fn force_resume_all(&self) {
        self.enqueue(AudioCommand::ResumeAll);
    }
}

impl Default for AudioRuntime {
    fn default() -> Self {
        Self::new()
    }
}
