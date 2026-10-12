# Testing

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

`just test` intentionally excludes platform, GPU/display tests, and doctests. `just validate` adds
the portable error-conversion and GLFW Null-backend suite after smoke tests.

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
It currently covers X11 `A`, XTEST pointer motion delivered as `MouseMoved`, each mouse button
whose X11 server mapping is available, and vertical/horizontal scroll in both directions. It
logs and skips mappings above the server's reported limit; the hosted Xvfb runner exposes 10
server buttons, so GLFW `Button7` and `Button8` are not qualified there:

```bash
just input-test-vmnl x11
```

This requires the X11 prerequisites above, XTEST, and a Vulkan-capable GPU with a Vulkan loader.
It is excluded from `just test-gpu` so running the general GPU suite does not inject input into the
active desktop. The command fails if the display prerequisites are unavailable; GPU initialization
and event/state assertions fail inside the test when their runtime requirements are not met.

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
configuration, retaining that configuration as an artifact. Otherwise its screenshot grab prevents
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
- A held A and left button followed by iconification: require both getters to report pressed,
  focus loss, synthetic releases in GLFW order, and released getters before OS-input cleanup.
  Freeze the trace after a 100 ms settling interval, then acknowledge cleanup without polling
  its extra mouse-up callbacks into the synthetic-release phase.

Win32 boundary readiness requires 100 ms of focused hover with an actual inside-client pointer;
re-entry uses the client's confirmed screen-space center rather than a sampled desktop cursor.

Win32 bare Win-key release can open Start and remove focus; these keys are explicitly excluded
from this profile. Cocoa Caps Lock and lock-modifier restoration remain not yet qualified.
Any mapped key that fails delivery, focus, event order, or cleanup fails the test rather than being
reclassified as unsupported. Schema-2 runtime results require a new qualified display execution;
headless oracle tests and compilation are not native evidence. VMNL batch-transition and
`Window::poll_events()` runtime qualification remain in stage 5.

## Continuous Integration

The CI workflow invokes Cargo directly. The Justfile is a local development helper and is never
invoked in CI.

Its strict job order is:

```text
Quality (format -> Clippy)
  -> Validation per OS (build -> unit -> API -> smoke -> platform)
  -> Documentation
```

Each OS validation job reuses one Cargo target directory for compilation and every test stage; no
target directory is cached or transferred between runners. Linux then forces the GLFW Wayland
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

Win32 uses `SendInput` for keyboard, button, motion, and scroll injection; `SetCursorPos` moves
outside and back into the window for its hover-boundary case. Cocoa uses `CGEventPost` for keyboard,
buttons, motion, hover-boundary, and scroll injection. These probes remain visible but non-blocking
until ten consecutive successful runs use the same runner image, GLFW revision, injector, and probe
schema; any of those changes resets the count. The explicit Core Graphics flags fixed Cocoa Shift+A
in run #38082851338. Scroll injections on Win32 and Cocoa are separated by 100 ms so GLFW can
process each message before the next reversal. Pointer-motion probes discard callbacks that report
no coordinate change. Cocoa scroll injection uses pixel deltas and matches the variadic Core
Graphics signature and axis count; all four directions passed in run #38109958980. Additional
modifier combinations, text, repeat, and public VMNL runtime cases remain unqualified.

The documentation job runs only after all OS validation jobs. Its pinned API tools are cached by
platform, architecture, and installer-script hash, and the installer still verifies every restored
tool version before use.

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
