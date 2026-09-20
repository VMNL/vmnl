# Network Backend Integration

## Status and Scope

This document defines the pre-implementation contract between the planned `vmnl_network` Rust
API and its C backend. No network crate, backend submodule, exported header, or concrete ABI exists
in the current workspace. Exact signatures and runtime behavior therefore remain unresolved until
the backend revision is selected and inspected.

The C backend is developed and governed outside VMNL. VMNL consumes it but does not own its
implementation. The expected integration mechanism is a Git submodule pinned to an exact commit.
Its repository, workspace path, and first pinned revision remain to be selected.

## Ownership Boundary

| Surface | Owner |
| --- | --- |
| Transport, framing, wire behavior, and C implementation | C backend repository |
| Exported C headers, ABI version, allocation rules, callbacks, and backend errors | C backend repository |
| Backend build and platform support | C backend repository |
| Pinned backend revision and parent-repository build integration | VMNL |
| Rust FFI declarations and ABI checks | `vmnl_network` |
| Safe, idiomatic Rust API and error mapping | `vmnl_network` |
| Optional facade exposure | `vmnl` |
| Rust-side tests, examples, and public Rustdoc | VMNL |

VMNL agents may read the pinned backend headers, ABI documentation, tests, build metadata, and
implementation to integrate the Rust API. They must not modify backend files or advance the pinned
revision. Backend changes belong to the backend repository and its non-AI development process.

## Source of Truth

When integrating observed backend behavior, use this order:

1. the approved C/Rust boundary documented here;
2. the exact pinned backend revision, with exported headers and ABI documentation authoritative
   within that revision;
3. backend tests, build definitions, and symbol export metadata;
4. backend implementation, only to clarify behavior omitted by its public contract.

An implementation detail does not become a VMNL promise. A contradiction between a header,
documentation, test, and implementation blocks the affected safe abstraction until the backend
owner resolves it or explicitly documents the intended behavior.

## Layer Boundary

```text
application
  -> vmnl facade (network feature)
    -> vmnl_network safe Rust API
      -> private unsafe FFI layer
        -> pinned C backend
```

The FFI layer must remain private. Whether it is an internal module or a separate `-sys` crate is
decided only after the backend build and header structure are known. A headless network client must
not acquire Vulkan, GLFW, windowing, or graphics initialization through this path.

## Required C Contract

The Rust integration must not start until the selected backend revision makes these points
inspectable:

- exported symbols and calling convention;
- ABI versioning or another deterministic compatibility check;
- exact integer, size, pointer, and structure representations;
- creator, owner, validity interval, mutability, and destructor for every handle and buffer;
- allocator and deallocator pairing for memory crossing the boundary;
- nullability and length rules for pointers, buffers, strings, and arrays;
- error capture lifetime and whether error state is global, thread-local, per-handle, or returned;
- callback thread, ordering, reentrancy, cancellation, and shutdown behavior;
- backend-created threads and their lifecycle;
- initialization, partial-failure cleanup, and shutdown ordering;
- supported targets, required toolchain, and static or dynamic linkage requirements.

Missing information is a blocked contract, not permission to infer a safe Rust guarantee from the
current implementation.

## FFI Invariants

- FFI declarations use the exact C ABI types. Rust-native layout is never exposed directly.
- Mirrored C structures use `#[repr(C)]` and have size, alignment, and field-offset checks where
  their layout crosses the boundary.
- C enums are represented as ABI-compatible integers and constants unless the backend guarantees
  that every possible value is valid for a closed Rust enum.
- Opaque C handles stay opaque. Rust stores them in private wrappers and never dereferences them.
- Pointer nullability is explicit. Pointer-plus-length inputs are validated before the call, and
  returned ranges are validated before creating Rust slices.
- C strings are not assumed to be UTF-8 unless the backend contract guarantees it.
- No Rust panic or foreign exception may unwind across the FFI boundary.
- `unsafe` is confined to the narrow FFI adapter and justified by local `// SAFETY:` invariants.
- ABI compatibility is checked before normal backend use; an incompatibility produces a structured,
  actionable error rather than a silent fallback.

