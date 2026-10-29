# Testing

## Audio Tests

The `vmnl_audio` manifest registers two integration-test targets from `tests/audio/`. There is
no separate audio test package or standalone manifest in that directory.

| Command | Scope |
| --- | --- |
| `just test-audio` | Target `audio`: configuration/errors through the public domain crate, plus private decoder, ring, mixer, runtime, and worker contracts; no output device is opened. |
| `cargo test -p vmnl_audio --tests --no-run` | Compile both integration targets without opening a device. |
| `just test-audio-backend` | Target `audio-backend`: explicitly run ignored native playback, bus, and worker tests against a usable CPAL stereo output device. |

Audio Rustdoc examples are enabled and included in `just doctest`. To run only those examples,
use `cargo test -p vmnl_audio --doc --locked`: the configuration example executes without a
device, while the device-creation example is marked `no_run` and only compiled.

The OGG regressions use the existing `tiny.ogg` fixture, including its empty Vorbis priming
packet. They check initial PCM against Lewton, complete PCM delivery across small reads,
agreement with full-resource decoding, and repeated reads after EOF. These checks do not
qualify container-timing trimming or audible playback.

`ogg_chains` concatenates two versioned Vorbis segments with distinct stream serials.
It checks rate-only, channel-only and combined transitions through streaming and full-resource
decoding. These six rejection regressions are active and are expected to fail until the decoder
rejects format changes with `UnsupportedFormat`. A same-format chain must retain Lewton's PCM
across four read patterns, empty reads and repeated EOF; a separate check verifies each fixture's
metadata, serial, empty priming packet and non-silent PCM. The oracle reads packets from the
actual chain, not the container durations or the sum of separately decoded segments.

The five `chain-*.ogg` fixtures in `tests/audio/fixtures/` encode original synthetic 100 ms sine
waves under the repository's MIT license. They were generated once with FFmpeg 8.1.3/libvorbis;
tests read their committed bytes and never invoke an encoder or require FFmpeg in CI.

| Fixture | Channels | Rate (Hz) | Tone (Hz) | OGG serial |
| --- | ---: | ---: | ---: | ---: |
| `chain-start-mono-8k.ogg` | 1 | 8,000 | 440 | 84 |
| `chain-next-mono-8k.ogg` | 1 | 8,000 | 660 | 85 |
| `chain-next-mono-16k.ogg` | 1 | 16,000 | 660 | 86 |
| `chain-next-stereo-8k.ogg` | 2 | 8,000 | 660 | 87 |
| `chain-next-stereo-16k.ogg` | 2 | 16,000 | 660 | 88 |

For each row, encoding used `ffmpeg -f lavfi -i
"sine=frequency=TONE:sample_rate=RATE:duration=0.1" -map_metadata -1 -ac CHANNELS
-c:a libvorbis -q:a 3 -flags:a +bitexact -fflags +bitexact -serial_offset SERIAL FILE`.
Substitute the table values; bitexact flags and explicit serials remove randomized stream IDs.
These source-copy checks qualify neither native playback nor encoder padding/container timing.

`playback_states` covers the sound/music state table, repeated commands after stop from another
thread, ordered global stop/pause/resume commands, exact-buffer completion, cursor accounting,
and dynamic loop changes around decoder EOF and refill. Its cross-thread cases use barriers
for a deterministic happens-before order; they are not exhaustive concurrency model checking.

`gain_semantics` checks PCM output, cursors, and EOF for mute and zero gain on master/music/SFX
buses, zero handle volume, pause precedence, independent category/handle states, unmute/volume
restoration, looping, and silent music underruns. It uses the private implementation harness
without opening a device; these cases do not qualify audible output or realtime performance.

`realtime` instruments the Rust global allocator using const-initialized thread-local counters.
It checks the first mix on a fresh OS thread, release/adoption of replaced views, actual resource
reclamation on the control thread, and the probe's ability to observe heap work. `snapshots`
checks unread-update coalescing, retained-view lifetime, overlapping-reader rejection, owner
destruction, and whole-view consistency with two concurrent publishers. These checks exercise
the production sources through the private harness. They do not instrument C/backend allocation,
prove a callback deadline, or exhaustively model weak-memory interleavings.

Worker tests force a looping source to disappear before refill, checking original-error
propagation, terminal stream state, cleanup on failure, persistent automatic-worker failure,
and release of its runtime after join. Private worker unit tests inject a cycle error/panic,
check an unexpected join panic, and gate an in-flight cycle during stop. Repeated stop/Drop is
covered. The native target additionally checks public control errors after worker failure;
compiling that ignored case does not qualify a real device or native shutdown latency.

Resampling regressions compare full-resource and streaming PCM for mono/stereo WAVs at five
rate pairs, including 44.1/48 kHz and 22.05/48 kHz in both directions. Seven source lengths
cover empty input, short exact-rate boundaries and longer refills. Four read patterns must
produce the exact rational frame count and identical streaming PCM; full-resource PCM is
compared with the existing interpolation tolerance. Empty reads and repeated EOF preserve
output. Separate playback regressions check simultaneous completion at 480 output frames
and three Music/Sound loop repetitions without an extra frame. These private source-copy
checks do not qualify encoder padding, audible quality or callback timing.

The resampler's zero-source-rate and incomplete-frame cases deliberately fabricate invalid
private PCM; they prove internal rejection before empty/same-rate shortcuts, not a state
constructible by clients. Separate WAV tests corrupt real metadata/data and check both full
and streaming decoder entries. `output_config` uses valid advertised CPAL ranges to check
stereo filtering, format preference, nearest/clamped rates, unsupported outputs, and the backend
default buffer policy. These tests do not require a host or qualify its actual device formats.

Ring regressions seed valid empty rings near `usize::MAX` and check FIFO order, exact capacity,
full/empty and partial transfers when the counters overflow together or separately. They cover
capacities 1, 2, 7, 44,100 and 48,000 samples and measure no Rust heap operations during reads.
The SPSC exercise checks 120,000 sample identifiers between two threads, both from zero and
across machine-counter overflow. These use the production source-copy harness on the host's
word width; they do not constitute native 32-bit execution, exhaustive memory-model checking
or a callback-deadline measurement.

Facade checks use `python3 tools/test_facade_features.py`, also run by `just test-api`.
They generate independent client workspaces under `target/`, seed their lockfiles from the root
lockfile, and test default, explicit graphics, audio-only, both, and empty configurations. Their
normal/build dependency trees must exclude graphics/Vulkan/GLFW without `graphics`. Audio checks
import through `vmnl::audio`, verify type identity with `vmnl_audio`, manual/default and explicit
automatic modes, and reject invalid configuration without opening a device. Empty configuration
is a compilation/graph check. Facade Rustdoc is also tested for each selected configuration.
Workspace-wide feature union cannot substitute for these clients.
`--clippy` checks the same clients with warnings denied. The runner changes only generated
clients and normal Cargo artifacts; it does not change the root lockfile or host configuration.

The independent `Audio` CI matrix compiles both targets and runs `audio` on Linux, macOS,
and Windows with Rust 1.87. Linux jobs install the ALSA development files. This matrix has no
dependency on the Quality job, so Clippy failures do not suppress audio diagnostics. There is
no `continue-on-error` for audio failures. It also checks isolated empty/audio-only clients
without graphics prerequisites; Validation checks all five combinations. Native compilation
is not native execution.

CI checks the selected compiler's actual version before validation. The pinned toolchain action
must accept its `toolchain` input: a version-specific action can hardcode another compiler and
ignore that input. Documentation additionally verifies the pinned API-extraction nightly and
builds the facade/Rustdoc with all features. A successful job on another compiler does not
qualify Rust 1.87.

The native tests are ignored by default and excluded from the device-independent target. An
explicit native run fails if device initialization or format selection fails; missing hardware
is not reported as a passing test. These tests can produce audible output. No recipe installs
system packages or changes the host audio configuration. Linux compilation still needs CPAL's
ALSA development prerequisites even when no output device is opened.

`just test` and `just validate` include `test-audio`, and their aggregate reports retain its
pass/fail counts. Known audio assertions remain enabled: registering a test does not imply that
the implementation already satisfies the [acceptance contract](audio.md).

The private implementation harness compiles copies of the production sources in a test-only
module tree, without making private types public. Its results prove behavior at that private
seam; they do not replace tests against the compiled public library or qualify a CPAL device.
Ordinary domain contract tests use `vmnl_audio`; isolated clients test `vmnl::audio` through
the compiled facade. Native backend results are separate from human
listening validation.

### Native audio qualification

Record the source SHA, OS, compiler, CPAL version, backend/default output route, resolved output
rate when observed, and initial mute/volume settings. Keep host audio settings fixed during
a run. Run `just test-audio-backend` on each available platform; preserve named results,
errors and the process exit status. Use an external watchdog against the test executable to
bound a hung run. A watchdog timeout is a failure, not an excluded test or a shutdown guarantee.
Compiling with `cargo test -p vmnl_audio --test audio-backend --no-run --message-format=json`
provides the executable path without opening an output device.

The eight current probes cover default manual constructors, automatic music refill without
client updates, manual refill after exhaustion, public errors after automatic worker failure,
shared bus/handle state, and load errors. Each creates and destroys a device. They do not inject
a backend disconnection or measure callback deadlines, teardown latency, output waveform, or
listening quality. Track those qualifications separately; no failed/missing device is a pass.

