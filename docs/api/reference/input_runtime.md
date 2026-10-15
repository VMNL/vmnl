# `InputRuntime`

## Public path and maturity

Import paths: `vmnl::InputRuntime`, `vmnl::InputRuntimeBuilder`, and `vmnl::InputRuntimeConfig`.
Status: experimental; exposes shared GLFW lifecycle, initialization configuration, standalone
joystick sampling, connection notifications, gamepad mapping updates, and GLFW joystick user
pointers.

## Purpose and use cases

Acquires the same process-wide GLFW runtime used by `Context` and `Window`, without constructing a
Vulkan instance or window. It owns global input configuration and standalone joystick operations.

## Public API

| Member | Contract |
|---|---|
| `InputRuntime::acquire()` | Reuse the active runtime/configuration or initialize GLFW with defaults. |
| `InputRuntime::builder()` | Start an explicit first-initialization configuration. |
| `InputRuntime::configuration()` | Inspect the settings resolved by the active runtime. |
| `InputRuntimeBuilder::hat_buttons(bool)` | Choose whether GLFW adds hat directions to joystick button arrays. |
| `InputRuntimeBuilder::build()` | Acquire the shared runtime with the builder's optional setting. |
| `InputRuntimeConfig::hat_buttons()` | Read the resolved GLFW hat-buttons setting. |
| `InputRuntime::sample_joystick(id)` | Copy one slot's raw data and optional mapped gamepad state. `Ok(None)` means absent. |
| `InputRuntime::poll_events()` | Process GLFW events and return this handle's joystick connection events. |
| `InputRuntime::set_joystick_callback(callback)` | Run a per-handle callback synchronously with each connection event and its raw user pointer. |
| `InputRuntime::unset_joystick_callback()` | Remove this handle's synchronous connection callback. |
| `InputRuntime::update_gamepad_mappings(text)` | Add or replace ASCII SDL gamepad mapping lines. |
| `InputRuntime::joystick_user_pointer(id)` | Read GLFW's process-wide raw pointer; `unsafe`. |
| `InputRuntime::set_joystick_user_pointer(id, pointer)` | Set GLFW's process-wide raw pointer; `unsafe`. |

## Construction, defaults, and validation

`hat_buttons` defaults to `true`, matching GLFW 3.4. An omitted setting adopts the active runtime's
configuration. An explicit setting is accepted when it matches; a different value returns
`GlfwInitializationConfigConflict` before another GLFW call. Once the last VMNL owner drops, a later
acquisition can initialize GLFW with a new setting.

Gamepad mappings must be ASCII and contain no NUL byte. GLFW accepts one or more mapping lines and
replaces an existing entry when the GUID matches. Its parser may report a malformed mapping through
the error callback while returning `true`; VMNL returns an error for either signal.

Joystick event delivery is opt-in. `poll_events` subscribes this handle on its first call; callback
registration subscribes immediately. Changes reported before subscription are not replayed.
`set_joystick_callback` invokes client code synchronously and also queues each event for this
handle's next `poll_events` call. If a callback panics, VMNL queues the event for every subscriber,
skips further user callbacks for the rest of that GLFW call, then resumes the panic after GLFW
returns to Rust.

## Units, coordinates, and valid ranges

Not applicable to runtime configuration. Raw joystick and mapped gamepad values are documented in
[joystick/gamepad data types](joystick_gamepad.md).

## Ownership, lifecycle, and threading

`InputRuntime`, `Context`, and their windows share one internal `Rc` owner. Each `InputRuntime`
acquisition returns a distinct, non-`Clone` handle. GLFW remains initialized until the last runtime,
context, window, or cursor owner drops. The handle is neither `Send` nor `Sync`; acquire and drop it
on GLFW's platform thread. GLFW joystick sampling, event processing, callback registration, and
mapping updates follow GLFW 3.4's main-platform-thread rule. GLFW callbacks run on the event-pumping
thread. VMNL catches joystick callback panics at the C boundary and resumes them after the enclosing
GLFW call returns to Rust. Queues still receive the event before the panic resumes; additional user
callbacks are skipped during that GLFW call. Recursive event processing from that callback is
unsupported.

User pointers are process-wide per-slot raw pointers. GLFW does not own or synchronize the pointed-to
allocation and clears it when the slot disconnects. The synchronous callback receives the pointer
while GLFW still exposes it, including during disconnect; deferred `Event` values do not retain it.
Dereferencing or retaining the pointer still follows the raw-pointer safety contract.

## Errors, panics, and failure conditions

Initialization failure returns `GlfwInitFailed`. A conflicting explicit setting returns the
structured `GlfwInitializationConfigConflict` variant with active and requested values. Acquiring
while another thread owns VMNL's active GLFW runtime returns `InvalidState`. Sampling or mapping
failures return `GlfwInputOperationFailed`; invalid mapping text returns `InvalidGamepadMapping`.

## Allocation, transfers, synchronization, and GPU cost

Acquisition allocates one shared owner on first initialization. Reuse adds an `Rc` owner. Sampling
copies variable-length arrays and owned strings, so it may allocate; exact allocation count is not
specified. The first sample lazily initializes GLFW's joystick subsystem; its latency is
unspecified. GLFW supplies fields through successive queries, so a device change during sampling is
not an atomic snapshot. Runtime event polling processes the shared native event queue, but drains
only this handle's joystick queue. It does not drain window events or another runtime handle's
queue. Runtime operations create no Vulkan resources and perform no GPU work.

## Platform, Vulkan, and display constraints

This API does not initialize Vulkan or create a window, but GLFW still needs an available backend
and its required system services. GLFW initialization, joystick sampling, event processing,
callback registration, and mapping updates follow GLFW 3.4's thread rules. The hat-buttons hint
applies only at the next GLFW initialization.

## Example and related types

```rust,no_run
# extern crate vmnl;
use vmnl::{InputRuntime, JoystickId};

fn main() -> vmnl::VMNLResult<()> {
    let runtime = InputRuntime::builder().hat_buttons(false).build()?;
    assert!(!runtime.configuration().hat_buttons());
    if let Some(sample) = runtime.sample_joystick(JoystickId::Joystick1)? {
        println!("{:?}: {} raw axes", sample.name(), sample.axes().len());
    }
    Ok(())
}
```

Related: [`Context`](context.md), [`VMNLErrorKind`](errors/vmnl_error_kind.md),
the [joystick/gamepad data types](joystick_gamepad.md), and [joystick/gamepad capability
coverage](../maintenance/joystick_capability_matrix.md).

The runnable [`joystick_runtime` example](../../../examples/input/joystick_runtime/src/main.rs)
samples all sixteen slots without constructing a `Context`, Vulkan instance, or window. Run it with
`just run joystick_runtime` on a host where GLFW can initialize its platform backend.
