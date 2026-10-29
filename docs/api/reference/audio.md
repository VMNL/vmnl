# Audio

## Public path and maturity

`vmnl::audio`, enabled by the `audio` feature. All audio types are experimental.
The independently usable `vmnl_audio` crate exports the same types; the facade adds no wrapper.
For an audio-only development client, adjust this checkout path to your client location:

```toml
vmnl = { path = "../vmnl/crates/vmnl", default-features = false, features = ["audio"] }
```

VMNL has no published release yet.

## Purpose and use cases

Cached sound effects and streamed music with independent device ownership, shared playback
handles, and master/music/SFX controls. Audio-only clients need no graphics `Context`, Vulkan,
GLFW, or display dependency. Playback requires a usable audio output device.

## Public API

All paths below are under `vmnl::audio`. Exact signatures and item-local contracts are in
Rustdoc, linked from the [symbol index](../appendices/public_symbol_index.md).

| Type | Fields, variants, and inherent methods |
|---|---|
| `AudioConfig` | `default`, `master_volume`, `set_master_volume`, `sample_rate`, `set_sample_rate`, `channels`, `set_channels` |
| `AudioDeviceBuilder` | `default`, `config`, `thread`, `control_mode`, `sample_rate`, `channels`, `master_volume`, `build` |
| `AudioDevice` | `builder`, `new`, `sample_rate`, `channels`, `control_mode`, `load_sound`, `load_music`, `set_master_volume`, `master_volume`, `music_bus`, `sfx_bus`, `update`, `stop_all`, `pause_all`, `resume_all`, `set_max_sound_voices`, `take_background_error` |
| `ControlMode` | `Manual`, `Automatic` |
| `AudioBus` | `set_volume`, `volume`, `mute`, `unmute`, `is_muted`, `pause`, `resume`, `is_paused`, `gain` |
| `Sound`, `Music` | `play`, `play_with_config`, `path` |
| `SoundPlayConfig`, `MusicPlayConfig` | Public `volume` and `looping` fields; `default` |
| `SoundHandle`, `MusicHandle` | `stop`, `pause`, `resume`, `set_volume`, `set_looping`, `is_playing`, `is_paused`, `is_stopped`, `cursor_frames` |
| `PlaybackState` | `Playing = 0`, `Paused = 1`, `Stopped = 2` |
| `AudioResult<T>` | Alias for `Result<T, AudioError>` |
| `AudioError` | `BackendInitFailed`, `ControlThreadStartFailed`, `DecoderFailed`, `InvalidState`, `UnsupportedFormat`, `CommandQueuePoisoned`, `SoundCachePoisoned`, `ActiveSoundVoicesPoisoned`, `ActiveMusicStreamsPoisoned`, `MusicDecoderPoisoned`, `Io` |

`AudioConfig` and `AudioDeviceBuilder` implement `Clone`, `Debug`, and `Default`.
The playback configurations additionally implement `Copy`. `ControlMode` and `PlaybackState`
implement `Clone`, `Copy`, `Debug`, `PartialEq`, and `Eq`; `clone`, `fmt`, and `eq` are their
corresponding trait operations. Buses implement `Clone` and `Debug`; resources and handles
implement `Clone`. `AudioError` implements `Debug`, `Display` (`fmt`), `Error`, and
`From<std::io::Error>` (`from`). `AudioDevice` implements `Drop` (`drop`), without `Clone`.

## Construction, defaults, and validation

`AudioDevice::builder()` and `AudioDevice::new(config)` default to manual control. Builder and
device `control_mode()` report the chosen VMNL policy. `thread(true)` explicitly enables its
worker; `thread(false)` restores manual control. CPAL output execution remains independent.
The device getter reports configuration, not worker health; a failed automatic worker does
not switch to manual mode.

Configuration defaults are stereo, 44,100 Hz, and master gain 1.0. Setters validate before
replacing a valid value. Builder settings are validated at `build`; zero sample rate, invalid
gain, or non-stereo output fails before querying a backend. Playback configurations default
to volume 1.0 and no loop; play validates them before registration.

Creation selects the default output device. Stereo formats are ranked F32, I16, U16, I32, U32,
F64, then I8/U8. Format rank precedes rate distance; requests are clamped to supported ranges.
Inspect `sample_rate()` for the resolved rate. The backend chooses its default buffer size;
no fixed latency is requested. The initial sound voice limit is 64;
`set_max_sound_voices(0)` clamps to one and stops the oldest excess voices immediately.

## Units, coordinates, and valid ranges

Rates are Hz; output is two interleaved channels. Gains are finite linear values in `0.0..=1.0`,
not decibels. Cursor units are consumed PCM frames at the resolved device rate, cumulatively
across loops, not hardware presentation time. Underrun silence does not advance music's cursor.
`path()` returns the supplied path without canonicalization. No graphics coordinates apply.

Mute and zero gain silence output while consuming PCM and permitting EOF. Bus pause freezes
its routes without replacing individual states; master pause freezes both routes. Handle pause
freezes that instance. Unmute/volume changes preserve pauses and cannot restart stopped playback.
`gain()` returns zero for mute or pause; inspect `is_paused()` separately.

