# Input

| Item | Role |
|---|---|
| [`Cursor`](cursor.md) | Shareable standard or custom native cursor resource |
| [`CursorMode`](cursor_mode.md) | Cursor visibility and confinement mode |
| [`Input`](input.md) | Window-owned keyboard/mouse snapshot |
| [`Key`](key.md) | Supported key identifier |
| [`KeyboardState`](keyboard_state.md) | Current and transition key queries |
| [`Modifiers`](modifiers.md) | Modifier flags attached to mouse-button events |
| [`MouseButton`](mouse_button.md) | Supported mouse-button identifier |
| [`MouseState`](mouse_state.md) | Current and transition button queries |
| [`StandardCursor`](standard_cursor.md) | System cursor shape identifier |

Input tracking depends on backend focus and compositor/window-manager policy. Public event delivery
can be disabled without disabling the keyboard/mouse-button snapshot.
See the generated [window platform compatibility matrix](../platform_compatibility.md) and the
[GLFW inventory](../../../maintenance/glfw_platform_inventory.md) for callback and cursor limits.
