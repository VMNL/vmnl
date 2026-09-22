# Input

| Item | Role |
|---|---|
| [`Cursor`](cursor.md) | Shareable standard or custom native cursor resource |
| [`CursorBuilder`](cursor_builder.md) | Borrowed RGBA8 custom-cursor configuration |
| [`CursorMode`](cursor_mode.md) | Cursor visibility and confinement mode |
| [`Input`](input.md) | Window-owned keyboard/mouse snapshot |
| [`Key`](key.md) | Supported key identifier |
| [`KeyboardState`](keyboard_state.md) | Current and transition key queries |
| [`Modifiers`](modifiers.md) | Modifier flags attached to keyboard, mouse-button, and legacy text events |
| [`MouseButton`](mouse_button.md) | Supported mouse-button identifier |
| [`MouseState`](mouse_state.md) | Current and transition button queries |
| [`Scancode`](scancode.md) | Platform-specific physical-key identifier |
| [Window input modes](modes.md) | Sticky keys/buttons and lock-key modifier reporting |
| [`StandardCursor`](standard_cursor.md) | System cursor shape identifier |
| [`StandardCursorBuilder`](standard_cursor_builder.md) | Deferred standard-cursor allocation |

Input tracking depends on backend focus and compositor/window-manager policy. Public event delivery
can be disabled without disabling the keyboard/mouse-button snapshot.
See the generated [window platform compatibility matrix](../platform_compatibility.md) and the
[GLFW inventory](../../../maintenance/glfw_platform_inventory.md) for callback and cursor limits.
