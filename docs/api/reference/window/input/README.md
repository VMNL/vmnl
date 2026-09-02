# Input

| Item | Role |
|---|---|
| [`Input`](input.md) | Window-owned keyboard/mouse snapshot |
| [`Key`](key.md) | Supported key identifier |
| [`KeyboardState`](keyboard_state.md) | Current and transition key queries |
| [`MouseButton`](mouse_button.md) | Supported mouse-button identifier |
| [`MouseState`](mouse_state.md) | Current and transition button queries |

Input delivery depends on backend focus, compositor/window-manager policy, and enabled callbacks.
See the generated [window platform compatibility matrix](../platform_compatibility.md) and the
[GLFW inventory](../../../maintenance/glfw_platform_inventory.md) for callback and cursor limits.
