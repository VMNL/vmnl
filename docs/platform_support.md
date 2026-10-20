# Platform Support

## Support policy

- Linux is the tier-1 target. Ubuntu is the current blocking CI environment; other distributions
  require environment-specific evidence.
- Windows and macOS are tier-2 targets until their native probes and release artifacts are
  qualified.
- Mobile and Web are outside the VMNL scope.
- Platform support applies only to the domains and backends explicitly exercised on that platform.

The stage-3 schema-2 input expansion passed every eligible native case in
[CI run #38206068512](https://github.com/VMNL/vmnl/actions/runs/38206068512) at `32f8b3e`:

| Backend | Passed named-key cases | Passed extended cases, including keys |
| --- | --- | --- |
| X11 | 118 | 161 |
| Nested Weston Wayland | 118 | 161 |
| Win32 | 116 | 142 |
| Cocoa | 110 | 152 |

Each eligibility manifest lists all 120 named keys and records excluded cases with their reasons;
excluded inputs are not passes. These results cover modifier/button combinations, eligible text,
Linux bounded repeat, and held-input focus loss through GLFW. Windows/macOS remain experimental
pending ten consecutive qualified runs of the current injector, schema, workload, and runner image.
Earlier CI links in the matrix below cover the schema-1 operations. See
[extended native cases](testing.md#extended-native-cases-issue-91-stage-3) for the exact profile limits.
The representative public VMNL/Vulkan scenario also passed locally on Fedora 44 with
Hyprland 0.56.2/Xwayland, Intel Iris Xe and Mesa 26.2.3. See the
[public input execution record](testing.md#native-input-contract) for its exact scope; this is
automatic runtime evidence and does not qualify other hardware/backends or manual graphics.
The 2026-10-13 public X11 expansion passed press/release, independent scancode and snapshot
checks for 118 of the 120 named keys in that local environment. `World2` lacks an X11 physical
fixture and `F25` lacks an active `FK25` name/alias; both are explicit exclusions. XKB lock state
is restored and checked. This does not qualify public text, modifier combinations, repeat,
focus loss/batch boundaries or other desktop environments. The first pointer-motion attempt
failed before keyboard injection; the diagnostic and final runs passed, with no established
cause for the initial timeout. Details and counts are in the linked execution record.

On 2026-10-15, the local Hyprland/Xwayland NoApi profile passed three consecutive complete
executions: 19 baseline operations and 161 eligible extended cases per run. Initial Xlib contents,
parent-side XTEST pointer preparation and hiding for held-input focus loss are required by this
probe profile. See [native probe preparation and results](testing.md#continuous-integration). These automatic
backend results remain separate from public VMNL, other desktops and physical-device validation.

## Validation Matrix

| Platform | CI validation | Local Justfile | Status |
|----------|---------------|----------------|--------|
| Ubuntu Linux | Configured: build, headless tests, GLFW Null, Weston Wayland nested in Xvfb/Openbox and Xvfb/Openbox X11, with XTEST `A` and Shift+A, mapped mouse buttons, pointer movement/leave/enter, and vertical/horizontal scroll. Hosted Xvfb's 10-button map reaches GLFW buttons 1–6; buttons 7–8 remain unsupported there. | `just input-test x11`, `just input-test wayland`, and opt-in GPU-backed `just input-test-vmnl x11`. | Blocking NoApi probes. The Linux job in [CI run #38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980) passed 17 nested-Wayland and 19 X11 operations, including the corrected leave/enter path inside Weston, GLFW buttons 1–6, pointer motion, and all scroll directions. Buttons 7–8 are explicitly not injectable with the hosted map. The Wayland boundary case requires the default desktop shell top panel. The representative public VMNL scenario passed locally on the Fedora/Xwayland stack recorded above; Ubuntu CI does not execute that GPU scenario. |
| Other Linux distributions | No distribution matrix. | Best effort. | Backend guarantees remain environment-scoped. |
| Windows | Configured: build, headless tests and GLFW Null; visible `A` and Shift+A keyboard probes, GLFW mouse buttons 1–5, pointer movement, leave/enter, and vertical/horizontal scroll probes use `SendInput`/`SetCursorPos` and are experimental. GLFW buttons 6–8 have no Win32 mapping. | `just input-test win32` on an accessible interactive desktop. | Experimental probes are non-blocking until qualified. Runs [#38089252090](https://github.com/VMNL/vmnl/actions/runs/38089252090) and [#38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980) passed the current keyboard, button, motion, boundary, and scroll probes; ten consecutive runs are still required. |
| macOS | Configured: build, headless tests and GLFW Null; visible `A` and Shift+A keyboard probes, all eight GLFW mouse buttons, pointer movement, leave/enter, and vertical/horizontal scroll probes use `CGEventPost` and are experimental. | `just input-test cocoa` with event-posting access. | Experimental probes are non-blocking until qualified. Runs [#38089252090](https://github.com/VMNL/vmnl/actions/runs/38089252090) and [#38109958980](https://github.com/VMNL/vmnl/actions/runs/38109958980) passed the current keyboard, button, motion, boundary, and scroll probes; ten consecutive runs are still required. |

`just bootstrap` invokes `./deps`, which requires `/etc/os-release` and only contains Linux
package-manager paths. CI invokes Cargo directly and never invokes the bootstrap recipe.

## Public input client qualification

On 2026-10-14, the separate public VMNL client completed three consecutive automatic local
X11/Vulkan protocol-3 runs on the Fedora/Hyprland/Xwayland stack: 848 checks, 828 passes,
20 explicit exclusions and no failures or unexecuted checks. In addition to 118 named keys,
physical aliases, buttons 1–6, motion and scrolling, this qualifies the eligible standard modifier
chords, Caps/Num key/button flags, ASCII a, actual repeat, leave/enter/hover, public poll controls
and a focus transfer between two owned VMNL windows. World2, F25, buttons 7–8 and unmodified
é/🦀 remain excluded. This is synthetic public API evidence; other backends, non-ASCII text,
IME composition and physical input devices retain their separate qualification limits. See the
[execution record](testing.md#extended-public-client-execution-protocol-3) for prerequisites,
startup preparation and failed diagnostic attempts. The client directly depends only on vmnl;
OS injection codes and timestamp pacing remain in the X11 driver.

On 2026-10-15, three consecutive executions without instrumentation revalidated the public scroll
oracle on the local Fedora/Hyprland/Xwayland/Mesa stack: each completed 848 checks with 828 passes,
20 exclusions and no failures or unexecuted checks. All four scroll directions passed with the
100 ms quiet interval and fixed five-second deadline. The
[revalidation record](testing.md#scroll-oracle-native-revalidation) includes exact versions,
source identification and the preceding text-fixture failure. This is automatic local evidence.

## Runtime Constraints

Joystick/gamepad native coverage is separate from the keyboard/mouse qualifications above.
On 2026-10-18, the opt-in Linux uinput scenarios passed three consecutive executions each with
the X11 profile selected: public `InputRuntime` 244/244 report checks and `Window::poll_events`
268/268 per execution, without failures, exclusions or unexecuted checks. Both hat_buttons
settings and consumer orders run in fresh clients. An explicitly authorized temporary uinput
ACL enabled these runs; its original root:root 0600 state was restored and verified afterwards.
This is local virtual-device evidence, not physical-controller, independent backend-selection,
rendered-frame or human graphical validation.
On 2026-10-19, the native Wayland joystick profile passed three consecutive runs per mode on
Hyprland 0.56.2: runtime 248/248 and window 272/272 checks, including one protocol-exchange
proof per fresh client with X11 unavailable. Both hat_buttons settings and consumer orders
were exercised. The temporary uinput ACL was restored and all owned devices were removed.
This qualifies the exercised Linux Wayland virtual-device paths. Windows/macOS virtual
injection and physical-controller behavior remain unqualified. Deterministic and Null results
do not establish present-device behavior.
The `CI` workflow prepares an opt-in `native_joystick` matrix for Ubuntu Xvfb/Openbox and
nested Weston/Pixman, with explicit lavapipe Vulkan in window mode and three repetitions
per backend/mode. These hosted profiles remain unqualified until their native JSON reports
are observed; local Hyprland evidence does not qualify them. Ordinary CI remains headless
for joystick scenarios. See [Linux joystick CI](testing.md#linux-joystick-ci).
The physical operator protocol is prepared for Linux (`x11`/`wayland`), Windows (`win32`) and
macOS (`cocoa`), in standalone runtime and window modes, with explicit profiles, tolerances
and exclusions. Host/backend mismatches fail; Linux-only virtual injection is kept separate.
No controller or native Windows/macOS host was available for this preparation on 2026-10-19;
physical qualification and native Windows/macOS compilation/execution remain pending.
CI compilation and portable oracles do not qualify a present controller. See the
[physical protocol](testing.md#physical-joystick-operator-protocol).
See the [scenario matrix](api/maintenance/joystick_capability_matrix.md#functional-linux-coverage-added-for-issue-94)
and [opt-in commands](testing.md#virtual-joystickgamepad-scenarios-issue-94).

- Visual examples and GPU tests require a Vulkan-capable GPU, a Vulkan loader, GLFW, and a display server.
- Headless verification uses `just test`; it excludes GPU/display tests.
- GLFW portability probes use `ClientApi::NoApi`; they create no Vulkan instance, surface, or GPU resource.
- Native `A` and Shift+A probes require a focused visible window. Mouse probes require a hovered window
  before injection. X11 and nested Weston inject every GLFW button mapping within the active X11
  pointer map, plus movement, leave/enter, and vertical/horizontal scroll. The hosted 10-button
  Xvfb map cannot exercise GLFW buttons 7–8. Win32 injects buttons 1–5; Cocoa injects all eight.
  Both also probe movement, leave/enter, and vertical/horizontal scroll; their first CI observations
  exist but do not yet meet the qualification threshold.
  X11 requires XTEST and an EWMH window manager, Windows requires an accessible desktop at the
  same integrity level, and macOS requires event-posting access.
- The nested Weston probe covers XTEST → Xvfb → Weston X11 backend → Wayland client input, not a
  native Wayland `libei`, portal, or `uinput` seat.
- Compile GPU tests without a display with `just test-gpu-compile`.

The generated [window compatibility matrix](api/reference/window/platform_compatibility.md) is
canonical for public VMNL operations. The exhaustive
[GLFW inventory](api/maintenance/glfw_platform_inventory.md) also records platform-sensitive GLFW
3.4 functions not currently used by VMNL. A successful Weston, Xvfb, Win32, or Cocoa probe only
qualifies the exact backend and recorded runner environment.

Platform badges in the root README describe project targets, not a guarantee of CI validation.
