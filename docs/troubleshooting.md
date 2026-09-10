# Troubleshooting

Use the smallest command that exercises the failing layer.

## shaderc Not Found

Hypothesis: `shaderc` cannot be located.

```bash
pkg-config --exists shaderc
pkg-config --modversion shaderc
pkg-config --variable=libdir shaderc
```

If discovery succeeds, retry the smallest failing Just recipe. Otherwise, run `just bootstrap` or install the shaderc development package for the active distribution. See [Build](build.md#shaderc-discovery).

## shaderc CMake Compiler Cache Mismatch

Hypothesis: a source-build cache was created with a different C or C++ compiler. The diagnostic contains `You have changed variables that require your cache to be deleted`, followed by changed `CMAKE_C_COMPILER` or `CMAKE_CXX_COMPILER` values.

Verify native discovery first:

```bash
pkg-config --exists shaderc
pkg-config --modversion shaderc
pkg-config --variable=libdir shaderc
```

If discovery succeeds, remove only the generated `shaderc-sys` artifacts and retry the failing recipe:

```bash
cargo clean -p shaderc-sys
just test-unit
```

Do not remove the workspace `target/` directory. If discovery fails, install the shaderc development package instead of treating the CMake or `gmock` error as a VMNL test failure.

## Vulkan Loader or Driver Failure

Hypothesis: the Vulkan loader or GPU driver is unavailable.

```bash
vulkaninfo
```

- If `vulkaninfo` fails, repair the loader or driver before debugging VMNL.
- If it succeeds but an example fails, retry with `just build d2_shapes` and retain the first Vulkan error.
- Keep shaderc diagnosis separate: shader compilation discovery does not establish driver availability.

## GLFW or Display Failure

Hypothesis: GLFW cannot initialize a window or the display server is unavailable.

- Run `just test` to verify headless behavior.
- Compile display-dependent tests with `just test-gpu-compile`.
- Run `just test-gpu` only in a graphical session with Vulkan available.

If headless tests pass and GPU tests cannot create a window, the failure is environmental rather than an API contract regression.

## Joystick Connected but Sticks Do Not Respond

If GLFW reports joystick presence and changing raw axes but `is_gamepad()` is
false, the device lacks a matching gamepad mapping. Set `VMNL_GAMEPAD_MAPPINGS`
to an ASCII SDL mapping file before creating `Context`. VMNL reads it once during
context creation, using the process working directory for relative paths. It
performs no automatic download. Mappings affect GLFW globally until termination.
An unreadable file, NUL/non-ASCII text, or parser rejection returns `InvalidState`.
A successfully parsed file may still lack a mapping for the device's GUID/platform.

For the Linux Xbox 360 device reporting GUID `030000005e0400008e02000045050000`,
run from the repository root:

```bash
VMNL_GAMEPAD_MAPPINGS=examples/window/events_input/gamecontrollerdb.txt just run window_events_input
```

The supplied mapping adapts the SDL_GameControllerDB Linux Xbox 360 layout:
left axes 0/1, right axes 3/4, and stick clicks 9/10. The user's raw samples confirm
left-axis movement; right-stick axes and click buttons remain to be verified on
the physical device. The mapping deliberately covers only VMNL's current stick
and click inputs. It is not a universal mapping for all controllers.

Test both sticks through the cardinal directions, centering, and both clicks.
If a control is incorrect, compare raw axes/buttons before changing the mapping;
do not infer a generic layout from the number of axes alone.

### Inspect a Mapping That Still Produces No Stick Events

Enable diagnostics explicitly and press P in the example window:

```bash
VMNL_GAMEPAD_DIAGNOSTICS=1 VMNL_GAMEPAD_MAPPINGS=examples/window/events_input/gamecontrollerdb.txt just run window_events_input
```

Each press prints to stderr: the configured file, present slots, names, GUIDs,
mapping availability, raw axes/buttons, hat count, and mapped state. Capture
one sample centered, then samples while holding each stick or click. This mode
only reads state and does not change mappings. Without the environment variable,
P produces no extra diagnostic output.

GLFW can parse a mapping file successfully yet reject its application to a device
if its GUID/platform differs or referenced controls exceed that device's axes or
buttons. The raw button array may include synthesized hat directions. Compare
raw and mapped samples before changing any mapping indices. VMNL still consumes
only slot 1 even though this diagnostic reports all present slots.
