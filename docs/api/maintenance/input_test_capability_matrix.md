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
| K | X11 / XTEST | Visible mapped window, confirmed focus, active keyboard layout; unavailable keys and intercepted shortcuts need a recorded reason. | `keyboard-native-input` injects only A directly into GLFW. |
| K | Nested Weston / parent X11 XTEST | Same focus/readiness checks; this exercises XTEST → Xvfb → Weston X11 backend → Wayland client, not a standalone Wayland seat. | `keyboard-native-input` injects only A directly into GLFW. |
| K | Win32 / `SendInput` | Active desktop at matching integrity; keyboard layout and system shortcuts constrain eligible keys. | A-only probe is configured as visible, non-blocking evidence until qualification. |
| K | Cocoa / `CGEventPost` | Active desktop; macOS may require Accessibility permission. Layout and system shortcuts constrain eligible keys. | A-only probe is configured as visible, non-blocking evidence until qualification. |
| M | X11 / XTEST | Mapped, focused, hovered window and recorded X server button map. Server buttons 4–7 are scroll; later server buttons map to additional GLFW buttons. | Left-button passed in [CI run #154](https://github.com/VMNL/vmnl/actions/runs/38058493301); vertical up/down passed in [CI run #38068589103](https://github.com/VMNL/vmnl/actions/runs/38068589103); both horizontal directions passed in [CI run #38071107344](https://github.com/VMNL/vmnl/actions/runs/38071107344); all-button assertions added, awaiting CI. |
| M | Nested Weston / parent X11 XTEST | Records nested Weston evidence only; button map and focus must be captured. Does not qualify a native Wayland seat. | Left-button passed in [CI run #154](https://github.com/VMNL/vmnl/actions/runs/38058493301); vertical up/down passed through parent X11 in [CI run #38068589103](https://github.com/VMNL/vmnl/actions/runs/38068589103); both horizontal directions passed in [CI run #38071107344](https://github.com/VMNL/vmnl/actions/runs/38071107344); all-button assertions added, awaiting CI. |
| M | Win32 / `SendInput` | Active desktop at matching integrity. GLFW Win32 maps left/right/middle and XBUTTON1/2, so only GLFW buttons 1–5 are eligible through this mapping. | First left-button pass in [CI run #154](https://github.com/VMNL/vmnl/actions/runs/38058493301); still experimental pending ten consecutive successes. |
| M | Cocoa / `CGEventPost` | Requires an active desktop and any required Accessibility permission; remaining button mapping must be established by probes. | First left-button pass in [CI run #154](https://github.com/VMNL/vmnl/actions/runs/38058493301); still experimental pending ten consecutive successes. |
| V | Public VMNL path / X11 XTEST | Requires a qualified Vulkan loader, GPU/driver, X11 EWMH display, XTEST, mapped and focused VMNL window. | `just input-test-vmnl x11` checks native A, all eight mouse-button transitions, and both scroll axes through `Window::poll_events()` → `Event`/`Input`; runtime result not yet qualified. |

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
- The native probe now checks one representative mouse case: focused and hovered window, ordered
  `Button1` press/release with modifiers, released final state, injector identity, and a failure
  reason. A passing probe still proves GLFW backend delivery only, not the public VMNL path.
- XTEST mouse-scroll injects vertical up/down (server buttons 4/5) and horizontal positive/negative
  (buttons 6/7), expecting ordered offsets `(0, +1)`, `(0, -1)`, `(+1, 0)`, and `(-1, 0)`. The
  CI pass [#38068589103](https://github.com/VMNL/vmnl/actions/runs/38068589103) qualifies vertical
  scroll and [#38071107344](https://github.com/VMNL/vmnl/actions/runs/38071107344) qualifies both
  horizontal directions on X11 and nested Weston. Downloaded Linux artifacts contain environment
  logs but no probe JSONL; this branch changes the artifact destination to an absolute workspace
  path, pending CI verification. The public VMNL Vulkan/display runtime remains unqualified.
- The new native mouse-button case sends X11 server buttons `1, 2, 3, 8–12`, which GLFW maps to
  buttons 1, 3, 2, 4–8 in event order. Its exhaustive ordered assertions and final released-state
  checks are checked in and await CI on X11 and nested Weston.
- CI run #154 passed the representative left-button probe on X11, nested Weston, Win32, and Cocoa.
  Linux results are blocking; Windows and macOS are first experimental passes and remain
  non-blocking until the documented ten-run qualification rule is met. Its Linux artifact also
  lacks probe JSONL because of the relative artifact destination; the correction is awaiting CI.
- `just input-test <backend>` checks the selected host, display, compositor/window-manager, and
  available input-permission prerequisites before running one native contract. CI continues to
  invoke the same selected contracts directly through Cargo: Linux X11 and nested Weston block,
  while Win32 and Cocoa remain experimental.
- `just input-test-vmnl x11` adds an opt-in Vulkan-backed public-facade path for A, all eight
  mouse-button press/release pairs, and vertical/horizontal scroll directions. It is excluded from the general
  `just test-gpu` suite because it injects input into the active desktop; no Vulkan/display runtime
  result is recorded yet.

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

| VMNL key | Independent GLFW 3.4 token | Required result | Deterministic evidence at baseline | Native evidence at baseline |
| --- | --- | --- | --- | --- |
| `Key::A` | `GLFW_KEY_A` | K1 | Round-trip assertion only; per-key event/state oracle missing | A-only direct GLFW probe; VMNL path not qualified |
| `Key::B` | `GLFW_KEY_B` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::C` | `GLFW_KEY_C` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::D` | `GLFW_KEY_D` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::E` | `GLFW_KEY_E` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F` | `GLFW_KEY_F` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::G` | `GLFW_KEY_G` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::H` | `GLFW_KEY_H` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::I` | `GLFW_KEY_I` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::J` | `GLFW_KEY_J` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::K` | `GLFW_KEY_K` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::L` | `GLFW_KEY_L` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::M` | `GLFW_KEY_M` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::N` | `GLFW_KEY_N` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::O` | `GLFW_KEY_O` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::P` | `GLFW_KEY_P` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Q` | `GLFW_KEY_Q` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::R` | `GLFW_KEY_R` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::S` | `GLFW_KEY_S` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::T` | `GLFW_KEY_T` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::U` | `GLFW_KEY_U` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::V` | `GLFW_KEY_V` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::W` | `GLFW_KEY_W` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::X` | `GLFW_KEY_X` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Y` | `GLFW_KEY_Y` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Z` | `GLFW_KEY_Z` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num0` | `GLFW_KEY_0` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num1` | `GLFW_KEY_1` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num2` | `GLFW_KEY_2` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num3` | `GLFW_KEY_3` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num4` | `GLFW_KEY_4` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num5` | `GLFW_KEY_5` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num6` | `GLFW_KEY_6` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num7` | `GLFW_KEY_7` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num8` | `GLFW_KEY_8` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Num9` | `GLFW_KEY_9` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Escape` | `GLFW_KEY_ESCAPE` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Enter` | `GLFW_KEY_ENTER` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Tab` | `GLFW_KEY_TAB` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Backspace` | `GLFW_KEY_BACKSPACE` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Left` | `GLFW_KEY_LEFT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Right` | `GLFW_KEY_RIGHT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Up` | `GLFW_KEY_UP` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Down` | `GLFW_KEY_DOWN` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F1` | `GLFW_KEY_F1` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F2` | `GLFW_KEY_F2` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F3` | `GLFW_KEY_F3` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F4` | `GLFW_KEY_F4` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F5` | `GLFW_KEY_F5` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F6` | `GLFW_KEY_F6` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F7` | `GLFW_KEY_F7` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F8` | `GLFW_KEY_F8` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F9` | `GLFW_KEY_F9` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F10` | `GLFW_KEY_F10` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F11` | `GLFW_KEY_F11` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F12` | `GLFW_KEY_F12` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Space` | `GLFW_KEY_SPACE` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Apostrophe` | `GLFW_KEY_APOSTROPHE` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Comma` | `GLFW_KEY_COMMA` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Minus` | `GLFW_KEY_MINUS` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Period` | `GLFW_KEY_PERIOD` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Slash` | `GLFW_KEY_SLASH` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Semicolon` | `GLFW_KEY_SEMICOLON` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Equal` | `GLFW_KEY_EQUAL` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::LeftBracket` | `GLFW_KEY_LEFT_BRACKET` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Backslash` | `GLFW_KEY_BACKSLASH` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::RightBracket` | `GLFW_KEY_RIGHT_BRACKET` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::GraveAccent` | `GLFW_KEY_GRAVE_ACCENT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::World1` | `GLFW_KEY_WORLD_1` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::World2` | `GLFW_KEY_WORLD_2` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Insert` | `GLFW_KEY_INSERT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Delete` | `GLFW_KEY_DELETE` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::PageUp` | `GLFW_KEY_PAGE_UP` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::PageDown` | `GLFW_KEY_PAGE_DOWN` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Home` | `GLFW_KEY_HOME` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::End` | `GLFW_KEY_END` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::CapsLock` | `GLFW_KEY_CAPS_LOCK` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::ScrollLock` | `GLFW_KEY_SCROLL_LOCK` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::NumLock` | `GLFW_KEY_NUM_LOCK` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::PrintScreen` | `GLFW_KEY_PRINT_SCREEN` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Pause` | `GLFW_KEY_PAUSE` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F13` | `GLFW_KEY_F13` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F14` | `GLFW_KEY_F14` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F15` | `GLFW_KEY_F15` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F16` | `GLFW_KEY_F16` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F17` | `GLFW_KEY_F17` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F18` | `GLFW_KEY_F18` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F19` | `GLFW_KEY_F19` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F20` | `GLFW_KEY_F20` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F21` | `GLFW_KEY_F21` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F22` | `GLFW_KEY_F22` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F23` | `GLFW_KEY_F23` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F24` | `GLFW_KEY_F24` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::F25` | `GLFW_KEY_F25` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp0` | `GLFW_KEY_KP_0` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp1` | `GLFW_KEY_KP_1` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp2` | `GLFW_KEY_KP_2` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp3` | `GLFW_KEY_KP_3` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp4` | `GLFW_KEY_KP_4` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp5` | `GLFW_KEY_KP_5` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp6` | `GLFW_KEY_KP_6` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp7` | `GLFW_KEY_KP_7` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp8` | `GLFW_KEY_KP_8` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Kp9` | `GLFW_KEY_KP_9` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::KpDecimal` | `GLFW_KEY_KP_DECIMAL` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::KpDivide` | `GLFW_KEY_KP_DIVIDE` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::KpMultiply` | `GLFW_KEY_KP_MULTIPLY` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::KpSubtract` | `GLFW_KEY_KP_SUBTRACT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::KpAdd` | `GLFW_KEY_KP_ADD` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::KpEnter` | `GLFW_KEY_KP_ENTER` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::KpEqual` | `GLFW_KEY_KP_EQUAL` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::LeftShift` | `GLFW_KEY_LEFT_SHIFT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::LeftControl` | `GLFW_KEY_LEFT_CONTROL` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::LeftAlt` | `GLFW_KEY_LEFT_ALT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::LeftSuper` | `GLFW_KEY_LEFT_SUPER` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::RightShift` | `GLFW_KEY_RIGHT_SHIFT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::RightControl` | `GLFW_KEY_RIGHT_CONTROL` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::RightAlt` | `GLFW_KEY_RIGHT_ALT` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::RightSuper` | `GLFW_KEY_RIGHT_SUPER` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |
| `Key::Menu` | `GLFW_KEY_MENU` | K1 | Round-trip assertion only; per-key event/state oracle missing | No per-key native case; not yet qualified |

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

At baseline, all eight variants have a checked-in conversion round-trip assertion. Event
translation and batch state used only `MouseButton::Left`; this branch now adds native XTEST cases
for all eight GLFW buttons and public VMNL press/release assertions for every variant. The public
Vulkan/display runtime remains unqualified.

| VMNL button | Independent GLFW 3.4 token | Required result | Deterministic evidence at baseline | Native evidence at baseline |
| --- | --- | --- | --- | --- |
| `MouseButton::Left` | `GLFW_MOUSE_BUTTON_1` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 and nested Weston representative probe passed in [CI run #154](https://github.com/VMNL/vmnl/actions/runs/38058493301); public VMNL runtime not qualified. |
| `MouseButton::Right` | `GLFW_MOUSE_BUTTON_2` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 button 3 → GLFW button 2 case added; awaiting CI. Win32 mapping is eligible but currently only left is tested. |
| `MouseButton::Middle` | `GLFW_MOUSE_BUTTON_3` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 button 2 → GLFW button 3 case added; awaiting CI. Win32 mapping is eligible but currently only left is tested. |
| `MouseButton::Button4` | `GLFW_MOUSE_BUTTON_4` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 button 8 case added; awaiting CI. Win32 XBUTTON1 is eligible but currently not tested; Cocoa mapping remains unqualified. |
| `MouseButton::Button5` | `GLFW_MOUSE_BUTTON_5` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 button 9 case added; awaiting CI. Win32 XBUTTON2 is eligible but currently not tested; Cocoa mapping remains unqualified. |
| `MouseButton::Button6` | `GLFW_MOUSE_BUTTON_6` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 button 10 case added; awaiting CI. Win32 does not expose this button through GLFW's XBUTTON1/2 mapping; Cocoa mapping remains unqualified. |
| `MouseButton::Button7` | `GLFW_MOUSE_BUTTON_7` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 button 11 case added; awaiting CI. Win32 does not expose this button through GLFW's XBUTTON1/2 mapping; Cocoa mapping remains unqualified. |
| `MouseButton::Button8` | `GLFW_MOUSE_BUTTON_8` | M1 | Round-trip assertion exists; event/state test only covers Left | X11 button 12 case added; awaiting CI. Win32 does not expose this button through GLFW's XBUTTON1/2 mapping; Cocoa mapping remains unqualified. |

The issue's GLFW 3.4 backend constraints apply per row: Win32 can synthesize buttons 1–5;
X11 reserves server buttons 4–7 for scroll and maps buttons 8–12 to GLFW buttons 4–8; nested
Weston inherits the tested parent-X11 route. Cocoa's remaining button eligibility is unqualified.
Never count an unavailable mapping as a pass.

## Other keyboard and mouse input families

| Family | Expected VMNL event/state | Independent expected value | Deterministic evidence at baseline | Native profile and current status |
| --- | --- | --- | --- | --- |
| Key modifiers | Key press, repeat, and release preserve Shift, Control, Alt, Super, Caps Lock, Num Lock. | Explicit GLFW modifier-bit combination and fixed VMNL bitmask. | Event translation covers all six together; conversion test checks positive and absent bits. | K; only unmodified A is injected. Modifier combinations not yet qualified. |
| Mouse-button modifiers | Button events preserve callback modifier flags. | Fixed GLFW modifier-bit combination. | Translation uses Shift+Caps Lock on press and Control on release; not an exhaustive combination set. | M; not yet qualified. |
| Text input | `Text(char)` carries the Unicode scalar independently of physical key events. | Fixed representative scalar literals, including non-ASCII and composed input. | Translation covers `é`; no public VMNL native path. | K; controlled layout/input method required; injector support not yet qualified. Do not enumerate all Unicode. |
| Legacy modified text | `TextWithModifiers { character, modifiers }` preserves both fields. | Fixed character and all six expected modifier bits. | Translation covers `É` with all six bits. | K; exercise only where the backend/input method emits this deprecated GLFW callback; otherwise record why unsupported. |
| Repeat | Repeat emits `KeyPressed { repeat: true }`, leaves the key down, and does not create another press transition. | Explicit `Action::Repeat` test value and bounded native hold. | Translation and reducer have deterministic representative tests. | K; hold a repeatable key for a bounded interval; not yet qualified. |
| Pointer movement | `MouseMoved { x, y }` uses content-area coordinates. | XTEST moves the hovered pointer by `(6, 4)` screen coordinates; X11 checks the exact content-area delta, nested Weston checks positive movement and getter/event agreement. | Translation preserves fractional and negative coordinates. | Native X11/nested Weston probe and public VMNL X11 event/state assertion added; awaiting CI. |
| Pointer position | `get_cursor_position` reports content-area position; `set_cursor_position` requests the specified target subject to backend precision. | Compare the post-motion getter with the `MouseMoved` event. | Public signatures and Null probe cover setting; native XTEST movement now checks reported position. | Same probe and qualification status as pointer movement. |
| Enter/leave | `MouseEntered` then `MouseLeft` when crossing the content-area boundary in each direction. | Expected ordered pair from a controlled pointer path. | Both event translations are covered. | M; no native hover probe. |
| Vertical scroll | `MouseScrolled { dx, dy }` preserves vertical direction. | X11 buttons 4/5; GLFW 3.4 maps them to `(0, +1)` / `(0, -1)`. | Translation covers fractional values; native XTEST and public VMNL up/down assertions are checked in. | M: X11 and nested Weston passed in [CI run #38068589103](https://github.com/VMNL/vmnl/actions/runs/38068589103); V: public Vulkan/X11 runtime not yet qualified. |
| Horizontal scroll | `MouseScrolled { dx, dy }` preserves horizontal direction. | X11 buttons 6/7; GLFW 3.4 maps them to `(+1, 0)` / `(-1, 0)`. | Translation covers fractional values; native XTEST and public VMNL direction assertions are checked in. | M: X11 and nested Weston passed in [CI run #38071107344](https://github.com/VMNL/vmnl/actions/runs/38071107344); V: public Vulkan/X11 runtime not yet qualified. |
| Event delivery and snapshots | Disabling one event source suppresses its event while key/button snapshots still update; sources remain independent. | Fixed enabled/disabled source combinations and expected event lists/state. | Internal delivery reducer covers sources and independent keyboard/mouse tracking; public tests cover facade accessors, not native event flow. | K/M; no end-to-end VMNL probe. |
| Focus loss and batch boundaries | Focus loss releases held keys/buttons; transitions survive one batch and clear on the next empty batch. | Fixed event order and snapshot truth table. | Reducer tests cover representative key and mouse-button states. | K/M; current native probes test focus before injection, not VMNL focus-loss state. |

## Remaining gaps for issue #91

- Expand the A-key and eligible mouse-button probes across remaining backend mappings; the new
  Linux all-button case is awaiting CI. Add the remaining native keyboard and mouse families,
  including enter/leave and modifier/text/repeat cases, with explicit non-injectable reasons per
  backend row and retained diagnostics.
- Execute the new public VMNL scenario in at least one qualified Vulkan/X11 environment and retain
  its result; GPU-test compilation and direct GLFW probes do not qualify it.

Existing references: [keyboard capability matrix](keyboard_capability_matrix.md),
[GLFW portability inventory](glfw_platform_inventory.md), [platform probe contract](../../../docs/testing.md),
[public input API tests](../../../tests/api/tests/input.rs),
[event translation and reducer](../../../crates/vmnl_graphics/src/window/event.rs),
[platform probe assertions](../../../tests/platform/tests/backend_contract.rs), and
[GPU window runtime tests](../../../tests/gpu/tests/window_runtime.rs).
