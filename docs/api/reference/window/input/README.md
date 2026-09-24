# Input

| Item | Role |
|---|---|
| [`Input`](input.md) | Window-owned keyboard/mouse/joystick snapshot |
| [`Stick`](joysticks.md) | Left or right stick selector, without measured values |
| [`JoystickId`](joysticks.md) | Select one of 16 controller slots |
| [`RawJoystickState`](joysticks.md) | Device-dependent raw axes, buttons, and hats |
| [`HatState`](joysticks.md) | Raw hat cardinal and diagonal directions |
| [`JoystickState`](joysticks.md) | Raw/mapped snapshots, metadata, and typed application data |
| [`GamepadState`](joysticks.md) | Complete mapped snapshot: 15 buttons and 6 axes |
| [`GamepadButton` and `GamepadAxis`](joysticks.md) | Named mapped control selectors |
| [`JoystickInfo`](joysticks.md) | Device name, GUID, mapping name and availability |
| [`JoystickOptions`](../../context.md) | Initialization policy for synthesized hat buttons |
| [`StickSettings`](joysticks.md) | Per-stick dead zone and angle convention |
| [`StickState`](joysticks.md) | Original axes, derived angle, magnitude, and click state |
| [`Key`](key.md) | Supported key identifier |
| [`KeyboardState`](keyboard_state.md) | Current and transition key queries |
| [`MouseButton`](mouse_button.md) | Supported mouse-button identifier |
| [`MouseState`](mouse_state.md) | Current and transition button queries |

Input delivery depends on backend focus, compositor/window-manager policy, and enabled callbacks.
See the generated [window platform compatibility matrix](../platform_compatibility.md) and the
[GLFW inventory](../../../maintenance/glfw_platform_inventory.md) for callback and cursor limits.
