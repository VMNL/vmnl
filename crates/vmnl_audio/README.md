# vmnl_audio

The [audio acceptance contract](../../docs/audio.md) defines the target for #84 and its remaining
implementation gaps. The examples below describe the current implementation, including its
automatic control-worker default; that default must become opt-in before acceptance.

`vmnl_audio` provides stereo sound effects, streamed music, playback handles and independent master/music/SFX volume controls. The public API intentionally hides the runtime, mixer, decoder state, voice objects and PCM ring buffer.

## Quick start — automatic mode

```rust
use vmnl_audio::{AudioDevice, AudioResult};
use std::time::Duration;

fn main() -> AudioResult<()> {
    let audio = AudioDevice::builder().build()?;
    let sound = audio.load_sound("assets/explosion.wav")?;
    let _handle = sound.play()?;

    // No audio.update() in automatic mode.
    std::thread::sleep(Duration::from_secs(2));
    Ok(())
}
```

The default is `.thread(true)`. The CPAL callback mixes prepared samples; a separate control
thread handles command processing, music buffer refill and cleanup. See the canonical
[snapshot ownership and evidence limits](../../docs/audio.md#snapshot-ownership) before relying
on realtime behaviour.
Generate the item-level API documentation with `cargo doc -p vmnl_audio --no-deps` for current
defaults, errors, ownership, and known playback limitations.

## Music

```rust
use vmnl_audio::{AudioDevice, AudioResult, MusicPlayConfig};
use std::time::Duration;

fn main() -> AudioResult<()> {
    let audio = AudioDevice::builder()
        .sample_rate(48_000)
        .channels(2)
        .thread(true)
        .build()?;

    let music = audio.load_music("assets/music.ogg")?;
    let handle = music.play_with_config(MusicPlayConfig {
        volume: 0.6,
        looping: true,
    })?;

    handle.set_volume(0.4)?;
    std::thread::sleep(Duration::from_secs(5));
    handle.pause();
    Ok(())
}
```

Music is decoded progressively into a preallocated ring buffer. A decoder reaching EOF does not stop playback while decoded samples remain in that buffer. With looping enabled, the decoder is reopened on the control thread.

## Manual control mode

```rust
use vmnl_audio::{AudioDevice, AudioResult};
use std::time::Duration;

fn main() -> AudioResult<()> {
    let audio = AudioDevice::builder()
        .thread(false)
        .build()?;

    for _ in 0..100 {
        // Update game/application state here.
        audio.update()?;
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}
```

`thread(false)` disables only the library's control worker. The CPAL output stream still runs on its backend-managed realtime thread, so the host application must call `update()` to refill music buffers and perform cleanup.

## Playback control

```rust
use vmnl_audio::{AudioDevice, AudioResult, SoundPlayConfig};

fn playback_example() -> AudioResult<()> {
    let audio = AudioDevice::builder().build()?;
    let sound = audio.load_sound("assets/shot.wav")?;
    let handle = sound.play_with_config(SoundPlayConfig {
        volume: 0.8,
        looping: false,
    })?;

    handle.pause();
    handle.resume();
    handle.set_volume(0.5)?;
    handle.stop();

    audio.music_bus().set_volume(0.7)?;
    audio.sfx_bus().set_volume(1.0)?;
    audio.pause_all()?;
    audio.resume_all()?;
    audio.stop_all()?;
    Ok(())
}
```

Gain values must be finite and between `0.0` and `1.0`. A call to `set_max_sound_voices()` takes effect immediately and stops the oldest voices when needed.

## Supported source formats and constraints

The incremental decoder handles WAV, MP3, OGG/Vorbis and FLAC. Sources must be mono or stereo; the mixer output is stereo. Sound effects are fully decoded and cached in memory, while music is decoded incrementally. Both paths use the same codec adapters, channel conversion and resampling implementation.

The output device is selected through CPAL. Its actual supported sample rate is exposed by `AudioDevice::sample_rate()`. `take_background_error()` retrieves and clears the last asynchronous control-worker or CPAL stream error.

## Architecture

- **CPAL output callback:** only mixes prepared samples into the device output.
- **Control worker (`thread(true)`):** processes global commands, pumps music streams and removes stopped streams.
- **StreamingDecoder:** reads supported formats in chunks and resamples/converts to stereo outside the callback.
- **Sound:** accumulates decoded samples for fast repeated playback.
- **Music:** progressively fills a bounded SPSC ring buffer.

Internal types such as `AudioRuntime`, `AudioMixer`, `MusicStream`, `SoundVoice`, `StreamingDecoder` and `AudioCommand` are not part of the public API.
