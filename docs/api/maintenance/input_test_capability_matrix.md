# Cross-platform keyboard and mouse input test matrix

Tracks issue [#91](https://github.com/VMNL/vmnl/issues/91) at baseline `4caa319`.
The matrix records checked-in assertions separately from test execution and from evidence
through a native GLFW backend or the public VMNL path. No test result is inferred from source
presence or successful compilation.

## Status and evidence levels

- `not yet qualified`: the issue's complete contract has no recorded result at this baseline.
- `not injectable`: the selected backend/injector cannot produce the input; record the reason
  for that row.
- `unsupported`: the backend explicitly does not provide the capability.
- `failure` and `qualified/pass`: reserved for retained probe results.
- Deterministic unit/API evidence, native `NoApi` evidence, and public VMNL GPU/display evidence
  are separate. A native GLFW-only result does not qualify `Window::poll_events()` →
  `Event`/`Input`.

## Native profiles and prerequisites

| Profile | Backend and injector | Prerequisites and limits | Current native path |
| --- | --- | --- | --- |
| K | X11 / XTEST | Visible mapped window, confirmed focus, active keyboard layout; unavailable keys and intercepted shortcuts need a recorded reason. | 118 eligible named-key cases and the eligible extended keyboard families passed directly through GLFW in [schema-2 CI run #38206068512](https://github.com/VMNL/vmnl/actions/runs/38206068512). The representative public A scenario passed locally (stage 5); other public key families remain unqualified. |
| K | Nested Weston / parent X11 XTEST | Same focus/readiness checks; this exercises XTEST → Xvfb → Weston X11 backend → Wayland client, not a standalone Wayland seat. | 118 eligible named-key cases and the eligible extended keyboard families passed directly through GLFW in [schema-2 CI run #38206068512](https://github.com/VMNL/vmnl/actions/runs/38206068512). Public VMNL runtime remains unqualified. |
| K | Win32 / `SendInput` | Active desktop at matching integrity; keyboard layout and system shortcuts constrain eligible keys. | 116 eligible named-key cases and the eligible extended keyboard families passed directly through GLFW in [schema-2 CI run #38206068512](https://github.com/VMNL/vmnl/actions/runs/38206068512). Experimental pending ten consecutive qualified runs; public VMNL runtime remains unqualified. |
| K | Cocoa / `CGEventPost` | Active desktop; macOS may require Accessibility permission. Layout and system shortcuts constrain eligible keys. | 110 eligible named-key cases and the eligible extended keyboard families passed directly through GLFW in [schema-2 CI run #38206068512](https://github.com/VMNL/vmnl/actions/runs/38206068512). Experimental pending ten consecutive qualified runs; public VMNL runtime remains unqualified. |
| M | X11 / XTEST | Mapped, focused, hovered window and recorded X server button map. Server buttons 4–7 are scroll; later server buttons map to additional GLFW buttons. CI Xvfb currently exposes 10 buttons. | GLFW buttons 1–6, motion, leave/enter, and all four scroll directions passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Buttons 7–8 are explicitly not injectable with the hosted 10-button map. |
| M | Nested Weston / parent X11 XTEST | Records nested Weston evidence only; button map and focus must be captured. Requires the default desktop shell with its top panel for boundary crossings. Does not qualify a native Wayland seat. CI parent Xvfb currently exposes 10 buttons. | GLFW buttons 1–6, motion, leave/enter, and all four scroll directions passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Buttons 7–8 are explicitly not injectable with the hosted 10-button map. |
| M | Win32 / `SendInput` | Active desktop at matching integrity. GLFW Win32 maps left/right/middle and XBUTTON1/2, so only GLFW buttons 1–5 are eligible through this mapping. | Eligible buttons, pointer motion, leave/enter, and all four scroll directions passed in [CI runs #38089252090](https://github.com/VMNL/vmnl/actions/runs/38089252090) and [#38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Still experimental pending ten consecutive successes. |
| M | Cocoa / `CGEventPost` | Requires an active desktop and any required Accessibility permission; remaining button mapping must be established by probes. | All eight buttons, pointer motion, leave/enter, and all four scroll directions passed in [CI runs #38089252090](https://github.com/VMNL/vmnl/actions/runs/38089252090) and [#38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Still experimental pending ten consecutive successes. |
| V | Public VMNL path / X11 XTEST | Requires a qualified Vulkan loader, GPU/driver, X11 EWMH display, XTEST, mapped and focused VMNL window. Buttons above the X11 server mapping are not injectable. | `just input-test-vmnl x11` checks A, pointer motion, every mouse-button mapping supported by the active X server, and both scroll axes through `Window::poll_events()` → `Event`/`Input`; representative runtime passed on the local Fedora/Xwayland stack recorded below. |

All native cases must use a bounded deadline, verify readiness before injection, preserve the
observed event order and identity, retain failure diagnostics, and release held inputs during
cleanup. Mouse movement requires an inside-window target; enter/leave cases must cross the
content-area boundary. Scroll directions and coordinate tolerances are backend-specific and must
be recorded. Physical-device qualification, including extra mouse buttons, is separate from
synthetic injection.

## Progress in this branch

- Headless unit assertions now compare every named `Key` and all eight `MouseButton` variants
  against fixed GLFW enum values in both conversion directions.
- Event translation assertions cover press/repeat/release for all 120 named keys, including
  scancode and modifier preservation, and press/release for all eight mouse buttons.
- Modeled `EventQueue` batches cover a press and release for every key/button in one batch, then
  verify transition clearing in the next empty batch. These remain unit-level reducer checks; they
  do not execute `Window::poll_events()` or a native backend.
- The native probes check focused and hovered mouse cases, ordered button and scroll events,
  released button state, pointer motion, and cursor-query agreement. They prove GLFW backend
  delivery only, not the public VMNL path.
- A and Shift+A passed on X11 and nested Weston in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980), and on Win32/Cocoa in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980).
- The nested Weston boundary failure reproduced in run #38091799251 came from exiting the
  compositor's X11 output, where Weston 13.0.0 does not clear pointer focus. Crossing from the GLFW
  content area to the default desktop shell's top panel inside the compositor fixed the probe:
  [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980) records exactly leave then enter on both Linux backends.
- The Linux job in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980) passed 17 Wayland and 19 X11 operations, including pointer movement,
  getter agreement, all four scroll directions, and ordered press/release for GLFW buttons 1–6.
  `GetPointerMapping` reports 10 server buttons; GLFW buttons 7–8 are recorded as not injectable
  because their server mappings 11/12 exceed this limit. Every tested button ends released.
- Linux results are blocking. Windows/macOS remain experimental until the documented ten-run
  qualification threshold is met. Native records are retained as JSONL artifacts; direct GLFW
  probes do not qualify the public VMNL Vulkan/display runtime.
- `just input-test <backend>` checks the selected host, display, compositor/window-manager, and
  available input-permission prerequisites before running one native contract. CI continues to
  invoke the same selected contracts directly through Cargo: Linux X11 and nested Weston block,
  while Win32 and Cocoa remain experimental.
- `just input-test-vmnl x11` adds an opt-in Vulkan-backed public-facade path for A, pointer motion,
  every mouse-button mapping supported by the active X11 server, and vertical/horizontal scroll
  directions. It is excluded from `just test-gpu` because it injects input into the active desktop;
  its representative runtime passed on the local Vulkan/Xwayland stack recorded below. On the hosted 10-button Xvfb, `Button7`/`Button8`
  remain outside the injectable range.

## Stage 3 native expansion

The selected backend contract now enumerates every named key using fixed physical-key fixtures
and emits a schema-2 `native-input-eligibility` manifest before running eligible cases. Each row
records its native code, expected scancode, status and exclusion reason. XKB names/aliases resolve
Linux physical keys independently of layout; nested Wayland selects an available parent code at the expected evdev code plus eight.
No manifest entry or compilation result qualifies native delivery.

The additional cases cover left-modifier subsets and right-side modifiers, modified mouse buttons,
Caps/Num lock reporting on Linux/Win32, three representative text scalars, bounded Linux repeats,
and held-key/button focus loss. See [extended native cases](../../testing.md#extended-native-cases-issue-91-stage-3)
for the exact targets, timing, cleanup and profile limits. Win32 bare Win keys, absent backend key
mappings, unavailable parent evdev transport codes, missing unmodified text scalars, event-poster hardware repeat,
and Cocoa lock restoration remain explicit exclusions or unqualified entries. Alt/Super keyboard
chords observe modifier key events; their flags are qualified by mouse callbacks, not by a target-key
shortcut. Linux Alt/Super mouse chords use side Button4 because the configured window managers
consume modified left clicks; those left-click exclusions are retained explicitly. Focus-loss
observation finishes before physical cleanup can emit an extra Cocoa mouse-up. Physical hardware
and public VMNL evidence remain separate.

[CI run #38206068512](https://github.com/VMNL/vmnl/actions/runs/38206068512) at `32f8b3e` retained
every eligible schema-2 case with `qualified: true`, no error record, and exact agreement between
the eligibility manifest and the passed case names:

| Backend | Named keys | Modifier key cases | Modified button cases | Lock cases | Text cases | Repeat holds | Focus loss | Total passed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| X11 | 118 | 19 | 19 | 2 | 1 | 1 | 1 | 161 |
| Nested Wayland | 118 | 19 | 19 | 2 | 1 | 1 | 1 | 161 |
| Win32 | 116 | 10 | 10 | 2 | 3 | 0 | 1 | 142 |
| Cocoa | 110 | 19 | 19 | 0 | 3 | 0 | 1 | 152 |

Each manifest contains all 120 named keys. Missing mappings, intercepted inputs and unqualified
lock/repeat sources retain their explicit reasons and are not counted as passes. X11's temporary
Openbox profile removes the bare Print binding and is loaded by absolute path; PrintScreen passes
on both Linux backends. Win32 Num Lock uses virtual-key injection so both toggle phases are
observable. Win32/Cocoa remain experimental pending ten consecutive qualified runs of the current
injector, schema, workload, and image. Existing schema-1 CI links above qualify only earlier cases.

## Public VMNL runtime evidence (stage 5)

On 2026-10-12, `just input-test-vmnl x11` passed `1/1` through the public facade on Fedora 44,
Hyprland 0.56.2/Xwayland, Intel Iris Xe (ADL GT2), Mesa 26.2.3 and Vulkan loader 1.4.341.
See the [retained execution record](../../testing.md#native-input-contract).

The scenario presents a first frame before requiring focus, resolves physical A through XKB
`AC01`, and requires the independently selected scancode. It covers A press/release and input
snapshots, pointer motion/getter agreement, mouse buttons 1–6 and both scroll axes/directions.
The local X11 pointer map reports ten server buttons, so VMNL buttons 7–8 remain non-injectable.
The X11 Vulkan instance also enables Xlib when GLFW selects XCB, matching the raw handle used
by Vulkano for surface creation.

This is automatic GPU/display execution. It does not qualify every public key, text, repeat,
enter/leave, focus loss or same-batch taps through VMNL, Windows/macOS VMNL runtime, a native
Xorg/Wayland seat, other drivers/hardware, or human manual graphical regression checks.

## Keyboard named-key rows

For each row, required result K1 is: press emits `KeyPressed { key: Key::<name>, scancode,
modifiers, repeat: false }`; release emits `KeyReleased` with the same scancode and expected
modifiers. The press sets `is_down`/`is_pressed`; release clears `is_down`. A press and release in
one `Window::poll_events()` batch retain both transition flags with `is_down == false`; the next
empty batch clears both flags. The `GLFW_KEY_*` token is the independent mapping oracle to assert
for that named VMNL variant. Deterministic level: unit/API. Native level: profile K; end-to-end
level: profile V.

At baseline, `keyboard.rs` has an exhaustive 120-key conversion round-trip assertion and count,
but that round-trip does not prove each fixed mapping independently. `event.rs` translates A press,
A repeat, F25 release, and an unknown key; batch state is exercised with A. The native probe tests
A directly through GLFW, not through VMNL. Every row therefore remains `not yet qualified` for
the full #91 event/state and VMNL-path contract.

The native column below records the schema-2 per-key press/release, scancode and released-getter
results from run #38206068512. `pass` refers only to the direct GLFW case; `not injectable` and
`not yet qualified` preserve the manifest status. The deterministic baseline column remains a
historical snapshot; the new exhaustive modeled assertions are listed in branch progress above.

| VMNL key | Independent GLFW 3.4 token | Required result | Deterministic evidence at baseline | Native evidence and current status |
| --- | --- | --- | --- | --- |
| `Key::A` | `GLFW_KEY_A` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass; public VMNL X11 press/release, scancode and snapshots: local pass (stage 5) |
| `Key::B` | `GLFW_KEY_B` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::C` | `GLFW_KEY_C` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::D` | `GLFW_KEY_D` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::E` | `GLFW_KEY_E` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F` | `GLFW_KEY_F` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::G` | `GLFW_KEY_G` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::H` | `GLFW_KEY_H` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::I` | `GLFW_KEY_I` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::J` | `GLFW_KEY_J` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::K` | `GLFW_KEY_K` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::L` | `GLFW_KEY_L` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::M` | `GLFW_KEY_M` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::N` | `GLFW_KEY_N` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::O` | `GLFW_KEY_O` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::P` | `GLFW_KEY_P` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Q` | `GLFW_KEY_Q` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::R` | `GLFW_KEY_R` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::S` | `GLFW_KEY_S` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::T` | `GLFW_KEY_T` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::U` | `GLFW_KEY_U` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::V` | `GLFW_KEY_V` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::W` | `GLFW_KEY_W` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::X` | `GLFW_KEY_X` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Y` | `GLFW_KEY_Y` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Z` | `GLFW_KEY_Z` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num0` | `GLFW_KEY_0` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num1` | `GLFW_KEY_1` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num2` | `GLFW_KEY_2` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num3` | `GLFW_KEY_3` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num4` | `GLFW_KEY_4` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num5` | `GLFW_KEY_5` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num6` | `GLFW_KEY_6` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num7` | `GLFW_KEY_7` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num8` | `GLFW_KEY_8` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Num9` | `GLFW_KEY_9` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Escape` | `GLFW_KEY_ESCAPE` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Enter` | `GLFW_KEY_ENTER` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Tab` | `GLFW_KEY_TAB` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Backspace` | `GLFW_KEY_BACKSPACE` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Left` | `GLFW_KEY_LEFT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Right` | `GLFW_KEY_RIGHT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Up` | `GLFW_KEY_UP` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Down` | `GLFW_KEY_DOWN` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F1` | `GLFW_KEY_F1` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F2` | `GLFW_KEY_F2` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F3` | `GLFW_KEY_F3` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F4` | `GLFW_KEY_F4` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F5` | `GLFW_KEY_F5` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F6` | `GLFW_KEY_F6` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F7` | `GLFW_KEY_F7` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F8` | `GLFW_KEY_F8` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F9` | `GLFW_KEY_F9` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F10` | `GLFW_KEY_F10` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F11` | `GLFW_KEY_F11` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F12` | `GLFW_KEY_F12` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Space` | `GLFW_KEY_SPACE` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Apostrophe` | `GLFW_KEY_APOSTROPHE` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Comma` | `GLFW_KEY_COMMA` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Minus` | `GLFW_KEY_MINUS` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Period` | `GLFW_KEY_PERIOD` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Slash` | `GLFW_KEY_SLASH` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Semicolon` | `GLFW_KEY_SEMICOLON` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Equal` | `GLFW_KEY_EQUAL` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::LeftBracket` | `GLFW_KEY_LEFT_BRACKET` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Backslash` | `GLFW_KEY_BACKSLASH` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::RightBracket` | `GLFW_KEY_RIGHT_BRACKET` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::GraveAccent` | `GLFW_KEY_GRAVE_ACCENT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::World1` | `GLFW_KEY_WORLD_1` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: not injectable; Win32: not injectable; Cocoa: pass |
| `Key::World2` | `GLFW_KEY_WORLD_2` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: not injectable; Wayland: pass; Win32: pass; Cocoa: not injectable |
| `Key::Insert` | `GLFW_KEY_INSERT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Delete` | `GLFW_KEY_DELETE` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::PageUp` | `GLFW_KEY_PAGE_UP` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::PageDown` | `GLFW_KEY_PAGE_DOWN` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Home` | `GLFW_KEY_HOME` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::End` | `GLFW_KEY_END` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::CapsLock` | `GLFW_KEY_CAPS_LOCK` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: not yet qualified |
| `Key::ScrollLock` | `GLFW_KEY_SCROLL_LOCK` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: not injectable |
| `Key::NumLock` | `GLFW_KEY_NUM_LOCK` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::PrintScreen` | `GLFW_KEY_PRINT_SCREEN` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Pause` | `GLFW_KEY_PAUSE` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: not injectable |
| `Key::F13` | `GLFW_KEY_F13` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: not injectable |
| `Key::F14` | `GLFW_KEY_F14` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F15` | `GLFW_KEY_F15` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F16` | `GLFW_KEY_F16` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F17` | `GLFW_KEY_F17` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F18` | `GLFW_KEY_F18` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F19` | `GLFW_KEY_F19` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F20` | `GLFW_KEY_F20` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::F21` | `GLFW_KEY_F21` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: not injectable |
| `Key::F22` | `GLFW_KEY_F22` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: not injectable |
| `Key::F23` | `GLFW_KEY_F23` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: not injectable |
| `Key::F24` | `GLFW_KEY_F24` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: not injectable |
| `Key::F25` | `GLFW_KEY_F25` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: not injectable; Wayland: not injectable; Win32: not injectable; Cocoa: not injectable |
| `Key::Kp0` | `GLFW_KEY_KP_0` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp1` | `GLFW_KEY_KP_1` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp2` | `GLFW_KEY_KP_2` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp3` | `GLFW_KEY_KP_3` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp4` | `GLFW_KEY_KP_4` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp5` | `GLFW_KEY_KP_5` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp6` | `GLFW_KEY_KP_6` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp7` | `GLFW_KEY_KP_7` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp8` | `GLFW_KEY_KP_8` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::Kp9` | `GLFW_KEY_KP_9` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::KpDecimal` | `GLFW_KEY_KP_DECIMAL` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::KpDivide` | `GLFW_KEY_KP_DIVIDE` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::KpMultiply` | `GLFW_KEY_KP_MULTIPLY` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::KpSubtract` | `GLFW_KEY_KP_SUBTRACT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::KpAdd` | `GLFW_KEY_KP_ADD` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::KpEnter` | `GLFW_KEY_KP_ENTER` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::KpEqual` | `GLFW_KEY_KP_EQUAL` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::LeftShift` | `GLFW_KEY_LEFT_SHIFT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::LeftControl` | `GLFW_KEY_LEFT_CONTROL` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::LeftAlt` | `GLFW_KEY_LEFT_ALT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::LeftSuper` | `GLFW_KEY_LEFT_SUPER` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: not injectable; Cocoa: pass |
| `Key::RightShift` | `GLFW_KEY_RIGHT_SHIFT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::RightControl` | `GLFW_KEY_RIGHT_CONTROL` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::RightAlt` | `GLFW_KEY_RIGHT_ALT` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |
| `Key::RightSuper` | `GLFW_KEY_RIGHT_SUPER` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: not injectable; Cocoa: pass |
| `Key::Menu` | `GLFW_KEY_MENU` | K1 | Round-trip assertion only; per-key event/state oracle missing | X11: pass; Wayland: pass; Win32: pass; Cocoa: pass |

`Key::Unknown` is not a named-key row: unknown physical keys must preserve `Scancode` and
modifiers in `KeyPressed`/`KeyReleased` events while remaining absent from `KeyboardState`.
The checked-in reducer test covers one unknown press; native availability is best-effort and must
be marked `not injectable` with the device/layout reason when no unknown key can be produced.

## Mouse-button rows

For each row, required result M1 is: press/release emits the corresponding
`MouseButtonPressed`/`MouseButtonReleased` with modifiers and updates `MouseState`. A same-batch
tap retains both transition flags with `is_down == false`; the next empty batch clears them.
The fixed `GLFW_MOUSE_BUTTON_N` token is the independent mapping oracle. Deterministic level:
unit/API. Native level: profile M; public VMNL end-to-end level: profile V.

All eight variants have checked-in conversion, event-translation, and batch-state assertions. The
native XTEST cases test every GLFW mapping present in the active X11 server and record mappings
that exceed its button limit. The Win32 probe injects its five GLFW mappings through SendInput;
the Cocoa probe injects all eight through CGEventPost. Hosted CI's 10-button Xvfb cannot inject
server buttons 11/12, so GLFW `Button7`/`Button8` remain unqualified there. Win32 eligible buttons
passed in [CI run #38079003041](https://github.com/VMNL/vmnl/actions/runs/38079003041), and all
Cocoa buttons passed in [CI run #38077590550](https://github.com/VMNL/vmnl/actions/runs/38077590550);
the local public Vulkan/Xwayland scenario qualifies buttons 1–6 only.

| VMNL button | Independent GLFW 3.4 token | Required result | Deterministic evidence at baseline | Native evidence and current status |
| --- | --- | --- | --- | --- |
| `MouseButton::Left` | `GLFW_MOUSE_BUTTON_1` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 and nested Weston server button 1 passed press/release in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32/Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Public VMNL press/release and snapshots passed on the local Vulkan/Xwayland stack (stage 5). |
| `MouseButton::Right` | `GLFW_MOUSE_BUTTON_2` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 and nested Weston server button 3 passed press/release in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32/Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Public VMNL press/release and snapshots passed on the local Vulkan/Xwayland stack (stage 5). |
| `MouseButton::Middle` | `GLFW_MOUSE_BUTTON_3` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 and nested Weston server button 2 passed press/release in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32/Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Public VMNL press/release and snapshots passed on the local Vulkan/Xwayland stack (stage 5). |
| `MouseButton::Button4` | `GLFW_MOUSE_BUTTON_4` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 and nested Weston server button 8 passed press/release in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32/Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Public VMNL press/release and snapshots passed on the local Vulkan/Xwayland stack (stage 5). |
| `MouseButton::Button5` | `GLFW_MOUSE_BUTTON_5` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 and nested Weston server button 9 passed press/release in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32/Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Public VMNL press/release and snapshots passed on the local Vulkan/Xwayland stack (stage 5). |
| `MouseButton::Button6` | `GLFW_MOUSE_BUTTON_6` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 and nested Weston server button 10 passed press/release in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32 has no mapping; Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Public VMNL press/release and snapshots passed on the local Vulkan/Xwayland stack (stage 5). |
| `MouseButton::Button7` | `GLFW_MOUSE_BUTTON_7` | M1 | Round-trip assertion exists; event/state test only covers Left | Not injectable on hosted X11/nested Weston: server button 11 exceeds the recorded 10-button map in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32 has no mapping; Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Public VMNL runtime remains unqualified. |
| `MouseButton::Button8` | `GLFW_MOUSE_BUTTON_8` | M1 | Round-trip assertion exists; event/state test only covers Left | Not injectable on hosted X11/nested Weston: server button 12 exceeds the recorded 10-button map in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32 has no mapping; Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Public VMNL runtime remains unqualified. |

The issue's GLFW 3.4 backend constraints apply per row: Win32 maps buttons 1–5; X11 reserves server
buttons 4–7 for scroll and maps buttons 8–12 to GLFW buttons 4–8; Cocoa maps native button numbers
0–7 to GLFW buttons 1–8. The hosted Xvfb pointer map has only 10 server buttons, so XTEST cannot
qualify the last two mappings there; nested Weston inherits the same parent-X11 limit. Expanded
Windows and Cocoa button probes now have a first passing CI result, but neither platform meets the
ten-run threshold. Never count an unavailable mapping as a pass.

## Other keyboard and mouse input families

| Family | Expected VMNL event/state | Independent expected value | Deterministic evidence at baseline | Native profile and current status |
| --- | --- | --- | --- | --- |
| Key modifiers | Key press, repeat, and release preserve Shift, Control, Alt, Super, Caps Lock, Num Lock. | Explicit GLFW modifier-bit combination and fixed VMNL bitmask. | Event translation covers all six together; conversion test checks positive and absent bits. | Schema 2: all eligible left-modifier subsets and right-side variants pass on all four backends; Alt/Super target flags are checked on mouse callbacks. Caps/Num lock reporting passes on X11, nested Wayland and Win32; Cocoa locks remain unqualified. |
| Mouse-button modifiers | Button events preserve callback modifier flags. | Fixed GLFW modifier-bit combination. | Translation uses Shift+Caps Lock on press and Control on release; not an exhaustive combination set. | Schema 2: eligible modifier chords pass on all four backends. Linux Alt/Super chords use side Button4; grabbed left clicks remain explicitly non-injectable. Caps/Num flags pass on Linux/Win32. |
| Text input | `Text(char)` carries the Unicode scalar independently of physical key events. | Fixed representative scalar literals, including non-ASCII and composed input. | Translation covers `é`; no public VMNL native path. | Schema 2: a passes on all four backends; é and 🦀 also pass on Win32/Cocoa. Those scalars are absent from the active Linux map and are explicitly non-injectable. Composed input and public VMNL text remain unqualified. |
| Legacy modified text | `TextWithModifiers { character, modifiers }` preserves both fields. | Fixed character and all six expected modifier bits. | Translation covers `É` with all six bits. | Schema 2: unmodified CharModifiers then Char callbacks pass for each eligible text scalar on all four backends. Modified text with all six flags remains unqualified. |
| Repeat | Repeat emits `KeyPressed { repeat: true }`, leaves the key down, and does not create another press transition. | Explicit `Action::Repeat` test value and bounded native hold. | Translation and reducer have deterministic representative tests. | Schema 2: a bounded 1.5-second A hold produces ordered repeat callbacks on X11/nested Wayland. Win32/Cocoa posters have no qualified hardware repeat source; public VMNL repeat remains unqualified. |
| Pointer movement | `MouseMoved { x, y }` uses content-area coordinates. | XTEST moves the hovered pointer by `(6, 4)` screen coordinates; X11 checks the exact content-area delta, nested Weston checks positive movement and getter/event agreement. | Translation preserves fractional and negative coordinates. | X11 and nested Weston passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980); Win32 and Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). Win32 ignores unchanged callbacks; public VMNL motion/getter agreement passed locally (stage 5). |
| Pointer position | `get_cursor_position` reports content-area position; `set_cursor_position` requests the specified target subject to backend precision. | Compare the post-motion getter with the `MouseMoved` event. | Public signatures and Null probe cover setting; native XTEST movement now checks reported position. | Same native motion evidence as pointer movement; public VMNL position/getter agreement passed locally (stage 5). |
| Enter/leave | `MouseEntered` then `MouseLeft` when crossing the content-area boundary in each direction. | Expected ordered pair from a controlled pointer path. | Both event translations are covered. | X11 and nested Weston passed the ordered leave/enter pair in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980); Win32 and Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). The Weston path crosses to its default top panel inside the compositor. Nested Weston does not qualify a standalone Wayland seat. Public VMNL runtime remains unqualified. |
| Vertical scroll | `MouseScrolled { dx, dy }` preserves vertical direction. | X11 buttons 4/5; GLFW 3.4 maps them to `(0, +1)` / `(0, -1)`. | Translation covers fractional values; native XTEST and public VMNL up/down assertions are checked in. | M: X11 and nested Weston passed all four directions in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980); Win32/Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). V: both directions passed locally through VMNL on Vulkan/Xwayland (stage 5). |
| Horizontal scroll | `MouseScrolled { dx, dy }` preserves horizontal direction. | X11 buttons 6/7; GLFW 3.4 maps them to `(+1, 0)` / `(-1, 0)`. | Translation covers fractional values; native XTEST and public VMNL direction assertions are checked in. | M: X11 and nested Weston passed all four directions in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980); Win32/Cocoa passed in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980). V: both directions passed locally through VMNL on Vulkan/Xwayland (stage 5). |
| Event delivery and snapshots | Disabling one event source suppresses its event while key/button snapshots still update; sources remain independent. | Fixed enabled/disabled source combinations and expected event lists/state. | Internal delivery reducer covers sources and independent keyboard/mouse tracking; public tests cover facade accessors, not native event flow. | K/M; no end-to-end VMNL probe. |
| Focus loss and batch boundaries | Focus loss releases held keys/buttons; transitions survive one batch and clear on the next empty batch. | Fixed event order and snapshot truth table. | Reducer tests cover representative key and mouse-button states. | Schema 2: all four backends pass held A/left-button getters, focus loss, ordered synthetic releases, and released getters before physical cleanup. Public VMNL focus-loss/batch-boundary runtime remains unqualified. |

## Remaining gaps for issue #91

- Reach ten consecutive passing Win32/Cocoa runs for the schema-2 workload. Cocoa lock restoration and
  hardware-repeat sources on Win32/Cocoa remain unqualified. Linux non-ASCII text requires an
  eligible controlled layout. Public VMNL focus-loss/batch-boundary runtime remains outstanding.
  `Button7`/`Button8` require a different Linux pointer mapping; the hosted 10-button Xvfb cannot
  inject them.
- Extend public VMNL runtime qualification beyond the representative local Xwayland scenario when
  claiming those additional contracts or environments; keep direct GLFW, GPU execution and human
  manual graphics evidence separate.

Existing references: [keyboard capability matrix](keyboard_capability_matrix.md),
[GLFW portability inventory](glfw_platform_inventory.md), [platform probe contract](../../../docs/testing.md),
[public input API tests](../../../tests/api/tests/input.rs),
[event translation and reducer](../../../crates/vmnl_graphics/src/window/event.rs),
[platform probe assertions](../../../tests/platform/tests/backend_contract.rs), and
[GPU window runtime tests](../../../tests/gpu/tests/window_runtime.rs).
