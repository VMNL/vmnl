# Current Graphics Limitations

Treat these as implementation snapshots, not permanent contracts. Verify the referenced code before relying on them and remove or update an entry when the repository resolves it.

## Device and Queue Selection

`Context::new()` currently ranks compatible physical devices and selects the first graphics queue. Equal-rank devices follow backend enumeration order.

Consequences:

- selection is not caller-controlled;
- equal-rank selection is not strictly deterministic across backends or machines;
- new APIs must not describe this path as fully explicit or deterministic;
- this automatic-only path does not yet satisfy the target replaceability contract in
  `docs/architecture.md`;
- preserve current behavior unless the task intentionally adds an explicit selection path while
  retaining the documented default.

## GLFW Initialization Lock

`GLFW_INIT_LOCK` is a legacy process-wide synchronization exception. If a task touches it:

- document the local invariant it protects;
- keep it private and limited to external initialization serialization;
- do not let it own GPU resources, cache visible behavior, or influence device selection;
- do not generalize it into application-visible global state.

## 3D Status

The public 3D types remain scaffolding without an operational rendering backend. Do not claim 3D rendering support. If a task makes part of it operational, update every canonical status location describing it as scaffolding.

## GPU Test Routing

`just test-gpu` runs ignored tests only in `vmnl-gpu-tests`. New Vulkan, surface, presentation, or
GPU/display tests under `tests/gpu` must stay ignored because this recipe selects them with
`--ignored`.

GLFW `NoApi` behavior without Vulkan belongs in `tests/platform` and is validated separately with
the applicable platform recipes.