## Ownership and Lifetimes

- Memory is released by the allocator family that created it. Rust does not free backend memory
  unless the backend exports that exact operation.
- Safe owning wrappers release initialized handles with `Drop`; partial initialization is cleaned
  up in reverse ownership order.
- A handle is not `Clone` unless the backend exposes a proven retain/release or duplication
  contract.
- A wrapper is not `Send` or `Sync` unless backend thread-safety and callback behavior prove the
  corresponding Rust invariant.
- The backend cannot retain borrowed Rust storage beyond a call unless registration,
  unregistration, lifetime, concurrency, and shutdown are all explicit and enforceable.
- Destruction cannot race a call or callback that may still access the handle or its Rust context.

## Callbacks and Execution

The safe API must expose the backend's real execution model. It must not present synchronous,
single-threaded, ordered, or reentrancy-free behavior unless the backend guarantees it.

For an asynchronous callback carrying Rust state, unregistration or shutdown must guarantee that
no later callback can use that state. If the backend cannot provide that guarantee, the callback
API remains unsafe or unavailable until VMNL can own a join or drain protocol. Callback trampolines
must contain Rust panics and must not let them cross into C.

VMNL does not add a hidden Rust worker runtime. Any backend-created thread or optional managed Rust
execution mode must expose its lifecycle, synchronization, blocking, and shutdown costs.

## Errors and Shutdown

- Inputs are validated on the Rust side before FFI when their invalidity is knowable locally.
- Backend error information is captured before another backend call can overwrite it.
- Known backend errors map to structured Rust variants; unknown codes remain observable with their
  raw value.
- Sentinel values are not conflated with valid data.
- The Rust API claims idempotent shutdown, retry safety, or post-error handle validity only when the
  backend contract proves it.
- A failed initialization reports which resources remain owned and ensures that the safe wrapper
  cannot be used in a partially initialized state.

## Submodule and Build Integration

- The parent repository pins an exact backend commit, never a floating branch.
- Advancing that commit is a separately reviewable dependency change with an ABI and behavior
  impact assessment.
- The build must fail with an actionable diagnostic when the backend, toolchain, target, symbols,
  or ABI version is incompatible.
- Static versus dynamic linkage, source build versus prebuilt library, and generated versus manual
  bindings remain open until the backend artifacts are available.
- Binding generation, if selected, must be reproducible; generated files are not edited manually.

## Validation Evidence

Validation must separate these claims:

| Evidence | What it proves |
| --- | --- |
| Header and binding comparison | Rust declarations match the selected public C surface. |
| C compile/link probe | Required headers and symbols are consumable by the selected toolchain. |
| Layout and ABI probes | Cross-boundary values match on the tested target. |
| Rust unit tests with a controlled seam | Rust validation, ownership state, and error mapping. |
| Integration tests with the pinned backend | Real lifecycle, errors, callbacks, and data exchange. |
| Platform or remote-network runs | Behavior in the reported OS, architecture, and network conditions. |

A mock, stub, or Rust-only test does not validate the C backend. One target's ABI result does not
establish portability. Runtime, platform, and remote-network evidence must report their tested
environment and remain distinct from compilation evidence.

## Deferred Decisions

These decisions require the selected backend revision and must not be guessed now:

- submodule URL, path, and initial commit;
- public C headers, symbol visibility, and ABI version mechanism;
- internal FFI module versus separate `-sys` crate;
- manual bindings versus reproducible generation;
- static versus dynamic linkage and supported target triples;
- polling, callback, or mixed execution model;
- allocator, error, cancellation, and shutdown conventions;
- wire-protocol compatibility and stability policy;
- final safe Rust types, traits, and facade feature name.

Public Rustdoc becomes authoritative for the safe Rust API after those choices are implemented.
The C ABI and network protocol become compatibility surfaces only when their own documentation
explicitly declares them stable, as required by [`deployment.md`](deployment.md).
