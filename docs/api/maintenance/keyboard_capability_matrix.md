# GLFW keyboard capability matrix

This matrix tracks issue [#78](https://github.com/VMNL/vmnl/issues/78) against the bundled
GLFW 3.4 keyboard surface. It complements the generated
[GLFW platform inventory](glfw_platform_inventory.md): that inventory owns backend-specific
behavior, while this page maps each keyboard capability to its VMNL contract and required
evidence.

The initial baseline was `main` after PR #81; `Current status` evolves with the implementation at
the same revision as this page. Deterministic evidence and native evidence are separate
requirements; unavailable physical keys or layouts must be reported as untested rather than
passed.

| Capability | GLFW surface | Current status | Target VMNL contract | Deterministic evidence | Native evidence |
|---|---|---|---|---|---|
| Named keys | 120 named `GLFW_KEY_*` tokens | Complete: `Key` exposes all 120 named keys and facade representatives cover every key family. | `Key` represents every named GLFW 3.4 key without exposing GLFW types. | Exhaustive bidirectional conversion test and exact tracked-key count. | Exercise every key available on the recorded keyboard; list unavailable keys. |
| Unknown physical keys | `GLFW_KEY_UNKNOWN` plus callback scancode | Complete: unknown events are emitted while snapshot state remains unchanged. | Emit `Key::Unknown` events with their `Scancode`; `KeyboardState` tracks named keys only and always reports `false` for `Key::Unknown`. | Unknown-key reducer test retains the event and proves the snapshot remains unused. | Best effort only; justify when the keyboard exposes no unknown key. |
| Scancode value | Key callback scancode | Complete: public `Scancode` preserves the callback value. | Public `Scancode` newtype with `from_raw` and `as_raw`; values are platform-specific and must not be persisted as portable identifiers. | API and translation tests preserve fixed positive and negative raw values without exposing a backend type. | Compare the reported scancode for a real key within one recorded environment. |
| Key event modifiers | Key callback modifier bits | Complete: press, repeat, and release preserve `Modifiers`. | Press and release events preserve `Modifiers`; Caps Lock and Num Lock appear only when lock-key reporting is enabled. | Translation tests convert every GLFW modifier bit for press, repeat, and release events. | Exercise Shift, Control, Alt, Super, Caps Lock, and Num Lock where available. |
| Press and release events | `GLFW_PRESS`, `GLFW_RELEASE` | Complete: named and unknown events retain key metadata. | `KeyPressed { key, scancode, modifiers, repeat }` and `KeyReleased { key, scancode, modifiers }`. | Translation tests cover named and unknown keys with metadata. | Press and release representative alphanumeric, punctuation, navigation, keypad, function, and modifier keys. |
| Repeat | `GLFW_REPEAT` | Complete: repeat preserves metadata and does not create a press transition. | Repeat produces `KeyPressed { repeat: true, .. }`, keeps the key down, and does not create a new press transition. | Existing transition test plus metadata preservation on repeat. | Hold a repeatable key and observe repeat events without repeated `is_pressed`. |
| Snapshot batches | Key callback plus `Window::poll_events` | Complete automatically: keyboard press and release transitions coexist for one batch and clear on the next batch. | One poll is one batch; press and release transitions are independent, non-consuming, and clear on the next batch. | Shared reducer tests include a keyboard press/release batch followed by an empty batch. | Regression check only. |
| Event timestamps | GLFW event channel timestamp | Complete after PR #81. | Every keyboard and text event uses the existing `Event` timestamp contract. | Existing timestamp translation test covers the shared envelope. | No keyboard-specific native evidence required. |
| Unicode text | Character callback | Complete automatically: `Text(char)` preserves Unicode input and delivery is inspectable. | Preserve `Text(char)` and expose whether character delivery is enabled. Physical keys remain distinct from text input. | Translation covers non-ASCII input; delivery tests cover default, enable, disable, and source isolation. | Enter accented text and a composed character using the recorded layout/input method. |
| Unicode text with modifiers | Deprecated character-with-modifiers callback | Complete automatically: `TextWithModifiers` preserves the character and every modifier bit as a documented legacy source. | Add `TextWithModifiers { character, modifiers }`; keep its polling API as documented legacy GLFW 3.4 behavior and direct clients to text plus key events for new code. | Translation preserves the character and every modifier bit; delivery tests cover enabled and disabled states. | Exercise a modified character if supported; otherwise record the backend/layout limitation. |
| Layout-dependent key name | `glfwGetKeyName` | Complete automatically with a Wayland readiness guard: named-key and scancode queries return owned names without exposing GLFW types. | `Context::get_key_name(Key) -> Option<String>` and `Context::get_scancode_name(Scancode) -> Option<String>` allocate an owned layout-dependent name. Before the first processed Wayland keyboard event, both return `None` rather than entering GLFW with an uninitialized XKB state. | Null backend proves named-key and scancode routing plus the non-printable `None` sentinel; the native contract keeps Wayland metadata queries behind interactive readiness. | Record results for representative letter, punctuation, and keypad keys on the active layout after the first key event. |
| Key scancode query | `glfwGetKeyScancode` | Complete automatically: the active mapping is exposed through `Scancode` and unknown keys return `None`. | `Context::get_key_scancode(Key) -> Option<Scancode>` returns the active platform mapping and preserves the unknown sentinel as `None`. | Exhaustive key conversion plus a Null-backend fixed key/scancode/name round trip. | Compare a queried scancode with the callback scancode for the same real key. |
| Sticky keys | `GLFW_STICKY_KEYS` input mode | Complete automatically: configuration is inspectable while VMNL snapshots remain callback-derived and non-consuming. | `Window::is_sticky_keys_enabled` and `Window::set_sticky_keys` expose the per-window mode. VMNL snapshots remain callback-derived and never consume the native sticky latch. | Null backend proves default, enable, disable, and callback delivery; reducer tests prove press/release snapshot semantics without `glfwGetKey`. | Validate the native latch with a focused platform probe and record the backend. |
| Lock-key modifiers | `GLFW_LOCK_KEY_MODS` input mode | Complete automatically: configuration is exposed and keyboard events preserve both lock bits. | Reuse the existing window getter/setter and preserve lock bits in keyboard events. | Existing mode tests plus keyboard-event conversion with Caps Lock and Num Lock. | Toggle both locks and observe event flags where the keyboard/platform supports them. |
| Key event delivery | Key callback registration plus VMNL delivery filter | Complete automatically: delivery is configurable and inspectable while state tracking remains independent. | `Window::is_key_polling_enabled` reports public event delivery; state tracking remains active when delivery is disabled. | Existing disabled-delivery state test plus getter/default tests. | Disable delivery, press a key, and verify that events stop while snapshots still update. |
| Character event delivery | Character and character-with-modifiers callbacks | Complete automatically: both sources are independently configurable and inspectable. | Add `is_char_polling_enabled` and legacy `is_char_mods_polling_enabled`; each reports its per-window delivery configuration. | Default, enable, disable, and source-isolation tests. | Confirm each enabled source produces only its documented event kind. |
| Direct key-state read | `glfwGetKey` | Deliberately not exposed; VMNL snapshots are callback-derived. | `KeyboardState` remains the non-consuming public state API. Document that sticky native reads are not substituted for final VMNL snapshot state. | Existing snapshot tests and an explicit GLFW inventory justification. | No separate native qualification beyond snapshot and sticky-mode scenarios. |
| Focus, waiting, and window isolation | Focus callback and GLFW event processing | Complete automatically: focus release, both wait variants, and per-window keyboard state are covered. | Focus loss releases held state, `wait_events*` does not start a batch, and each window owns independent state. | Reducer tests cover focus release and keyboard isolation; Null and native contracts include keyboard wait-then-poll probes. | Alt+Tab regression and recovery on the recorded backend. |

## Approved public contracts

```text
pub struct Scancode(i32);

EventKind::KeyPressed {
    key: Key,
    scancode: Scancode,
    modifiers: Modifiers,
    repeat: bool,
}

EventKind::KeyReleased {
    key: Key,
    scancode: Scancode,
    modifiers: Modifiers,
}

EventKind::TextWithModifiers {
    character: char,
    modifiers: Modifiers,
}
```

Key names and scancodes are queried from `Context`, because their GLFW operations require an
initialized platform context. Sticky modes and event-delivery configuration remain per-window.
The deprecated GLFW character-with-modifiers source remains available for GLFW 3.4 completeness,
but its Rustdoc must identify it as legacy and recommend `Text` plus key events for new code.

## Implementation stages

1. Complete `Key`, conversions, and exhaustive conversion tests.
2. Complete: add `Scancode` and preserve key-event metadata, including unknown keys.
3. Complete: translate legacy character-with-modifiers events and make keyboard delivery settings inspectable.
4. Complete: add key-name/scancode queries and sticky-key mode with portability inventory entries and probes.
5. Automatic portion complete: finish Rustdoc and API-book references, instrument the visual
   example, close the keyboard batch/wait/isolation gaps, and provide the native qualification
   probe. Operator-reported native results remain required.

## Native qualification protocol

Record the operating system, desktop, session type, and backend with the observations. Automatic
Null or native platform probes do not replace the following operator checks.

Run the public VMNL workflow:

```bash
just run window_events_input
```

1. Confirm the startup queries for `A`, `Semicolon`, `Kp0`, and `Escape`; record unavailable
   physical keys instead of treating them as passed. `Escape` must have no printable name.
2. Press representative alphanumeric, punctuation, navigation, keypad, function, and modifier
   keys. For named keys, verify `scancode_matches=true` between the event and queried scancode.
3. Hold a repeatable key and verify repeat events without repeated `is_pressed` transitions.
4. Enter accented and composed text, then exercise Shift, Control, Alt, Super, Caps Lock, and Num
   Lock where available. Record unsupported legacy modified-text delivery explicitly.
5. Use right click to disable and restore key-event delivery; verify keyboard snapshots continue to
   update. Use `F9` and `F10` to check independent text and legacy modified-text delivery.
6. Toggle sticky-key configuration with `K`, then Alt+Tab away and back; verify recovery and the
   absence of stuck keyboard state.

Qualify the underlying GLFW sticky latch separately because VMNL deliberately does not expose the
consuming `glfwGetKey` read. Replace `wayland` with `x11` when testing that backend, focus the probe
window, then press and release `A` once:

```bash
cargo run -p vmnl-platform-tests --bin platform_probe -- wayland sticky-keys-manual
```

The probe qualifies the latch only when its JSON record reports `qualified: true`, a first read of
`Press`, and a second read of `Release`.
