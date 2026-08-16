# GLFW portability protocol

Use this protocol for every newly used GLFW function and every newly observed backend limitation.
The canonical source is `tools/data/glfw_platform_inventory.toml`; generated Markdown must never be
edited directly.

## Required flow

```text
discovery
-> inventory entry
-> primary source or unverified status
-> deterministic test or explicit justification
-> docs-api-update
-> public Rustdoc update when VMNL exposes the behavior
-> docs-api-check
```

1. Record the exact GLFW C symbol, Rust wrappers, introduction version, category, and VMNL usage.
2. Record each relevant backend separately with one allowed status: `supported`, `unsupported`,
   `conditional`, `best-effort`, or `unverified`.
3. Distinguish callback errors, no-ops, sentinel getter values, and asynchronous requests.
4. Link the GLFW 3.4 reference or another primary platform source. If the behavior is not proved,
   use `unverified`; do not infer a guarantee from another backend.
5. Add a focused subprocess probe when the operation can be made deterministic. Otherwise record
   an explicit justification explaining why no automated assertion is valid.
6. Add or update the public compatibility row and Rustdoc when a VMNL API exposes the behavior.
7. Regenerate and review both generated GLFW pages, then run the non-mutating check.

## Dependency audit

Any change to `glfw`, `glfw-sys`, the bundled GLFW C version, the fork URL, or its revision requires
a complete re-audit. Update the `[audit]` table only after comparing upstream release notes, error
codes, backend remarks, and bundled header constants. The checker rejects a manifest/inventory
mismatch.

VMNL intentionally enables `glfw/src-build`. Allowing `pkg-config` to substitute a distribution
GLFW would make the audited C implementation depend on the host and invalidate backend evidence.

## Test qualification

- Null proves window creation and error handling without a display; it does not prove native-window behavior.
- Weston/Pixman proves only the recorded Wayland compositor environment.
- Xvfb/Openbox proves only the recorded X11 server and EWMH window-manager environment.
- Win32 and Cocoa remain experimental until ten consecutive successful runs use the same probe
  schema, GLFW revision, and runner image. Any change resets the count.
- A backend that did not execute must be reported as failed or unqualified, never as passed.

GPU and Vulkan surface behavior stays in `tests/gpu`; `tests/platform` always uses
`ClientApi::NoApi`.