## Ownership, lifecycle, and threading

The device owns the stream and optional worker. Resources retain runtime data but cannot keep
output alive after device destruction. Resource clones share their path/runtime; each play
creates an independent instance. Handle clones control one instance; dropping a handle does
not stop it. Bus clones share controls. No public audio singleton or graphics context is used.

Manual `update()` processes commands, streaming refill and cleanup; no universal cadence is
guaranteed. A healthy automatic `update()` does not execute another cycle on the caller.
Global pause/resume/stop queue commands; manual mode also processes them immediately. Handle
controls use shared state directly. Changes do not withdraw samples already handed to a backend.

`Stopped` is terminal, including EOF; replay creates a new handle. Enabling looping on live
music permits reopening after decoder EOF, but never after stop. Disabling a sound loop finishes
its current repetition. Disabling a music loop retains prefetched PCM, including repetitions.
See the [full contract](../../audio.md) for transitions and snapshot ownership.

Device `drop` requests worker stop, wakes idle waiting and joins in-flight work before destroying
the stream and releasing runtime ownership. Joining can block on decoding/I/O, without a shutdown
deadline. Backend-specific thread/device qualification remains separate.

## Errors, panics, and failure conditions

Fallible operations use `AudioResult`. Invalid gains/rates yield `InvalidState`; unsupported
formats/channels yield `UnsupportedFormat`. Loading/play/refill can return decoder/file errors.
Backend initialization, worker creation and poisoned runtime locks have the distinct variants
listed above. `Io` wraps an I/O error and `From::from` constructs it; codec adapters can translate
their own opening/decoding failures to `DecoderFailed`.

An OGG/Vorbis chain must retain its initial sample rate and channel count. A detected change
returns `UnsupportedFormat` before PCM from that logical stream is accepted. Uncached sound
loading decodes the whole source; music loading checks initial metadata only. A change detected
during Music prefill fails `play` before registration; one encountered later fails refill.
Already mixed samples are not withdrawn. The control-mode-specific failure behavior follows below.

A failed refill stops its music instance. Manual cycles return the original failure and attempt
cleanup; later cycles can process remaining streams. Automatic control errors/unwinding panics
terminate the worker. Automatic update/global commands then return `InvalidState`, with no
manual fallback. A command racing with failure may remain unapplied. Taking the diagnostic does
not reset this state. A new device is required to recover automatic execution; old resources
remain attached to the old runtime.

`take_background_error()` takes one last-error slot shared with CPAL diagnostics; later reports
can overwrite unread ones. It is not a history. Backend errors do not restart streams/workers.
`panic=abort` is not captured. State queries do not prove audible output, a live device, or absence
of underruns.

## Allocation, transfers, synchronization, and GPU cost

Configuration/builder creation does not open a device. Device construction can allocate and
start backend execution; automatic control adds a VMNL worker. Loading/play can allocate, lock
and perform file I/O; sound caches full PCM, while music prepares a decoder/ring per instance.
Manual update also decodes/refills and cleans up on the caller. Never run these operations on
an audio data callback. There is no GPU upload or Vulkan dependency in audio-only mode.

The data callback mixes prepared PCM. Snapshot acquisition uses fixed slots and atomic ownership
transfer; returned resources are destroyed on the control side. Work still depends on frames
and active playbacks. Measured Rust heap behaviour does not qualify C/backend allocations,
error-callback formatting/locking, hardware deadlines, or native shutdown.

## Platform, Vulkan, and display constraints

Headless checks do not prove device availability. Linux compilation needs ALSA development files,
even for device-independent tests; see [build prerequisites](../../build.md). Native tests are
opt-in and distinct from listening evidence. Feature compilation does not qualify native
Windows/macOS, listening, container padding/timing, or realtime performance.

## Example and related types

Configure without opening a device:

```rust
# extern crate vmnl;
use vmnl::audio::{AudioDevice, ControlMode};
assert_eq!(AudioDevice::builder().control_mode(), ControlMode::Manual);
assert_eq!(AudioDevice::builder().thread(true).control_mode(), ControlMode::Automatic);
```

Manual streaming requires a device and a control loop:

```rust,no_run
# extern crate vmnl;
use vmnl::audio::{AudioDevice, AudioResult};
# fn main() -> AudioResult<()> {
let audio = AudioDevice::builder().build()?;
let music = audio.load_music("assets/music.ogg")?;
let handle = music.play()?;
while handle.is_playing() {
    audio.update()?;
    std::thread::sleep(std::time::Duration::from_millis(5));
}
# Ok(())
# }
```

The sleep is illustrative, not a buffer-supply guarantee. Use `thread(true)` for a VMNL worker
and monitor `take_background_error()`; the caller need not run refill cycles in that mode.
Related: [facade features](facade.md), [acceptance contract](../../audio.md), and
[test evidence](../../testing.md#audio-tests).
