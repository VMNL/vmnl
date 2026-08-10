# Current Graphics Limitations

Treat these as implementation snapshots, not permanent contracts. Verify the referenced code before relying on them and remove or update an entry when the repository resolves it.

## Device and Queue Selection

`Context::new()` currently ranks compatible physical devices and selects the first graphics queue. Equal-rank devices follow backend enumeration order.

Consequences:

- selection is not caller-controlled;
- equal-rank selection is not strictly deterministic across backends or machines;
- new APIs must not describe this path as fully explicit or deterministic;
- preserve the behavior unless the task intentionally changes its documented contract.

## GLFW Initialization Lock

`GLFW_INIT_LOCK` is a legacy process-wide synchronization exception. If a task touches it:

- document the local invariant it protects;
- keep it private and limited to external initialization serialization;
- do not let it own GPU resources, cache visible behavior, or influence device selection;
- do not generalize it into application-visible global state.

## 3D Status

The public 3D types remain scaffolding without an operational rendering backend. Do not claim 3D rendering support. If a task makes part of it operational, update every canonical status location describing it as scaffolding.

## GPU Test Routing

`just test-gpu` runs ignored tests only in `vmnl-gpu-tests`. It does not execute ignored library tests.

Two legacy GPU-oriented ignored tests still live outside `tests/gpu`:

- `crates/vmnl_graphics/src/vmnl_instance/tests.rs`;
- `crates/vmnl_graphics/src/2d/shape/mod.rs`.

Do not count them as executed by `just test-gpu`. When modifying their behavior, prefer migrating durable GPU coverage to `tests/gpu` rather than adding more out-of-suite tests.

New GPU/display tests under `tests/gpu` must stay ignored because `just test-gpu` selects them with `--ignored`.
