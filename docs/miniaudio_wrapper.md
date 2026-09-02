Audio Architecture

The audio system is organized into several layers:

Entry point — exposes the public audio API.
Core — owns the device, decoder, and error types.
Runtime audio — manages sounds, music, voices, streams, caching, and commands.
Backend — communicates with the underlying audio backend and OS audio API.
flowchart TB

    subgraph Entry
        MOD[mod.rs]
    end

    subgraph Core
        DEVICE[device.rs]
        DECODER[decoder.rs]
        ERROR[error.rs]
    end

    subgraph Runtime_Audio
        SOUND[sound/mod.rs]
        MUSIC[music/mod.rs]
        RUNTIME[runtime.rs]
    end

    subgraph Backend
        MINI[Miniaudio Backend]
    end

    MOD --> DEVICE
    MOD --> DECODER
    MOD --> ERROR
    MOD --> SOUND
    MOD --> MUSIC

    SOUND --> DEVICE
    SOUND --> DECODER
    SOUND --> RUNTIME

    MUSIC --> DEVICE
    MUSIC --> DECODER
    MUSIC --> RUNTIME

    DEVICE --> RUNTIME
    DEVICE --> MINI

Backend Architecture

The backend is responsible for communicating with the platform audio system.

The application should not interact directly with the backend. AudioDevice acts as the abstraction layer between the public audio API and the backend implementation.

Game / ApplicationAudio APIAudioDeviceMiniaudioOS Audio API
Error handling

Backend initialization and device operations are fallible because the backend may fail to initialize, the requested device may be unavailable, or the underlying OS audio API may return an error.

Fallible functions return:

AudioResult<T>


which is equivalent to:

Result<T, AudioError>


On success, the function returns the requested value T.

On failure, it returns an AudioError describing the reason for the failure.

Errors should be propagated with ? whenever the caller can also return an AudioResult.

Sound Architecture

Sounds represent decoded audio resources that can be played multiple times.

Audio FileDecoderPCM SamplesSound ResourceSoundVoiceSoundHandle
Sound

Sound owns the decoded audio resource and keeps a reference to the shared AudioRuntime.

Loading a sound is fallible because the audio file must be decoded and the decoded data may need to be inserted into the runtime cache.

Conceptually:

Sound::from_file(...) -> AudioResult<Sound>

Success

Returns a Sound containing:

the shared AudioRuntime;
the source file path;
the decoded audio data.
Failure

The function can return an error when:

the audio file cannot be read;
the decoder cannot decode the file;
the sound cache lock is poisoned.

These failures are represented by AudioError.

Sound::play()

Playing a sound creates a new SoundVoice and registers it with the runtime.

Sound::play() -> AudioResult<SoundHandle>


The function is fallible because registering the new voice requires acquiring the runtime's active-voice lock.

Success

Returns a SoundHandle associated with the newly created SoundVoice.

The handle can be used by the caller to control the individual playback instance.

Failure

The function can return an error if the active sound voice registry cannot be locked.

For example:

self.runtime.register_sound_voice(voice.clone())?;


If the registry lock is poisoned, ActiveSoundVoicesPoisoned is returned and propagated to the caller.

Music Streaming Architecture

Music uses streaming playback rather than decoding the entire file into memory.

Music FileStreaming DecoderRolling BufferMusic InstanceMusicStreamAudio Device

Music streams decode audio progressively and maintain a rolling buffer.

This avoids loading an entire long music file into memory.

Fallible operations

Music creation and registration can be fallible when they:

open or decode the source file;
access the shared runtime;
register a new music stream;
interact with backend resources.

Functions that perform these operations should return:

AudioResult<T>


rather than silently ignoring failures.

Success

The returned value contains the newly created music resource, stream, or handle, depending on the API.

Failure

The returned AudioError identifies why the operation could not be completed.

For example, a poisoned active music stream registry is represented by:

AudioError::ActiveMusicStreamsPoisoned

Runtime Architecture

AudioRuntime is the central state manager for active audio playback.

It owns:

master, music, and SFX buses;
decoded sound cache;
active sound voices;
active music streams;
queued audio commands;
voice and stream identifiers;
maximum simultaneous sound voices.
AudioRuntimeSound CacheActive Sound VoicesActive Music StreamsCommand QueueAudio Buses
enqueue()
enqueue(command) -> AudioResult<()>


enqueue() adds an AudioCommand to the runtime command queue.

Success

Returns:

Ok(())


The command has been added to the queue.

Failure

Returns AudioError::CommandQueuePoisoned if the command queue mutex cannot be locked.

apply_commands()
apply_commands() -> AudioResult<()>


apply_commands() takes all queued commands and applies them to the runtime.

The command queue is emptied atomically using std::mem::take().

Success

Returns:

Ok(())


after all queued commands have been processed.

Failure

Returns AudioError::CommandQueuePoisoned if the command queue cannot be locked.

Commands that operate on active voices or streams currently handle their registry locks according to the runtime's chosen error policy.

get_or_decode_audio()
get_or_decode_audio(path) -> AudioResult<Arc<DecodedAudio>>


This function first checks the decoded-audio cache.

If the audio is already cached, the existing Arc<DecodedAudio> is returned.