On 2026-10-29, source `ef899d8c098e101f64f8294169c9f3f40a4224be` passed all eight native probes
on Fedora 44 x86_64, kernel 7.2.11-200.fc44, Rust 1.95.0 and CPAL 0.18.2. ALSA 1.2.16.1's default
output routed through PipeWire 1.6.10 to the `sof-hda-dsp` speaker sink. The sink stayed muted at
volume zero before and after the run. The test executable ran serially with
`--ignored --test-threads=1 --nocapture` under a 45-second watchdog and returned in 2.071 seconds: 8 passed,
0 failed, 0 ignored. The resolved CPAL rate was not recorded. This is native callback/control
and ordinary teardown smoke evidence, not human listening or a worst-case shutdown bound.
Windows/macOS native audio execution remains unqualified.

## Virtual joystick/gamepad scenarios (issue #94)

These tests preserve the existing unit/API/Null suites and reuse #91's bounded, continuously
drained process transport, strict READY/INJECTED/OBSERVED barriers, five-second observation
deadlines and per-check terminal report. Shared transport/report code lives in `tests/support`.
The Linux uinput fixture stays in the driver; the separate client depends directly only on
`vmnl`. There is no production API or synthetic VMNL event injection added for testing.

```bash
just joystick-test x11 runtime     # public InputRuntime; no window or Vulkan initialization
just joystick-test x11 window      # Window::poll_events -> Event/Input; requires Vulkan/display
just joystick-test wayland runtime # same Linux fixture; native Wayland compositor
just joystick-test wayland window  # same public window checks; requires Vulkan/Wayland
VMNL_JOYSTICK_REPORT=/tmp/vmnl-joystick.json just joystick-test x11 runtime
```

