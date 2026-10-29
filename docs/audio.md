# Audio Acceptance Contract

## Status and Scope

This document defines the acceptance target for the audio integration in PR #84. It records
the backend and facade decisions and the behavior required before that integration is complete.
It is not a claim that the current implementation already satisfies every requirement. Public
Rustdoc describes item-level behavior and current limitations; this document defines the target
against which the remaining implementation work is evaluated.

`vmnl_audio` uses CPAL. The facade defaults to graphics and exposes audio through opt-in
`audio`; disable default features for audio without graphics. The VMNL control worker
is disabled by default. Terminal states, consumed-frame cursors, live looping,
mute/pause semantics, and snapshot heap behaviour have device-independent regression coverage.
Native shutdown and realtime timing remain unqualified. The audio test files are registered as
separate [device-independent and native Cargo targets](testing.md#audio-tests); their scope does
not establish complete acceptance of the facade, execution defaults, or native backend.

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

The graphics dependency and its exports are conditional on `graphics`. With no features, the
facade compiles without any domain. Using audio does not require a graphics `Context`. Direct use of `vmnl_audio` remains
supported. Removing the facade or splitting repositories is deferred until after #84.

## Decoding and Output Selection

Source metadata must describe mono/stereo audio at a nonzero rate. Full-resource decoding
produces finite, complete stereo PCM frames. The control-side resampler also rejects a zero
source rate or an incomplete frame before its empty/same-rate shortcuts. Clients cannot directly
construct the private decoded resource; tests that bypass construction check internal defense,
while malformed-file tests check the actual decoder entry paths.

Device creation selects supported stereo ranges using the format priority documented on
`AudioDevice`, then the nearest supported rate within that priority. The stream keeps the
backend's default buffer policy. Availability, resolved rate, and buffer timing can differ by
platform; inspect the device's resolved rate rather than assuming the requested rate was used.

## Execution and Ownership

The VMNL control worker is opt-in: `.thread(false)` is the default for builder and
`AudioDevice::new`; `.thread(true)` explicitly selects automatic control. Builder/device
`control_mode()` expose `ControlMode::Manual` or `ControlMode::Automatic`, independently of
backend threads and worker health. A failed automatic worker retains its configured mode. Clients
that relied on the earlier experimental automatic default must select `thread(true)` or drive
`update()` themselves. In manual mode,
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

### Control Failures and Shutdown

In manual mode, `update()` returns the original control/refill error. A stream whose refill
fails is marked `Stopped`; cleanup still runs on a failed cycle, without replacing its original
error. Later manual updates can service the remaining streams. The failing cycle does not
guarantee that every other stream was refilled before returning.

In automatic mode, the first control-cycle error or unwinding panic stops the worker. A panic
is reported, not retried: runtime locks may have been poisoned. Automatic `update()` and queued
global controls then return `InvalidState`; there is no automatic fallback to manual processing
or worker restart. A command racing with failure can remain unapplied. Individual handle
controls still operate on shared state; unaffected playbacks can consume already buffered PCM
but streamed music eventually underruns without a control producer. Recovery requires a new
device; existing resources remain bound to their original runtime.

`take_background_error()` takes the last worker/CPAL stream diagnostic from one shared slot.
A later report can overwrite an unread diagnostic; it is not an error queue. Taking the message
does not clear the worker's failed state. Backend stream errors are reported separately and do
not automatically restart a stream or stop the VMNL worker. Their reporting path formats a
string and locks the shared diagnostic slot, outside the measured data-mixing path.
Panic capture requires unwinding;
`panic=abort` and process aborts cannot be recovered this way.

Stopping the worker is idempotent: request stop, unpark its idle wait, and join its thread.
Joining waits for an in-flight cycle, including decoding and file I/O; it does not cancel that
work or impose a shutdown deadline. A panic detected by join is retained as a background
diagnostic. Device destruction joins the worker before dropping the stream and then its runtime
ownership. Backend-specific stream shutdown still needs native platform qualification.

The data callback must perform bounded work without allocation, deallocation, blocking locks, file
I/O, decoding, or device lifecycle operations. Publishing snapshots does not by itself prove
this requirement: final-owner destruction and snapshot acquisition/reclamation require evidence.

### Snapshot Ownership

Each category has three vector slots: one owned by the control writer, one exchange slot, and
one owned by the reader. Publishing replaces the writer's contents, then transfers ownership
with an acquire/release atomic exchange. Unread publications are coalesced; the next reader
adopts the latest available view without replaying intermediate lists. Playback state remains
shared, so stopped instances are skipped even in an older view.

The reader uses one atomic admission attempt and at most one slot exchange. It does not use
thread-local registration, clone/drop snapshot Arcs, lock a mutex, or retry until a writer
finishes. Overlapping readers are rejected without consuming that category; the native data
callback remains the sole intended consumer. A read guard retains its slot between calls and
returns the previous slot only on adopting a later publication.

The writer serializes publication and reclamation on the control side. The three ownership
indices remain distinct. Acquire/release exchanges publish vector initialization and complete
the reader's prior accesses before a returned slot is cleared. Reader admission also serializes
successive readers, including when callback execution moves between OS threads. No vector is
mutated or cleared while a read guard can reference it.

Control cleanup clears returned views even when active playback lists have not changed. A
current or unread view can retain resources until a later reader/control cycle or runtime
destruction. The number of retained views is fixed at three per category; their byte size still
depends on the playback count and prior vector capacities. There is no growing retirement list.
The device retains its runtime while its stream is destroyed. Runtime destruction then releases
any remaining views outside data-callback mixing; native backend teardown still needs platform
qualification.

These are ownership and heap-operation guarantees for VMNL's snapshot path. They do not bound
OS scheduling, backend behaviour, total callback duration, or mixing work independently of the
number of active playbacks.

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
to consume PCM. Master controls affect both categories; a music/SFX bus controls only its route.
Resuming the master bus preserves category pauses. Unmuting or restoring volume continues from
the current position, without replaying silently consumed PCM. Silent playback still requires
music refill through the selected control mode; underrun silence does not advance its cursor.

## Position, EOF, and Looping

The OGG adapter must consume empty Vorbis priming packets without treating them as EOF.
It continues until PCM is available, the upstream decoder reports end of input, or an error
occurs. This decoding work stays on the control side.

`cursor_frames()` counts complete PCM frames consumed by the mixer at the device's resolved
sample rate. It does not represent the exact hardware playback position and does not count
silence inserted for an underrun. A sound that has never looped must not advance beyond its
decoded length; once looping has been used, progress remains cumulative across repetitions,
including after looping is disabled.

Decoder EOF does not complete a music playback while decoded PCM remains buffered. Completion
occurs after the final frame is consumed, including when it exactly fills an output buffer.
Enabling looping before the playback reaches `Stopped` must allow refill to restart even if
the decoder has already reached EOF. Control changes are observed by subsequent mixing/refill
work; they do not undo already mixed output or restart an instance whose completion won a race.
Disabling a sound loop finishes the current repetition without resetting its cumulative cursor.
Disabling a music loop retains PCM already buffered, including prefetched repetitions.

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