Otherwise, the file is decoded and inserted into the cache.

Success

Returns:

Arc<DecodedAudio>


The decoded audio can be safely shared between multiple sound instances and playback voices.

Failure

The function can fail when:

the sound cache read lock is poisoned;
the audio file cannot be decoded;
the sound cache write lock is poisoned.

The corresponding AudioError is propagated to the caller.

register_sound_voice()
register_sound_voice(voice) -> AudioResult<()>


Registers a new active SoundVoice.

After registration, the runtime enforces the maximum number of simultaneous sound voices.

Success

Returns:

Ok(())


The voice has been registered.

If the maximum voice count is exceeded, the oldest voices are stopped and removed.

Failure

Returns:

AudioError::ActiveSoundVoicesPoisoned


if the active sound voice registry cannot be locked.

register_music_stream()
register_music_stream(stream) -> AudioResult<()>


Registers a new active MusicStream.

Success

Returns:

Ok(())

Failure

Returns:

AudioError::ActiveMusicStreamsPoisoned


if the active music stream registry cannot be locked.

Force commands

The runtime exposes convenience functions for queuing global playback commands:

force_stop_all() -> AudioResult<()>
force_pause_all() -> AudioResult<()>
force_resume_all() -> AudioResult<()>


These functions do not immediately modify every voice or stream.

Instead, they enqueue the corresponding command:

AudioCommand::StopAll
AudioCommand::PauseAll
AudioCommand::ResumeAll

Success

Returns:

Ok(())


when the command was successfully added to the command queue.

Failure

Returns:

AudioError::CommandQueuePoisoned


if the command queue cannot be locked.

Errors are deliberately propagated instead of being ignored:

pub fn force_stop_all(&self) -> AudioResult<()> {
    self.enqueue(AudioCommand::StopAll)
}

Non-Fallible Runtime Queries

Not every runtime operation needs to return an AudioResult.

Some functions are intentionally infallible from the public API's perspective.

For example:

max_sound_voices() -> usize
bus_gain(bus) -> f32
is_anything_playing() -> bool


These functions return a useful default/value directly rather than exposing internal synchronization failures.

For example, is_anything_playing() returns a bool because its purpose is to answer a state query, not to report runtime synchronization errors.

This distinction should be intentional:

Operations that must succeed → return AudioResult<T> when failure is possible.
State queries → may return a plain value when an internal failure can reasonably be treated as an unavailable/negative state.
Constructors/resource loading → return AudioResult<T> when I/O, decoding, backend initialization, or synchronization can fail.
Error Architecture

All audio errors are represented by AudioError.

AudioErrorBackend ErrorsDecoder ErrorsInvalid StateIO ErrorsLock ErrorsBackendInitFailedDecoderFailedInvalidStateIOCommandQueuePoisonedSoundCachePoisonedActiveSoundVoicesPoisonedActiveMusicStreamsPoisoned
Error propagation

The audio API should avoid silently ignoring errors from fallible operations.

Prefer:

let cache = self
    .sound_cache
    .read()
    .map_err(|_| AudioError::SoundCachePoisoned)?;


over:

if let Ok(cache) = self.sound_cache.read() {
    // ...
}


The first version makes the failure visible to the caller.

The second version silently continues when the lock fails.

This is particularly important for resource creation and playback operations, where silently failing can make the API appear to have succeeded when the requested operation did not actually occur.

General Error-Handling Rules

The following rules should be applied throughout the audio crate.

Return AudioResult<T> when

A function can fail because of:

file I/O;
decoding;
backend initialization or interaction;
resource registration;
poisoned synchronization primitives;
another condition represented by AudioError.

The T represents the actual successful result of the operation.

Examples:

AudioResult<Sound>
AudioResult<SoundHandle>
AudioResult<MusicStream>
AudioResult<Arc<DecodedAudio>>
AudioResult<()>

Return AudioResult<()> when

The operation has no meaningful value to return on success.

For example:

register_sound_voice(...) -> AudioResult<()>
register_music_stream(...) -> AudioResult<()>
enqueue(...) -> AudioResult<()>
apply_commands() -> AudioResult<()>
force_stop_all() -> AudioResult<()>


Ok(()) means the operation completed successfully.

Propagate errors with ?

When a fallible operation is called from another fallible function, propagate the error:

self.runtime.register_sound_voice(voice.clone())?;


rather than:

self.runtime.register_sound_voice(voice.clone());


This preserves the original AudioError and allows the caller to decide how to handle it.

Avoid unnecessary unwrap()

Runtime synchronization should not use unwrap() when lock poisoning represents a condition that the API can report.

Prefer:

.write()
.map_err(|_| AudioError::ActiveSoundVoicesPoisoned)?;


This prevents an internal synchronization failure from becoming an uncontrolled panic.

Design Principle

The audio API follows a simple rule:

If an operation can fail and the caller can reasonably handle that failure, return the failure as AudioResult<T> instead of hiding it.

The return type should communicate the contract of the function:

AudioResult<T>
        │
        ├── Ok(T)
        │     └── operation succeeded and returned T
        │
        └── Err(AudioError)
              └── operation failed and explains why


This keeps failures explicit, makes error propagation predictable, and prevents low-level runtime problems from being silently discarded.