Linux needs a usable selected GLFW display backend, uinput ABI >= 5, write access to
`/dev/uinput`, and read access to the **new fixture's** `/dev/input/eventN` node. Readable
existing nodes alone do not prove that newly created nodes will be accessible. The launcher
checks prerequisites before building or injecting. The driver obtains its owned sysname through
UI_GET_SYSNAME and waits at most five seconds for its event node to become readable. Local test
commands never install rules, change permissions, load modules or invoke sudo. A missing
prerequisite is an error, never an exclusion or successful test. The kernel's
[uinput documentation](https://docs.kernel.org/input/uinput.html) describes this interface.

Every run starts four fresh clients: hat_buttons false/true, each with runtime/window consumer
order AB/BA. The driver creates the device only after subscription and READY, holds each
injected state until complete observation, and destroys only its own fixture. It compares native
integer injection with independent literal public values: all 15 button pairs, distinct vectors
and endpoints for all six axes, nine hat positions, and four appended hat buttons when enabled.
Controlled mapping cases cover all standard controls, hat-to-Dpad, A/B replacement, mapping
unavailability and restoration. Identity is discovered across all public slots, then checked by
name/GUID; no joystick slot is assumed. Reconnection is identified again.

The runtime case also checks independent notification queues, late subscription without replay,
owned sample stability, callback thread, and an owned pointer before native disconnect cleanup.
The window case uses two tracking/delivery consumers plus tracking-only and delivery-only
windows. It checks their Event identity/timestamps, independent snapshots, non-consuming
transitions, explicit and next-poll clearing, tracking versus delivery controls and held-button
disconnect releases, including the batch where gamepad() is None. Runtime polling never advances
window snapshots. A press/release wholly between samples is deliberately not tested as
observable; every state under test must survive until its acknowledgement.

Client mismatch, missing injection, invalid protocol, timeout or unsuccessful exit fails.
On failure the report retains FAIL and NOT RUN entries. The driver releases controls and removes
the virtual device during error cleanup, and the existing transport kills/reaps a failed or
timed-out client while draining both pipes. Explicit held-disconnect cases remove the held
device before physical cleanup so cleanup cannot satisfy the VMNL release oracle.

Native tests are ignored by default. `just test` and `just validate` remain display-free;
the ordinary `just test-gpu` also skips the uinput scenario. Default CI executes the portable
transport, report and driver regressions. Its [Linux validation job](#linux-joystick-ci)
also schedules the native virtual-device scenarios automatically. The optional
`VMNL_JOYSTICK_REPORT` path receives schema-version-1 JSON once the Rust driver starts, including
PASS/FAIL/NOT RUN, reasons and qualified=false for incomplete/failed runs. Preflight rejection
occurs before the Rust report exists. `requested_backend` alone records only a request.
For the Wayland profile the driver removes `DISPLAY` and inherited `WAYLAND_SOCKET`, selects
the named/default Wayland display, and enables `WAYLAND_DEBUG=client` only in the separate client.
Before creating a fixture, each client must provide an outgoing display registry request,
an incoming registry announcement and an incoming callback completion. The driver reads the
existing bounded stderr capture (64 KiB) with a five-second deadline; missing or incomplete
evidence fails the run. `backend_observations` retains these protocol lines, the child PID and
profile for each fresh client. No GLFW import or backend-query API is added to the public client.
The trace facility is documented by [Wayland](https://wayland.freedesktop.org/extras.html).
This joystick route does not require the nested Weston/XTEST seat used by keyboard/mouse probes.

### Public window stick observations (workload 2)

The window client adds seven acknowledged observations through the public builder, setters
and `Window::poll_events()`. Native integer axes remain in the driver; client expectations
are fixed float vectors, magnitudes and worked angles, never derived from VMNL's observed
values or its angle/dead-zone algorithm. The scalar tolerance is `0.0001`; values must be
finite and angles must be in `[0, 2π)`. Every injected vector is held until observation.

Window 0 starts with left `(0.5, Math2D)` and right `(0.5, Screen2D)` dead-zone/convention
settings. Window 1 uses left `(0.0, Heading)` and right `(0.75, Math2D)`. Window 2 keeps the
defaults; window 3 remains untracked. The latter two preserve tracking/delivery isolation.

| Observation | Injected left / right axes | Expected window-0 behavior after polling |
| --- | --- | --- |
| Builder configurations | `(0.5, 0)` / `(0, 0.75)` | Left is centered at the exact threshold; right retains magnitude `0.75`, angle `π/2`. |
| Left setter | Same held vector | Set left to `(0.0, Heading)`: magnitude `0.5`, angle `π/2`; right unchanged. |
| Right setter | Same held vector | Set right to `(0.75, Math2D)`: centered; left unchanged. |
| Paired setter | Same held vector | Set left to `(0.25, Math2D)`, right to `(0.0, Heading)`: retain magnitudes `0.5`/`0.75` and angles `0`/`π`. |
| Dead-zone boundaries | `(0, -0.25)` / `(-0.5, 0)` | Left centered, right magnitude `0.5` and angle `3π/2`. |
| Diagonal angles | `(0.75, -1)` / `(0.3, 0.4)` | Retain magnitudes `1.25`/`0.5`, angles approximately `0.9272952`/`2.4980915`; no normalization or rescaling. |
| Neutral/configuration reset | `(0, 0)` / `(0, 0)` | Reset both configured windows to defaults; magnitudes zero, angles absent. |

Each tracked window has independent fixed expectations. Setters must preserve raw samples,
presence and button transitions, and leave the other windows unchanged. Processed sticks are
included in snapshot-isolation and transition-clear comparisons. Unmapped, disconnected and
untracked slots must expose no processed sticks. Workload 2 requires these seven observations
in every window profile; earlier window reports do not qualify this expansion.

On 2026-10-22, three consecutive local Linux virtual-device executions passed per backend/mode:
X11 runtime 244/244, X11 window 296/296, Wayland runtime 248/248 and Wayland window 300/300.
All twelve reports have zero failures, exclusions or unexecuted checks. Each window execution
includes the seven stick observations in all four hat_buttons/poll-order profiles, and each
Wayland client supplied independent protocol evidence. The temporary uinput ACL was restored
and verified against its initial snapshot; no owned virtual device remained. This qualifies
the exercised local workload-2 paths, independently of pure oracle tests and compilation.
Hosted Ubuntu execution is qualified separately in the
[hosted evidence record](#hosted-linux-joystick-evidence-workload-2); physical controllers and
other OSes remain unqualified.

### Lazy joystick activation regressions

Headless unit tests instrument VMNL's private backend boundary only in test builds. The
shared-runtime Null lifecycle test requires zero slot queries and zero joystick callback
installations during acquisition, configuration inspection and ordinary GLFW event pumping.
Explicit sampling must query the requested slots. The first subscription installs one callback,
additional subscribers and repeated polls reuse it, and the last subscription removes it.
`InputRuntime::poll_events()` is an explicit subscription; acquisition alone is not.

The private snapshot transaction used by `Window::poll_events()` is tested with fixed
keyboard/mouse press, release and empty batches: default builder options must invoke no sampler
and leave all slots `NotTracked`. Explicit tracking queries every slot once per batch; disabled
tracking stops queries. A failed sweep preserves the previous snapshot and initialization flag
until a complete retry succeeds. This transaction is extracted without changing its behavior.

These deterministic/Null checks observe calls initiated by VMNL, not GLFW's internal device
enumeration. They create no Vulkan window, inject no native input, and do not qualify a physical
controller. Public native-window coverage remains the separate virtual-device workload above.
The opt-in GPU regression `default_window_joystick_state_stays_untracked_after_polling`
additionally checks actual default-window construction and three public event polls: tracking
and connection delivery stay disabled, every slot remains `NotTracked`, and no joystick Event
is delivered. It does not expose the unit-test counters. No instrumentation API or dependency
is added to the public clients.

### Joystick callback lifecycle regressions

`cargo test -p vmnl-api-tests --test input_runtime_callbacks` exercises the public facade under
Null. A separate test driver invokes the installed GLFW C trampoline with synthetic connection
notifications; it creates no device, window or Vulkan resource. Cases cover callback self-removal,
self-replacement, last-subscriber removal, immediate resubscription and cleanup before a user
panic resumes. Polled queues retain their ordered notifications when their callback is removed.
The driver also checks that the unused native callback is actually unregistered after returning
to Rust. These are deterministic/Null callback-boundary checks, not hot-plug or physical-device
qualification.

### Linux joystick CI

The `CI` workflow includes four blocking steps in `Validation: Linux` after the keyboard/mouse
probes: X11/Wayland, each in runtime/window mode, with three consecutive repetitions per step.
They run on pull requests, pushes to `main` and manual workflow dispatches without a joystick
opt-in input or separate job. The existing workspace build is reused. `just test` and
`just validate` remain display-free, and local native commands remain explicitly opt-in.
The workflow does not use `continue-on-error` for joystick scenarios. Any failed prerequisite,
scenario or report check fails Linux validation; later scenario steps are skipped after failure.
Physical-controller qualification remains a later, separate step.

The Linux job provisions its disposable Ubuntu runner. It loads uinput, saves and temporarily
grants the runner UID access to `/dev/uinput`, and installs a volatile udev rule limited to newly
created event nodes with this fixture's name/vendor/product. An always-run cleanup removes
the rule and restores/checks the saved uinput ACL. No repository test or local launcher grants
permissions. This provisioning is limited to the disposable hosted runner configured in the
workflow; it is not a setup script for a developer's machine.

X11 uses Xvfb/Openbox. Wayland uses Weston/Pixman nested in that display; each public client
still supplies its independent Wayland protocol evidence with DISPLAY unset. Window mode
uses Mesa's lavapipe software Vulkan ICD selected explicitly through `VK_DRIVER_FILES`, scoped
to the two window steps through a step output;
the [Vulkan loader](https://github.com/KhronosGroup/Vulkan-Loader/blob/main/docs/LoaderDriverInterface.md)
documents this driver override. Both runtime steps precede ICD selection and do not invoke
the Vulkan preflight. Each backend/mode uses a fresh display and a separate artifact directory.
Missing input access or display, and missing software ICD/surface support in window mode,
fail the job. No fallback
to Null or a different display backend qualifies the run. The inspected hosted workload-2
execution is recorded below. Local Hyprland results remain separate from that
Xvfb/Weston/lavapipe qualification on Ubuntu.

`tools/joystick_ci.py` only repeats the existing `tools/input_test.py --joystick` command;
it does not introduce another injector, transport or client. It requires a new artifact
directory, rejects unsuccessful exits, missing/malformed reports, unexpected schema/profile,
duplicates and every FAIL/EXCLUDED/NOT RUN check. Every hat_buttons/poll-order profile must
match the fixed version-2 acceptance inventory in that validator, independently of the driver's
reported plan. Missing and unexpected check identities fail, including a check omitted or
replaced in all four profiles. The expected totals are 244/296 for X11 runtime/window and
248/300 for Wayland; the inventory comparison checks identities, not just counts. The repetition
summary records `workload_version` separately from its JSON schema version. Each Wayland
profile also needs its own backend evidence. A fixed per-repetition timeout terminates the
complete subprocess group. Failure stops that scenario's remaining repetitions and leaves its
summary unqualified.

The same repetition/report validation can be exercised on an already configured desktop:

```bash
python3 tools/joystick_ci.py x11 runtime target/joystick-local-x11-runtime
python3 tools/joystick_ci.py wayland window target/joystick-local-wayland-window
```

The Linux job uploads one joystick artifact containing the four scenario directories, with
three native schema-version-1 reports per successful scenario, per-run logs, source/host/
runner/display/ICD metadata, versioned repetition summaries, display/Vulkan diagnostics and
the before/after uinput ACLs. Upload and access restoration run even after failure; skipped
steps do not produce successful reports. Compilation and portable report-validator
tests do not qualify a native runner. The inspected hosted execution is recorded below;
physical devices and Windows/macOS virtual fixtures remain unqualified.

### Hosted Linux joystick evidence (workload 2)

On 2026-10-22, the `joystick-linux-1` artifact from
[CI run #39711518595](https://github.com/VMNL/vmnl/actions/runs/39711518595), attempt 1, was
downloaded and checked against its GitHub SHA-256 digest. All twelve native reports were
revalidated with the existing exact-workload validator, then checked against their repetition
summaries, client logs and successful Linux job steps.

| Profile | Checks per execution | Consecutive executions | Observed display/Vulkan setup |
| --- | --- | --- | --- |
| X11 runtime | 244 | 3/3 PASS | Xvfb/Openbox; no window or Vulkan initialization in the client. |
| Wayland runtime | 248 | 3/3 PASS | Weston/Pixman nested in Xvfb/Openbox; independent client protocol proof. |
| X11 window | 296 | 3/3 PASS | Xvfb/Openbox; lavapipe ICD selected explicitly. |
| Wayland window | 300 | 3/3 PASS | Weston/Pixman nested in Xvfb/Openbox; lavapipe ICD and independent client protocol proof. |

The job ran on Ubuntu 24.04 runner image `20261018.339.1`, Linux x86_64 kernel
`7.0.0-1017-azure`. Weston reported `13.0.0` with Pixman. Both window profiles selected
`/usr/share/vulkan/icd.d/lvp_icd.json`; their Vulkan diagnostics reported only llvmpipe,
Mesa `25.2.8-0ubuntu0.24.04.4`, LLVM `20.1.2`. Runtime summaries had no Vulkan driver override.

The PR head was `6683cc4a3a0cc6ae49090680451d61e12900eb5c`. All four summaries record a clean
checkout of the CI merge commit `f3dc91434ce98b11a37341372828c0f3186b9cac`, whose parents are
`main` at `e710938e2029e525c790c069ff76d920b24450b9` and that PR head. These results qualify
that tested integration, not every future branch revision or runner image.

All 3,264 checks passed, with zero FAIL/EXCLUDED/NOT RUN entries. Each report includes all four
hat_buttons/poll-order profiles: 48 fresh-client profiles in total. Each window profile includes
all seven stick observations. The 24 Wayland clients have distinct recorded PIDs and an outgoing
registry request, incoming global announcement and incoming callback completion. Their evidence
records DISPLAY unset in the client; the parent retained DISPLAY for nested Weston. X11 reports
record the requested profile and configured display, without an independent client protocol trace.

The cleanup step removing the fixture rule, reloading udev and restoring input access succeeded.
The archived `uinput-before.acl` and `uinput-after.acl` are byte-identical: root:root, mode 0600,
with no named-user ACL entry. No final per-device inventory was archived, so these files alone
do not establish absence of every fixture node after cleanup. This is native virtual-device
evidence through public VMNL clients; physical controllers, Windows/macOS virtual injection,
hardware GPU behavior, rendered presentation and human graphical validation remain unqualified
by this run.

### Windows/macOS virtual fixture prerequisites

Windows/macOS virtual injection is not implemented. Preparing the physical operator protocol
does not provide a virtual-device supplier. The controlled mapping text in the public client
now has no `platform:Linux` filter; its deterministic test fixes all 21 button/axis selectors
and their swap, mapping-loss and hat variants. The current fixture identity, native driver
and native qualification remain Linux-specific. Platform-neutral mapping text alone is not
Windows/macOS native evidence.

The Windows VHF path needs a separately built/deployed HID source driver. Microsoft's
[VHF specification](https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/virtual-hid-framework--vhf-)
supports the source driver in kernel mode; VHF is not a user-mode joystick injection API.
An existing virtual-device supplier would need an identified version/API and owned-device
connect, held-state update, disconnect and cleanup semantics before adapter implementation.
No supplier, driver package or signing/deployment infrastructure has been selected here.

The macOS candidate is an IOHIDUserDevice helper carrying Apple's
[`com.apple.developer.hid.virtual.device` entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.hid.virtual.device).
Apple's [IOHIDResourceDeviceUserClient source](https://github.com/apple-oss-distributions/IOHIDFamily/blob/777ccd9698845aadf711e32d843c8c9b777431d9/IOHIDFamily/IOHIDResourceUserClient.cpp)
checks the developer virtual-device or Apple user-device entitlement when opening that user
client. The inspected implementation has no administrator/root exception to that check.
No authorized signing profile/helper is available; sudo, accessibility access or a generic
physical-controller connection are not substitutes for this capability.

Once provisioned, suppliers must stay outside the public VMNL client and reuse the existing
driver's barriers, deadlines and cleanup. Expected samples must come from a fixed fixture
specification, independently of values observed through VMNL. Missing supplier/capability,
failed creation or timeout must fail qualification. OS injection, compilation, deterministic
mapping tests and physical hardware results remain separate evidence. No driver, entitlement,
certificate, security setting or privileged host configuration is installed by these tests.

### Local Linux joystick evidence

On 2026-10-17, actual host preflight found `/dev/uinput` root-owned mode 0600 and open returned
EACCES; current event nodes were readable. Sandbox inspection initially hid the nodes, so host
access was checked separately. Neither fixture detection by the pinned GLFW nor the public
native scenarios could be qualified. Deterministic tests, Null, native virtual-device tests and
physical-controller observations are separate evidence levels. Windows/macOS fixtures and all
physical controllers remain unqualified. See the
[capability matrix](api/maintenance/joystick_capability_matrix.md#functional-linux-coverage-added-for-issue-94).

On 2026-10-18, an explicitly operator-authorized temporary ACL granted UID 1000 access to
uinput. The native Linux fixture was then detected by the pinned GLFW through the public client;
its new event nodes were readable without an additional permission change. Three consecutive
executions of each command with the X11 profile selected passed after adding sample/slot identity
checks: runtime 244/244 report checks and window 268/268 per execution, with no failures,
exclusions or unexecuted checks. Each execution includes both hat_buttons settings and both
poll orders in fresh client processes. The window case also passed window-first connection and
disconnection fan-out. Initial successful runs before the identity assertion are separate from
these final repetitions. This qualifies the exercised Linux virtual-device paths, not physical
controllers, another display profile, Windows/macOS, rendered frames or human graphical behavior.

The local environment was Fedora 44, kernel 7.2.8-200.fc44.x86_64, Hyprland 0.56.2 and Xwayland;
Vulkan enumeration reported Mesa 26.2.3 with Intel Iris Xe and llvmpipe available. This does not
identify the client's selected GPU or prove presentation. Production source remained PR #97 at
6488761, with the test patch in the worktree. The main/PR conflict was still present against
2ead9f4, so integration requires revalidation. The backend field in each JSON remains a request,
not an independent backend observation. After all six final runs, the guard restored and verified
the original root:root 0600 uinput ACL with no added named user entry. The repository commands
never grant access themselves; another execution still requires suitable pre-existing access.

On 2026-10-19, the Wayland profile passed three consecutive executions per mode on Fedora 44,
Hyprland 0.56.2 and the native `wayland-1` display, using source commit `0609d2e` plus the local
test-support patch. Runtime passed 248/248 checks and window passed 272/272 per execution;
four checks per command are the new backend observations. Every run included both hat_buttons
settings and both polling orders, with four independently captured Wayland protocol proofs.
All six reports had zero failures, exclusions or unexecuted checks. The explicitly authorized
temporary uinput ACL was restored to root:root 0600 and no owned fixture remained afterwards.
Reports, logs and source/environment metadata are retained locally under
`target/input-test-artifacts/joystick-wayland-2026-10-19/`. These are native virtual-device
observations through public VMNL APIs; they do not qualify physical controllers, Windows/macOS,
rendered presentation or human graphical behavior.

## Physical joystick operator protocol

The physical path reuses the public joystick client, #91 process transport, strict
READY/INJECTED/OBSERVED barriers, per-check report and Wayland backend observation. It does
not create a virtual device, write input events, change mappings or request uinput access.
The launcher accepts `x11`/`wayland` on Linux, `win32` on Windows and `cocoa` on macOS;
a backend/host mismatch fails before compilation. The Rust driver checks the pair again before
starting its client. A connected controller accessible to the current user and an interactive
operator terminal are required. Linux additionally needs its selected display backend and
readable controller nodes. Window mode also requires Vulkan and a native display. The operator
declares the model and USB/Bluetooth connection; neither a GUID nor a virtual HID node proves physical
hardware by itself. Known VMNL virtual fixtures are rejected. No keyboard/mouse injection
permission (SendInput desktop access or Cocoa event-posting permission) is requested by this
read-only controller path.

```bash
VMNL_JOYSTICK_REPORT=/tmp/physical-runtime.json \
  just joystick-physical-test wayland controller.json runtime
VMNL_JOYSTICK_REPORT=/tmp/physical-window.json \
  just joystick-physical-test wayland controller.json window
# Native Windows, from its operator terminal (use python if python3 is not on PATH):
python3 tools/input_test.py --joystick-physical runtime --physical-profile controller.json win32
# Native macOS, with its own independently prepared controller profile:
python3 tools/input_test.py --joystick-physical window --physical-profile controller.json cocoa
# x11 is also selectable; requested_backend alone does not independently prove backend selection
```

Prepare a schema-version-1 JSON profile from the device specification or an independent
measurement, **not values copied from the VMNL output being tested**. This illustrative
profile describes one button, one axis and one hat; it is not a qualified controller profile:

```json
{
  "schema_version": 1,
  "model": "replace with the physical controller model",
  "connection": "usb",
  "buttons": [{"index": 0, "label": "South face button", "mapped": 0}],
  "axes": [{
    "index": 0, "label": "Left stick horizontal",
    "minimum": -1.0, "neutral": 0.0, "maximum": 1.0, "tolerance": 0.1,
    "gamepad": {"index": 0, "minimum": -1.0, "neutral": 0.0, "maximum": 1.0}
  }],
  "hats": [{"index": 0, "label": "Dpad"}],
  "exclusions": []
}
```

All four arrays are required. Every reported raw button, axis and hat must have a tested
index or an explicit exclusion, for example
`{"kind":"button","index":1,"reason":"control unavailable for this session"}`.
Duplicate/out-of-range indices, an implicit omitted control or an empty reason fail the plan.
At least one button must be tested to exercise held disconnection. The protocol bounds the
inventory at 128 raw buttons, 64 axes and eight hats; larger devices fail with a diagnostic.
`mapped` and axis `gamepad` are optional (or null) for raw-only/unmapped devices. Their indices
use the public GamepadButton/GamepadAxis order. Mapped controls without independent expectations
are explicitly EXCLUDED in the report. A successful report qualifies only its tested profile.

The client lists all present slots with names, GUIDs, raw counts and mapping availability.
Type PHYSICAL to attest the hardware, then select an actually listed zero-based slot; slot 0
is never assumed. The selected identity and counts must remain unchanged throughout the run.
Each prompt has a 45-second deadline (q cancels). Perform the action, press Enter, and maintain
the state until PASS before changing it. VMNL has five seconds to observe values stable for
150 ms. A press/release completed between samples is not an observable test action.

Each tested button follows initial release, press, release. Axis steps use neutral, minimum,
maximum and restored neutral, with an explicit absolute tolerance in [0.001, 0.25]; target
bands must not overlap, except that a trigger's neutral may equal an endpoint. Optional mapped
axis targets are independent and may be inverted relative to the raw axis. Hats test all nine
positions then return to center. The final button stays held until the operator disconnects
the device. The client requires absence and a single disconnect notification in each active
consumer; window mode also checks mapped transitions, no held controls after disconnection,
one-batch release flags, and their clearing at the next absent batch.

No device, invalid profile/protocol, cancellation, EOF, timeout, wrong value or unsuccessful
child exit fails the run. Remaining planned checks stay NOT RUN. On error the child is reaped;
the operator must release the physical controls. JSON uses evidence=physical-operator-declared
and records the profile, operator declaration and selected identity. It does not independently
authenticate the hardware. The physical metadata records the driver host OS/architecture.
Only the Wayland path currently supplies independent backend protocol observations; an empty
backend_observations array on other profiles is not backend-selection proof. Windows/macOS
launches discard inherited Linux display/session variables. The physical scenarios remain
ignored and excluded from normal GPU validation. CI tests their deterministic oracles and failure handling, not physical hardware.

On 2026-10-19 the operator reported no controller available. Physical qualification remains
pending; implementation, compilation and simulated driver tests are not hardware evidence.
The Windows/macOS protocol is prepared for native execution, with the same slot selection,
barriers, deadlines, tolerances and held-disconnection checks. Neither OS is qualified without
its native runtime and window reports. Profiles must be prepared independently for each
OS/driver/connection: raw control order and available mappings are not assumed portable.
The existing CI platform build/test steps compile the physical driver and run its portable
oracles on each native runner; they never invoke the ignored operator scenario. Cross-OS
compilation is not established by a local Linux build. No Windows/macOS virtual fixture is
implemented or installed by this protocol; those separate injection paths remain unqualified.

### Windows and macOS joystick evidence

[CI run #40248972120](https://github.com/VMNL/vmnl/actions/runs/40248972120) observed successful
`Validation: Windows` and `Validation: macOS` jobs on `windows-2022` and `macos-14-arm64` runners.
Both checked out merge `d117935eeafe43f9ff6e1c2840a4a62a578ffe81` (PR head `22a0137`, `main`
at `e710938`). The evidence is scoped to that source and those runner environments.

| Evidence level | Windows | macOS |
| --- | --- | --- |
| Native workspace compilation, including the public client and runtime/window test targets | `cargo build --workspace --all-targets --locked` succeeded. | Same command succeeded. |
| Deterministic, API/Null and portable driver coverage | Unit, API, smoke and portable platform steps succeeded, including all five callback lifecycle cases and physical-profile/driver oracles. | Same steps and cases succeeded. |
| Present virtual joystick through public runtime/window paths | Fixture unimplemented; unqualified. | Fixture unimplemented; unqualified. |
| Physical-controller runtime/window protocol | Operator scenarios not run; qualification pending. | Operator scenarios not run; qualification pending. |

These builds and portable tests do not exercise a present joystick or qualify GPU execution.
The physical operator's 2026-10-19 preparation had no controller or local Windows/macOS host;
its runtime/window hardware reports remain outstanding. Virtual injection has the separate
[supplier and signing prerequisites](#windowsmacos-virtual-fixture-prerequisites).

## Test Types

```text
unit test   = close to implementation, small scope, may test private internals
API test    = black-box public behavior through the vmnl facade
smoke test  = executable startup path, no window, exit code is the oracle
platform test = GLFW NoApi window behavior, no Vulkan or GPU surface
GPU test    = Vulkan/display behavior, may create a window
doctest     = Rustdoc example compilation/execution
example     = user-facing visual program, not a test oracle
```

## Test Design

Start from the observable contract and its owner: explicit requirement, canonical documentation,
public Rustdoc, upstream specification, or an approved measured baseline. Select the lowest
practical deterministic seam that exercises that contract rather than an implementation detail
that merely correlates with it.

- Public facade behavior belongs in an API test when it can remain headless.
- A private invariant or algorithm may be covered by a focused unit test.
- Backend, platform, GPU, FFI, and hardware behavior requires evidence that crosses the real
  boundary; a mock, Null backend, stub, or Rust-only seam proves only its modeled behavior.
- Keep lower-level tests that prove distinct safety, ABI, platform, or failure-handling invariants;
  coverage through a higher interface does not make them redundant.

Expected results must come from an independent oracle: a specification, known worked case, fixed
literal, qualified reference implementation, or directly observable outcome. A test that derives
its expectation with the same algorithm as the implementation is tautological. Tests should fail
when the required behavior is absent and should survive unrelated internal refactors.

For a deterministic feature or fix, prefer one vertical behavior slice at a time:

```text
one observable contract
  -> one focused failing test
    -> the smallest coherent implementation
      -> repeat for the next contract
```

Observing the focused test fail before the change proves that it can detect the missing or defective
behavior. Do not fabricate that evidence or force this cycle when the real boundary is unavailable,
intermittent, visual, platform-specific, or hardware-dependent. In those cases, record the attempted
reproduction and missing environment, add the narrowest deterministic coverage that remains valid,
and keep the required native, GPU, FFI, remote-system, or operator evidence explicitly outstanding.

Compilation, deterministic tests, simulated or Null backends, native platform runs, GPU or hardware
runs, remote CI, and operator observations are separate evidence classes. Report only the class
actually exercised; success at a weaker boundary does not establish a stronger one.

## Layout

```text
crates/*/src/** #[cfg(test)]  unit tests
tests/api/                    public headless API tests
tests/smoke/                  startup executables without windows
tests/platform/               isolated GLFW backend probes without Vulkan
tests/gpu/                    Vulkan/display tests
tests/support/                backend-neutral public test fixtures
examples/                     visual runnable examples
```

## Commands

```bash
just test-unit
just test-api
just test-smoke
just test-platform
just test-platform-compile
just test-platform-null
just test-platform-wayland
just test-platform-x11
just input-test x11
just test-gpu
just test
just doctest
```

`just test` runs unit, API, smoke, and portable platform tests (error conversion and the GLFW Null
backend). It excludes native platform probes, GPU/display tests, and doctests. `just validate` adds
the workspace build, formatting, Clippy, doctests, and documentation checks.

### Native input contract

Select one backend explicitly; the command checks its prerequisites and runs only the native
backend contract:

```bash
just input-test x11
just input-test wayland
just input-test win32
just input-test cocoa
```

The public VMNL `Window::poll_events()` → `Event`/`Input` scenario is a separate opt-in GPU test.
Its executable client lives in `tests/gpu/input_client` and has only `vmnl` as a direct
dependency. It creates the context/window and observes native events and input snapshots on
its main thread, through the public facade. The separate test driver uses X11/XTEST/XKB for
injection and physical fixtures; neither process calls GLFW directly. The GLFW-specific
`tests/platform` probes remain a separate backend contract.
The client and headless API tests share only the 120 named `vmnl::Key` values in
[tests/support/input_keys.rs](../tests/support/input_keys.rs). This inventory contains no native
identifiers and does not include files from `tests/platform`. Physical XKB/evdev/Win32/Cocoa
correspondences remain in the injectors' table under `tests/platform/src/input_key_table.rs`;
the X11 driver uses it to select each key available in the active XKB map. It also covers XTEST
pointer motion delivered as `MouseMoved`,
each available X11 mouse-button mapping, and both scroll axes/directions. Missing physical
key names/aliases and mouse mappings above the server limit are explicitly excluded; the hosted Xvfb runner exposes 10
server buttons, so VMNL `Button7` and `Button8` are not qualified there:

```bash
just input-test-vmnl x11
```

The launcher builds the client and obtains its executable path from Cargo's JSON artifacts,
including a custom `CARGO_TARGET_DIR`. For direct Cargo invocation of the ignored GPU scenario,
first build `vmnl-input-client` and set `VMNL_INPUT_CLIENT` to that executable; an unset path
fails with guidance. The driver gives only the client process `XDG_SESSION_TYPE=x11`, removes
its `WAYLAND_DISPLAY`/`WAYLAND_SOCKET`, and confirms the client's title and PID in the X11
window list before injecting. The host session environment is not modified.
Startup uses `MAPPED` → native pointer preparation → `START` → `HELLO`. The driver confirms
X11 title/PID and live geometry, posts an actual XTEST inside-window pointer move, and verifies
its root position. The client must then confirm focus/hover and report its public cursor position
before any scenario begins; a cached public position alone is not readiness evidence.

The client greeting uses protocol version 4. Key commands contain only the expected public
key name and press/release or mapping-check action; XKB injection codes remain private to the driver.
Each action has a numbered `ARM` → `READY` → native injection → `INJECTED` → checks → `OBSERVED` exchange over pipes. The
client clears the previous event batch before `READY`, except when a control scenario explicitly
inspects that batch. It waits for the numbered `INJECTED` barrier before observing the new inputs.
The driver waits for all press checks
and their input snapshots before issuing release. Missing, malformed, failed or out-of-order
records, client crashes and timeouts fail the test. Failure reasons are sanitized and capped at 1024
Unicode scalars so one diagnostic stays below the protocol line limit; stdout/stderr are continuously drained
with bounded capture. Success requires `DONE` and a successful client exit. On failure the
driver kills/reaps its client, releases injected held inputs and attempts XKB lock restoration.

The terminal report groups checks under `Keyboard` (one block per key) and `Mouse` (one block
per button, plus pointer and scroll). `Press` and `Release` verify event delivery and metadata;
their separate `input states` checks verify `is_down`, `is_pressed` and `is_released`.
Scroll commands use the four `(0, ±1)` / `(±1, 0)` tokens to select an axis and direction,
not to prescribe backend offset units. The public client requires one or more scroll events;
each must have a finite nonzero magnitude on that axis with the expected sign. Every scroll
event in the collected trace is checked individually; deltas are not summed, so opposite values
cannot cancel. Any invalid scroll payload fails immediately after the poll that observes it.
After the first valid scroll, the client continues polling until 100 ms have passed without
another scroll event; each additional valid batch restarts that quiet interval. Unrelated event
kinds do not restart it. Every batch is validated before testing completion, including a batch
at the quiet-interval boundary. A fixed five-second deadline bounds the entire observation,
including initial delivery and settling; absent input, a continuous stream or input arriving too
late to settle fails. Focus must remain held through the final poll. These timing checks have
deterministic headless coverage using supplied elapsed durations rather than sleeps. Exact GLFW
offsets remain separate backend assertions.
`Scancode` requires `Context::get_key_scancode`, captured before the first variant's `READY`,
to match at least one scancode actually received for that named key. `KEY_MAPPING` performs
this check after every physical variant has been released. A named key can have several
physical scancodes: PrintScreen exercises `PRSC` and the distinct `I218` fixture when the
active XKB map assigns it the Print symbol. KpDecimal similarly exercises `KPDL` and the
distinct `I129` fixture when it carries KP_Decimal. RightAlt also exercises the distinct
`LVL3` and `MDSW` fixtures when they carry ISO_Level3_Shift and ISO_Level5_Shift.
Each alternate has its own press/release
and input state checks; an unavailable alternate is explicitly excluded.
Release checks compare the same key and scancode with the acknowledged press. Comparisons use
the opaque `Scancode` type, without arithmetic or positivity requirements. A missing query
mapping after a matching named-key press fails explicitly; it does not exclude the input.
The headless facade and client tests check that the neutral inventory contains 120 distinct
named VMNL keys, excluding `Unknown`, using `Eq`/`Hash` without numeric casts or ordering.
An exhaustive match generated from the same literal names makes adding a public `Key` variant
fail test compilation until the inventory is updated. These inventory checks do not require
contiguous or ordered discriminants.
Each check reports `PASS`, `FAIL` with its diagnostic, `EXCLUDED` with the X11 mapping reason,
or `NOT RUN` when an earlier failure stopped execution. The summary counts checks, including
environment readiness, rather than distinct input devices or keys; exclusions and unexecuted
checks never count as passes. Returned errors and assertion panics print the report and retain
the failing exit status; an incomplete report also fails the command. Formatting, failure
accounting, physical-name selection and Right Alt fallback priority are tested without a display.
The separate-client version adds `Client shutdown` to the previous 640 checks, and the
four physical alternates add sixteen checks, for 657 in the protocol-2 workload;
the historical extended protocol-3 workload plans 848 checks; protocol 4 plans 849 checks
with the separate focus-receiver preparation acknowledgement;
the shutdown check rejects extra records after `DONE` and unsuccessful exits. Headless driver
fixtures verify that injection waits for `READY` and release waits for the complete press
observation, and exercise crashes, timeouts, malformed records and large output on both pipes.

Focus preparation uses two numbered observations. `PREPARE_FOCUS` creates, presents and
hides the receiver, requests primary focus, then acknowledges `Receiver prepared`.
`FOCUS_READY` emits `READY` before the driver prepares the native pointer. The driver reads
actual focus, hover ancestry, geometry and pointer coordinates; it injects XTEST motion to an
interior point derived from the current native geometry. A geometry, focus, hover or pointer
change resets its 100 ms settling interval and requires a new placement when focused.
A request acknowledgement alone never qualifies the pointer. A fixed 4.9-second native
preparation deadline reserves the existing final 100 ms public check from the five-second
budget. After `INJECTED`, the client requires focus and hover throughout that 100 ms check;
it does not warp the cursor or recover a lost readiness condition during that assertion.
Only after `Receiver ready` may the driver hold A and the left button for focus loss.
The deterministic readiness tests replay a warp followed by reconfiguration and an outside
pointer, reject missing motion and late readiness, and verify focus/hover/geometry resets.
They run without a display in the platform suite as well as the GPU driver tests.

An opt-in native regression exercises just these two barriers and receiver cleanup through the
same client and driver, independently of text/modifier fixtures. After building
`vmnl-input-client`, set `VMNL_INPUT_CLIENT` to its Cargo-reported executable path and run:

```bash
cargo test -p vmnl-gpu-tests --locked --test native_input \
  vmnl_public_native_focus_preparation -- --ignored --exact --nocapture
```

This command moves pointer/focus and needs the same X11/Vulkan/XTEST prerequisites. It is
explicitly excluded from `just test-gpu` alongside the full native input scenario. Its success
qualifies receiver preparation/cleanup only; it does not replace the complete input scenario.

This requires the X11 prerequisites above, XTEST, and a Vulkan-capable GPU with a Vulkan loader.
The scenario presents a first frame before requesting focus so an Xwayland window has a buffer
the compositor can display. It resolves physical XKB names and aliases, not layout characters
or runtime scancode lookups, to choose the injected key and independently expected VMNL key.
The client queries only the public facade for scancode metadata. This checks query/event and
press/release consistency in one context with a stable keyboard layout; it does not establish
the correctness of an exact native numeric encoding. A shared query/event encoding defect
could pass this check. Raw backend encoding and translation checks remain in the unit/platform
suites. Headless client regressions cover zero and negative values, missing/mismatched mappings,
release without a matching press, query selection among observed physical variants, and
rejection of key commands containing native codes.
Fresh XTEST key presses are separated by 5 ms so two legitimate presses cannot receive the
same millisecond X11 timestamp and be filtered as an input-method duplicate. This pacing stays
inside the OS driver; no native timestamp or keycode is sent to the public client.
Canonical `RALT` is preferred to the backend's `LVL3`/`MDSW` fallback. The driver retains
the existing `x11rb` version and its `xkb` feature for this lookup.
The test rejects an initially held native key and checks that no VMNL key remains down after
each release. The injector verifies the pointer's root position after posting its motion;
the client must independently receive `MouseMoved` and observe the matching public getter.
Lock keys are tested as individual keys; the initial XKB locked-modifier mask is
restored and verified before the mouse-button cases. Cleanup releases every currently injected
key/button in reverse hold order and attempts lock restoration on an error or assertion panic.
The protocol-3 scenario additionally observes the following contracts through the public client:

- All 15 nonempty subsets of the four left standard modifiers and the four individual right
  modifiers. Shift/Control chords target A; Alt/Super chords use modified Button4 to avoid
  window-manager left-click grabs. Key and button callbacks verify modifier flags and held
  snapshots; each release preserves the observed opaque press scancode.
- Caps Lock and Num Lock flags on key and button events, both toggle phases and the public
  reporting-mode getter. XKB locks are temporarily cleared for independent expectations and
  the original locked-modifier mask is restored and verified after the extended cases.
- Text `a`, `é` and `🦀`, independently selected from unmodified group-1/level-1 XKB symbols
  with active group 0. Missing scalars are explicitly excluded; no keymap/input method is
  installed or changed. Both ordinary and legacy modified-text payloads are checked separately
  from physical key identity; this does not enumerate Unicode or qualify IME composition.
- A bounded 1500 ms A hold with actual native repeats. Repeat batches preserve down state
  without reporting a new press; disabled per-key/global repeat or excessive delay is excluded
  from the OS controls captured before injection. Posted repeat events are not substituted.
- Real key/button press and release in a single `Window::poll_events`, non-consuming queries,
  next-empty-poll clearing, held-state preservation and `clear_input_transitions`. The driver
  completes both checked OS operations before `INJECTED`; the client performs one poll for
  the same-batch assertion. No fabricated VMNL event or private reducer is used.
- Disabled key/button event delivery with continuing native snapshot updates, followed by
  re-enabled delivery and acknowledged events.
- Pointer leave/enter with the public hover getter, and held A/left-button focus loss with
  synthetic release events/snapshots. VMNL does not prescribe the relative callback order across
  focus, keyboard and mouse sources; each release must match its acknowledged hold.
  The driver first prepares a second VMNL window before holding inputs. Focus transfers to
  that receiver; both windows remain owned by the client. The trace is checked before physical
  cleanup; the receiver is then destroyed and the primary window regains focus and hover.

The retained protocol-3 execution evidence and environment limits are recorded below. A mapped key
intercepted by a compositor shortcut still fails; missing delivery is not converted into an exclusion.
Keep the desktop idle during injection: external keyboard/pointer activity can change focus or
add events and must not be reported as a passing input sequence.
It is excluded from `just test-gpu` so running the general GPU suite does not inject input into the
active desktop. The command fails if the display prerequisites are unavailable; GPU initialization
and event/state assertions fail inside the test when their runtime requirements are not met.

On 2026-10-12, an automatic local execution passed `1/1` on Fedora 44, Hyprland 0.56.2 with
Xwayland/X11, Intel Iris Xe (ADL GT2), Mesa 26.2.3 and Vulkan loader 1.4.341, after the Xlib instance
extension, first-frame readiness and physical-A fixes. The retained result was:

```text
X11 pointer mapping supports server buttons 1 through 10; unsupported VMNL buttons are not injected: [(11, Button7), (12, Button8)]
test vmnl_public_native_keyboard_and_mouse_events_update_input ... ok
test result: ok. 1 passed; 0 failed; 0 ignored
```

This exercises physical A press/release, scancode and batch snapshots, pointer motion/position,
GLFW mouse buttons 1–6, and both scroll axes in both directions through the public facade.
It does not qualify buttons 7–8, every key, text, repeat, enter/leave, or public focus-loss batches,
other display/GPU stacks, or human manual graphical validation.

On 2026-10-13, the expanded public X11 scenario passed all five checks for 118 of the 120
named keys on the same Fedora 44 / Hyprland 0.56.2 / Xwayland stack (Mesa 26.2.3, Vulkan loader
1.4.341). `World2` has no X11 physical-name fixture; `F25` has no `FK25` name/alias in the active
XKB map. Buttons 7–8 remain outside the ten-button pointer map. The final terminal summary was:

```text
Summary:
  Checks:   640
  PASS:     622
  FAIL:     0
  EXCLUDED: 18
  NOT RUN:  0
  Result:   PASS
```

The 18 excluded checks are five per excluded key and four per excluded mouse button; the 622
passes include readiness and verified XKB lock restoration. All tested keys finish released.
The first attempt timed out on the existing pointer-motion check before keyboard injection;
a diagnostic run and the final run without instrumentation both passed. The cause of that
initial failure is not established. These results qualify only individual named-key
press/release, scancodes and snapshots, plus the existing mouse cases. Text, modifier
combinations/lock flags, repeat, unknown keys, enter/leave, focus loss, same-batch taps and other
platforms/environments remain outside this public scenario's runtime evidence.

On 2026-10-13, the separate-client version passed three consecutive automatic executions on
the local X11 display: each reported 641 checks, 623 `PASS`, 18 `EXCLUDED`, zero `FAIL` and zero
`NOT RUN`. The additional pass checks successful client shutdown. The driver confirmed the
client's X11 title/PID before injection. An immediate post-XTEST position query initially still
reported the old root coordinates; delivery verification now waits up to 500 ms for the target,
while the client independently checks `MouseMoved` and the public position getter. These runs
do not establish the cause of the earlier event timeout or qualify another backend's numeric
scancode representation. They are automatic evidence, not operator graphical validation.

On 2026-10-14, protocol version 2 passed three consecutive automatic local X11 executions
without instrumentation: 657 checks, 639 `PASS`, 18 `EXCLUDED`, zero `FAIL` and zero `NOT RUN`.
The four additional physical variants (`PrintScreen / I218`, `KpDecimal / I129`,
`RightAlt / LVL3` and `RightAlt / MDSW`) each passed event and input-state checks for press
and release. The public query matched an actually observed variant for each logical key;
every variant preserved its own press/release scancode. Earlier strict per-press equality
failed on legitimate query/event pairs (PrintScreen 218/107, KpDecimal 129/91 and RightAlt
203/108), demonstrating why one-to-many mappings require this group check. A pointer attempt
also failed before keyboard injection with an unchanged X11 root position; its cause remains
unestablished. These final runs qualify metadata consistency on this stack, not exact backend
numeric encoding, cross-environment stability or human graphical validation.

X11 needs `DISPLAY`, `xprop`, and an EWMH window manager; the probe checks XTEST when it injects
input. Wayland needs a live `WAYLAND_DISPLAY` socket and the X11 parent display used by XTEST.
This probe requires nested Weston using its X11 backend on that parent display and the default
desktop shell with its top panel; it does not run on a standalone Wayland seat. Windows needs an accessible interactive input desktop, and the target
must run at the same integrity level. Cocoa checks event-posting access and reports when the
terminal or app running `just` needs Accessibility permission.

An unavailable host backend, display, compositor socket, window manager, or required permission
fails before the input sequence runs. The native contract still fails if XTEST, focus, hover, or
actual OS-to-GLFW event delivery is unavailable. This command is opt-in and is never part of
`just test` or `just validate`.

### Extended public client execution (protocol 3)

On 2026-10-14, three consecutive local X11/Vulkan executions without instrumentation passed:
848 checks, 828 `PASS`, 20 `EXCLUDED`, zero `FAIL`, zero `NOT RUN`. This is automatic public
VMNL evidence on the Fedora/Hyprland/Xwayland stack recorded above. All 19 modifier cases,
Caps/Num key and mouse flags, ASCII `a` through both text sources, actual A repeat, key/mouse
poll controls, pointer boundaries and the two-owned-window focus cycle passed. The exclusions
are World2/F25 (ten checks), buttons 7–8 (eight) and unmodified `é`/`🦀` text (two). Neither a
new layout nor an input method was installed; non-ASCII native text remains unqualified here.

Earlier attempts exposed a duplicated native Control press timestamp (`16870183` twice),
followed by a missing public callback. A 5 ms XTEST press spacing removed duplicate timestamps
in the diagnostic run while preserving strict event/state assertions. Iconification was ignored
(`focused=true`, `iconified=false`); hiding with a held button produced an additional focus-gained
callback even after 100 ms settling. The final focus case uses a prepared owned VMNL receiver
and verifies releases before native cleanup, then destroys that receiver and restores the primary.
A startup attempt reported an inside public cursor position with hover false; the explicit native
startup pointer preparation precedes the final three passes. These observations do not establish
a general cause for the older intermittent Xwayland pointer delivery failures or qualify other
compositors, native seats, GPU/display stacks, IME composition, physical devices or human
manual graphical validation.

### Scroll oracle native revalidation

On 2026-10-15, `just input-test-vmnl x11` passed three consecutive executions without
instrumentation after the scroll-oracle changes: 848 checks, 828 `PASS`, 20 `EXCLUDED`,
zero `FAIL` and zero `NOT RUN` each. The native test took 4.36 s, 4.40 s and 4.36 s.
Readiness and all four scroll directions passed with the 100 ms quiet interval and fixed
five-second observation deadline. Headless regressions separately cover invalid later batches,
quiet-interval resets and timeout precedence; the native passes exercise the nominal public path.

The observed environment was Fedora 44, kernel 7.2.8-200.fc44.x86_64, Hyprland 0.56.2,
Xwayland (`DISPLAY=:0`) and Mesa 26.2.3. The workspace was based on `e519fa7`, with the
four scroll-oracle corrections; the client source `tests/gpu/input_client/src/lib.rs` had SHA-256
`bd619d527440cec51ce290e6b0fb1688dfbbb167b50b1c45978419311b67668b`.
The exclusions remained World2/F25, buttons 7–8 and unmodified `é`/`🦀` text.

The first revalidation attempt passed readiness and all four scroll checks, then failed text:
the fixture injected X11 keycode 24 expecting `a`, while VMNL received `q`. A second execution
with read-only keymap measurements passed; its map stayed at keycode 24 → `q`, 38 → `a`.
The three final passes above followed that diagnostic execution and used the same client source.
The cause of the initial text-fixture mismatch and the older intermittent pointer-preparation
failures remains unestablished. These results qualify this automatic local execution, not other
backends, layouts, hardware or human graphical validation.

### Extended native cases (issue #91, stage 3)

The selected backend contract now follows the existing representative operations with serialized
`native-input-case` processes. Schema 2 adds an eligibility manifest and named per-case results;
the previous Win32/Cocoa qualification count does not apply to this schema and expanded workload.
The manifest contains all 120 named keys, native codes/scancodes, eligible cases, and explicit
reasons for non-injectable or not-yet-qualified cases. These entries are not counted as passes.

Physical key fixtures are fixed literals audited against the bundled GLFW 3.4 key tables. Linux
resolves XKB physical names and aliases without changing the layout. Nested Wayland selects the
parent X11 code as the expected evdev code plus eight, requiring that code in the parent XKB range;
Weston forwards raw codes independently of parent key names. This also reaches PrintScreen through
parent code 218 / evdev 210 rather than the X11 PRSC position at code 107. Windows posts scan codes, with
extended-key flags and a separate Pause virtual-key path; Cocoa posts hardware key codes.
Modified-button cases acknowledge both mouse callbacks before releasing modifiers, preventing
Win32 keyboard/mouse message scheduling from ending the modifier hold prematurely. Their getter
snapshot must show modifiers pressed and the target button released before cleanup. Win32 Shift
uses side-specific virtual keys, preserving the fixed physical scancode while making the side's
key state visible to GLFW's stuck-Shift repair. Num Lock uses `VK_NUMLOCK` with the extended
physical scancode so the injection updates the OS toggle state as well as generating callbacks.
The CI X11 profile removes only the distro's bare Print shortcut from a temporary Openbox
configuration passed by absolute path because Openbox changes its working directory during startup,
retaining that configuration as an artifact. Otherwise its screenshot grab prevents
PrintScreen from reaching GLFW. Local X11 qualification requires the same absence of a Print grab.
Each case checks exact ordered identity/scancodes, final released states, focus, and a completion
marker before a 100 ms settling interval rejects extra callbacks. Readiness and event observation
are bounded by five seconds, with a seven-second parent-process watchdog. Injector errors retain
a JSON failure and release tracked keys/buttons; interrupted double-tap lock cases restore the
missing toggle.

Additional families cover:

- All nonempty subsets of the four left standard modifiers, plus the four right modifiers;
  Shift/Control chords use A as a key target. Alt/Super chords use bare modifier key events and
  modified-button callbacks to avoid application or desktop key shortcuts.
- Modified left-button press/release for Shift/Control and Win32/Cocoa chords. Linux Alt/Super
  chords use side Button4 (server button 8); default Openbox Alt+left and Weston Super+left grabs
  are explicitly recorded as non-injectable left clicks. A map without server button 8 records
  that side-button case as non-injectable too. Caps Lock and Num Lock reporting
  on key and button events on X11, nested Wayland, and Win32, with two taps restoring each lock.
  Separate lock taps by 30 ms so the X11/XIM same-timestamp duplicate filter cannot discard the
  second press; both press/release pairs remain required.
- Text `a`, `é`, and `🦀`, observed separately from physical keys via character and deprecated
  modified-character callbacks. Win32 uses UTF-16 packets and Cocoa sets Unicode event strings.
  Linux only posts scalars present at unmodified group 1 / level 1 in the current parent map;
  missing scalars are explicitly non-injectable. No keymap or input method is installed or changed.
- A bounded 1.5-second hold requiring actual repeat callbacks on X11/nested Wayland. Win32/Cocoa
  event posting does not provide a qualified hardware repeat source; synthesized repeated presses
  would not prove that contract and are recorded as non-injectable for this profile.
- A held A and left button followed by hiding the X11 window, or iconification on other backends:
  require both getters to report pressed,
  focus loss, synthetic releases in GLFW order, and released getters before OS-input cleanup.
  Freeze the trace after a 100 ms settling interval, then acknowledge cleanup without polling
  its extra mouse-up callbacks into the synthetic-release phase.

Win32 boundary readiness requires 100 ms of focused hover with an actual inside-client pointer;
re-entry uses the client's confirmed screen-space center rather than a sampled desktop cursor.

Win32 bare Win-key release can open Start and remove focus; these keys are explicitly excluded
from this profile. Cocoa Caps Lock and lock-modifier restoration remain not yet qualified.
Any mapped key that fails delivery, focus, event order, or cleanup fails the test rather than being
reclassified as unsupported. [CI run #38206068512](https://github.com/VMNL/vmnl/actions/runs/38206068512)
at `32f8b3e` retained 161/161 extended X11 cases, 161/161 nested-Wayland cases, 142/142 Win32 cases,
and 152/152 Cocoa cases, with no error records. These totals match each eligibility manifest and
include respectively 118, 118, 116, and 110 named-key cases. Exclusions are not passes, and the
Win32/Cocoa probes remain experimental until the ten-run threshold is met.
Headless oracle tests and compilation are not native evidence. VMNL batch-transition and
`Window::poll_events()` qualification beyond the recorded public X11 scenario remains environment-specific.

## Continuous Integration

CI build and test stages invoke Cargo directly. The documentation job uses pinned Just tooling
for API-book verification; the local aggregate `just validate` is not a CI entry point.

Its job dependencies are:

```text
Quality (format -> Clippy)
  -> Validation per OS (build -> unit -> API -> smoke -> platform)
Audio per OS (compile all audio test targets -> device-independent audio tests)
Validation + Audio -> Documentation
```

Each OS validation job reuses one Cargo target directory for compilation and every test stage; no
target directory is cached or transferred between runners. Every OS runs the full portable
`vmnl-platform-tests` suite with `cargo test -p vmnl-platform-tests --locked`, including error
conversion, the GLFW Null backend, the portability inventory and applicable headless tests for probe
helpers. `selected_backend_contract` requires a display, so Cargo skips it in this invocation and the
native stages enable it explicitly. Linux then forces the GLFW Wayland
backend under Weston with Pixman nested on Xvfb with Openbox, then tests the GLFW X11 backend
under Xvfb with Openbox. Both paths run visible keyboard and mouse probes. Separate probe
processes wait for readiness, inject `A`, Shift+A, and left-button press/release through XTEST, and
require the exact native GLFW event sequence before timeout. A Linux-only button probe reads the
X11 pointer mapping and injects only server buttons within its reported limit, checking the
resulting GLFW button press/release order and released final states. The Windows probe checks
SendInput for GLFW buttons 1–5; the Cocoa probe checks CGEventPost for all eight GLFW button
numbers. Both also check native pointer motion, leave/enter, and vertical/horizontal scroll
direction. The hosted Xvfb mapping
currently exposes 10 buttons. CI run
[#38073455968](https://github.com/VMNL/vmnl/actions/runs/38073455968) showed XTEST rejecting
server button 11 (`BadValue`), so GLFW `Button7` and `Button8` are not qualified by hosted CI.
CI run [#38091799251](https://github.com/VMNL/vmnl/actions/runs/38091799251) stopped at the
nested Weston hover-boundary case, before X11. The test moved outside the compositor's X11 output
and received no Wayland leave/enter events. Weston 13.0.0's
[`clear_pointer_focus()` implementation](https://github.com/wayland-mirror/weston/blob/13.0.0/libweston/input.c#L2533-L2537)
is empty, so exiting that output does not clear the client's pointer focus. The server trace's
later background enter occurred after the probe exited, not during the injected path.

The corrected Wayland probe crosses from the GLFW content area to the default desktop shell's top
panel inside the Weston output, waits 100 ms, then returns to the original position using XTEST.
It requires the default desktop shell with its top panel; output bounds and both pointer targets
are checked and logged. The Linux job in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980) passed this ordered leave/enter pair on both
backends, all 17 Wayland operations, and all 19 X11 operations. The eligible GLFW buttons 1–6,
pointer motion, and four scroll directions passed; buttons 7–8 are recorded as not injectable
with the hosted 10-button map. Windows and macOS remain experimental and non-blocking.

The scroll probe checks vertical up/down and horizontal positive/negative in order. X11 injects
buttons 4/5 and 6/7 and expects offsets `(0, +1)`, `(0, -1)`, `(+1, 0)`, and `(-1, 0)`. Cocoa's
native delta magnitude is backend-specific, so its probe checks axis and direction. Mouse probes
require the pointer to hover the focused window. The Wayland path injects through the parent Xvfb
server into Weston's X11 backend; it does not qualify a standalone Wayland compositor seat.

CI invokes Cargo directly for these selected backend contracts; it does not call the local
`just input-test` recipe. Linux X11 and nested Weston runs block CI. Windows and macOS native runs
remain visible and non-blocking until they meet the documented qualification threshold.
The Wayland NoApi probe attaches a zero-filled shm buffer so its surface can be mapped without
Vulkan. It signals MAPPED after attaching the buffer; the parent finds the Weston X11 window by
WM_CLASS and clicks its center until GLFW confirms keyboard focus and signals READY. Before
injection, the parent also requires Weston to hold the parent X11 keyboard focus and reacquires it
if necessary. The mouse probe drains activation clicks before READY and the parent does not issue
another activation click after READY, so activation cannot satisfy the tested button sequence.
MAPPED or Wayland READY alone never authorizes input injection.

Visible X11 NoApi input probes fill their initial drawable with Xlib before requesting focus.
This supplies contents needed for mapping on Hyprland/Xwayland without initializing OpenGL or
Vulkan. The test-only helper borrows GLFW's live display/window, releases its graphics context
and synchronizes the drawing request. Preparation failures produce an error JSON record;
successful drawing alone never signals READY or bypasses focus/hover checks.

After acquiring focus, X11 mouse and extended probes signal `POINTER <window-id>`. The parent
checks that the native window belongs to its child PID and has X11 focus, queries its live geometry,
then posts an inside-window XTEST motion and waits up to 500 ms for the requested root coordinates.
Only then does it acknowledge `POSITIONED`. The probe continues processing events while waiting;
after the acknowledgement it requires focus/hover and drains preparation events before `READY`.
The parent rejects premature `READY`; it sends no preparation motion after `READY`. A GLFW
`set_cursor_pos`/XWarpPointer alone can update X11 coordinates without delivering a crossing on
the local Hyprland/Xwayland stack. The X11 held-input focus case hides its owned window to obtain
native focus loss because iconification can be ignored there. Its strict event order, synthetic
releases and released getters remain required before physical-input cleanup.

On 2026-10-15, three consecutive automatic `just input-test x11` executions passed on the local
Fedora/Hyprland 0.56.2/Xwayland stack: all 19 baseline operations and all 161 eligible extended
cases, including 118 named keys and held-input focus loss. The eligibility manifest records 18
non-injectable rows with reasons; the button operation separately records buttons 7–8 above the
active X11 pointer-map limit. The first diagnostic passed pointer preparation but exposed ignored
iconification; a targeted native unmap observed synthetic releases before OS cleanup, followed by
the three complete passes. This is automatic NoApi evidence for this local profile, separate from
the public VMNL/Vulkan runs, other backends and physical or human graphical validation.
A later completion recheck missed all callbacks in the legacy A probe despite reported focus.
Two isolated diagnostic runs and the next complete X11 run passed; the intermittent failure's
cause remains unresolved.

Win32 uses `SendInput` for keyboard, button, motion, and scroll injection; `SetCursorPos` moves
outside and back into the window for its hover-boundary case. Cocoa uses `CGEventPost` for keyboard,
buttons, motion, hover-boundary, and scroll injection. These probes remain visible but non-blocking
until ten consecutive successful runs use the same runner image, GLFW revision, injector, and probe
schema; any of those changes resets the count. The explicit Core Graphics flags fixed Cocoa Shift+A
in run #38082851338. Scroll injections on Win32 and Cocoa are separated by 100 ms so GLFW can
process each message before the next reversal. Pointer-motion probes discard callbacks that report
no coordinate change. Cocoa scroll injection uses pixel deltas and matches the variadic Core
Graphics signature and axis count; all four directions passed in run #38109958980. Additional
modifier combinations, eligible text, Linux repeat, and focus loss passed in the schema-2 run
recorded above. Cocoa lock restoration, Win32/Cocoa hardware repeat, Linux text scalars absent from
the active map, and public VMNL runtime remain unqualified or non-injectable as recorded per case.

The documentation job runs only after all OS validation and audio jobs succeed. Its pinned API
tools are cached by platform, architecture, and installer-script hash, and the installer still
verifies every restored tool version before use.

CI sets `CARGO_INCREMENTAL=0` because GitHub-hosted jobs use fresh workspaces. This avoids
producing incremental artifacts that cannot be reused by later jobs.

GPU/display tests remain excluded from hosted CI because they require a compatible GPU, driver,
and display server.

## Invariants

- API tests must be headless.
- Smoke tests must not open a window.
- Platform probes must use `ClientApi::NoApi`, run one operation per subprocess, and emit one
  versioned JSON record. Missing output, unexpected backend, non-zero status, signal, or abort is
  a failure.
- Native-input probes must acquire focus before signaling readiness, receive the injected press and
  release in order, and fail non-zero on focus failure, wrong input, timeout, crash, or abort.
- GPU tests must be isolated under `tests/gpu`.
- Visual examples must live under `examples`.
- Tests must assert behavior or fail through a non-zero exit code.
- `just test` combines the unit, API, and smoke suites.

## Classification

Use this decision order:

```text
tests private/local implementation detail -> unit test
tests vmnl as an external user would      -> API test
initializes and exits without window      -> smoke test
needs Vulkan/display/window               -> GPU test
needs only a native GLFW window            -> platform test
demonstrates visual user workflow         -> example
```

## GPU Tests

GPU tests are separated because they depend on the machine:

- Vulkan loader.
- GPU driver.
- Display server.
- GLFW window creation.

Compile without running them:

```bash
just test-gpu-compile
```

Run them explicitly:

```bash
just test-gpu
```
