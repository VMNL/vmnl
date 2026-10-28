# Audio Acceptance Contract

## Status and Scope

This document defines the acceptance target for the audio integration in PR #84. It records
the backend and facade decisions and the behavior required before that integration is complete.
It is not a claim that the current implementation already satisfies every requirement. Public
Rustdoc describes item-level behavior and current limitations; this document defines the target
against which the remaining implementation work is evaluated.

`vmnl_audio` currently uses CPAL. The facade still depends unconditionally on graphics and has no
audio feature; its optional audio integration remains to be implemented. The control worker
currently defaults to enabled. Playback transitions, mute behavior, and realtime resource
reclamation still need to be aligned and validated. The audio test files are registered as
separate [device-independent and native Cargo targets](testing.md#audio-tests); their assertions
still expose implementation gaps and are not a passing acceptance record.

## Backend and Public Boundary

CPAL remains the private device backend for #84. VMNL owns playback, mixing, decoding, error
translation, and resource lifetimes; clients use VMNL types. Replacing CPAL with miniaudio or
supporting multiple device backends is outside this PR's scope. This decision preserves the
existing implementation; it does not assert a performance advantage over miniaudio.

The domain crate remains independently usable. The facade retains graphics by default and adds
audio as an opt-in domain:

| Facade configuration | Required dependency and API boundary |
| --- | --- |
| Default features | Graphics remains available through the existing exports. Audio is disabled. |
| `graphics` only | Graphics dependency and existing graphics exports. |
| `audio`, with default features disabled | Audio through `vmnl::audio`; no graphics, Vulkan, GLFW, or graphics initialization. |
| `graphics` and `audio` | Both domains, with independent device/runtime ownership. |

The graphics dependency must therefore become optional, and its exports must be conditional on
`graphics`. Using audio does not require a graphics `Context`. Direct use of `vmnl_audio` remains
supported. Removing the facade or splitting repositories is deferred until after #84.

## Execution and Ownership

The VMNL control worker is opt-in: `.thread(false)` is the required default; `.thread(true)`
explicitly selects automatic control. The resolved mode must be inspectable. In manual mode,
the application calls `update()` to process control commands, refill music buffers, and clean up
finished playbacks. Its cadence must keep the music buffer supplied; no fixed interval is
guaranteed to work for every stream or device. Loading and control work may allocate, decode,
and perform file I/O, so they do not belong on the realtime callback.

In automatic mode, the worker owns the control cycle; `update()` must not run a second cycle
concurrently. Disabling the VMNL worker does not disable backend-managed audio execution:
[CPAL's stream contract](https://docs.rs/cpal/0.18.2/cpal/) describes platform callback execution
and threads used for blocking backends. Manual control is not synchronous rendering on the
application thread.

The device owns backend lifecycle. Its callback is the sole mixing consumer; music refill and
snapshot publication belong to the control side. Shutdown must quiesce every producer/consumer
before destroying resources it can access. A worker sleep interval is not an upper bound on
shutdown time: an in-progress control cycle can include decoding or I/O. Background failures must
remain observable to the caller.

The callback must perform bounded work without allocation, deallocation, blocking locks, file
I/O, decoding, or device lifecycle operations. Publishing snapshots does not by itself prove
this requirement: final-owner destruction and snapshot acquisition/reclamation require evidence.

## Playback States

The same state machine applies to `SoundHandle` and `MusicHandle`:

| Initial state | `pause()` | `resume()` | `stop()` |
| --- | --- | --- | --- |
| `Playing` | `Paused` | `Playing` | `Stopped` |
| `Paused` | `Paused` | `Playing` | `Stopped` |
| `Stopped` | `Stopped` | `Stopped` | `Stopped` |

`Stopped` is terminal for that playback instance, including after EOF and when commands race
with completion or an explicit stop. Replaying the resource creates a new instance and handle.
Cloned handles control the same instance. Changing volume or looping cannot restart a stopped
instance.

## Gain, Mute, and Pause

- Mute or zero effective volume produces silence while playback and PCM consumption continue.
  A non-looping playback can reach EOF and become stopped while muted.
- Pausing a handle freezes its playback position.
- Pausing a bus freezes playbacks routed through it without replacing their individual states.
  Resuming the bus does not resume an explicitly paused handle or restart a stopped handle.
- Unmuting and changing volume preserve pause and terminal states. Mute preserves the configured
  volume value. A paused playback stays frozen even when its effective volume is zero.

The mixer must distinguish pause from zero gain; the effective gain alone cannot decide whether
to consume PCM.

## Position, EOF, and Looping

`cursor_frames()` counts complete PCM frames consumed by the mixer at the device's resolved
sample rate. It does not represent the exact hardware playback position and does not count
silence inserted for an underrun. A non-looping sound's cursor must not advance beyond its
decoded length; a looping playback reports cumulative progress across repetitions.

Decoder EOF does not complete a music playback while decoded PCM remains buffered. Completion
occurs after the final frame is consumed, including when it exactly fills an output buffer.
Enabling looping before the playback reaches `Stopped` must allow refill to restart even if
the decoder has already reached EOF.

## Acceptance Evidence

The implementation must supply these distinct checks; this list is not a record of completed tests:

| Requirement | Evidence |
| --- | --- |
| Independent domains | Compile the facade with graphics only, audio only, and both; inspect the audio-only dependency graph for graphics/Vulkan/GLFW. |
| Public facade | Headless API tests through `vmnl::audio`, alongside independent domain-crate tests and existing graphics API tests. |
| Execution mode | Deterministic tests for the default and resolved mode; verify automatic `update()` does not introduce a second control producer. |
| Playback semantics | Device-independent tests for every state transition, stop/completion races, bus/handle pause combinations, mute, and zero volume. |
| Streaming and cursors | Exact and partial buffers, buffered EOF, looping changes before/after stop, and underrun position accounting. |
| Realtime ownership | Allocation/deallocation instrumentation covering first use and snapshot replacement/reclamation; lifecycle and single-consumer evidence. |
| Native backend | Separate device startup, error, and shutdown checks on each qualified platform; deterministic mixer tests do not qualify hardware behavior. |

Rustdoc, user workflows, and the legacy design material must be reconciled with the implemented
contract before completion. Release requirements remain governed by [Deployment](deployment.md).
