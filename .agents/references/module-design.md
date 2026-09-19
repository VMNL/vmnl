# VMNL Module Design

## Scope

Use this reference when designing or reviewing a module interface, crate boundary, facade, raw API,
backend adapter, or FFI seam. It supplements the project direction in
[`docs/architecture.md`](../../docs/architecture.md); it does not replace that contract or authorize
an architecture change.

The objective is an interface that concentrates useful behavior and complexity while preserving
VMNL's progressive path from documented defaults to explicit, raw, and backend-level control.

## Working Vocabulary

- **Module**: implementation hidden behind an interface. It may be a function, type, Rust module,
  crate, or subsystem.
- **Interface**: everything callers must know, including types, invariants, ordering, errors,
  ownership, lifecycle, costs, threading, synchronization, allocation, and platform behavior.
- **Seam**: a location where behavior or implementation can vary without changing its callers.
- **Adapter**: the implementation that connects a seam to a backend, platform, test double, or
  foreign interface.
- **Depth**: useful behavior and policy provided per unit of interface callers must understand.
- **Locality**: the degree to which knowledge, change, failure handling, and validation stay owned
  by one module instead of leaking into callers.

Use these terms as analysis tools, not as mandatory public API vocabulary.

## Establish the Design Problem

Before proposing an abstraction:

1. Identify the concrete caller and use case that require it.
2. Read the canonical contract and inspect the current implementation and tests at the affected
   boundary.
3. State which complexity the module would own and which decisions callers must retain.
4. Identify the dependencies, ownership transfers, unsafe or FFI operations, platform behavior,
   and evidence needed to validate the boundary.
5. Separate current facts, proposed policy, and unresolved decisions.

Do not introduce a seam, adapter, trait, manager, or generic parameter for a hypothetical future
variation without a current ownership, safety, testability, platform, or control requirement.

## Evaluate the Interface

An interface is useful when it gives callers leverage and keeps the corresponding knowledge local.
Evaluate it with these checks:

- **Caller burden**: callers provide only information they actually own.
- **Hidden complexity**: backend ordering, resource cleanup, validation, and unsafe details stay
  behind the boundary that owns them.
- **Progressive control**: convenient defaults do not prevent explicit configuration, raw control,
  or deliberate backend interoperability where the architecture permits them.
- **Observable policy**: resolved defaults and behavior affecting output, errors, ownership, costs,
  synchronization, or platform behavior remain inspectable and documented.
- **Error boundary**: invalid public combinations are rejected before backend or FFI calls when
  practical, and backend failures remain distinguishable.
- **Deletion test**: if the module disappeared, meaningful complexity would reappear across its
  callers. If deletion merely removes a pass-through name, the abstraction needs another concrete
  responsibility or should not exist.

A small method count is not a goal by itself. Do not compress distinct ownership, lifecycle, or
control decisions into opaque configuration solely to make an interface appear deeper.

## Place Seams Deliberately

A seam is justified when it isolates at least one real concern:

- domain independence through the `vmnl` facade;
- ownership or lifecycle policy;
- unsafe, FFI, or backend-specific behavior;
- an actual platform or implementation variation;
- a deterministic test boundary that does not weaken the production contract.

One production adapter can still justify a seam when VMNL intentionally fixes a backend, such as
Vulkan, or must isolate a foreign C ABI. Do not add a second public abstraction merely to satisfy a
generic multi-adapter rule.

Keep test-only seams private whenever callers do not need the variation. A mock, stub, Null backend,
or in-memory adapter validates behavior at that seam only; it does not prove native platform, GPU,
hardware, FFI, or remote-system behavior.

## Classify Dependencies and Evidence

Use the dependency shape to choose the interface and its validation:

| Dependency | Preferred ownership | Required evidence |
| --- | --- | --- |
| Pure or in-process logic | Keep behind the owning module's interface. | Deterministic unit or API behavior. |
| Private replaceable mechanism | Use an internal seam only when a real test or implementation variation needs it. | Tests through the owning interface plus adapter-specific checks. |
| Platform or graphics backend | Isolate backend details without claiming unsupported replaceability. | Platform or GPU evidence for the exercised configuration. |
| C backend or submodule | Keep unsafe declarations and lifetime translation in a narrow private adapter. | ABI/layout probes and integration tests against the pinned backend. |
| External or remote system | Expose timeout, cancellation, ordering, and failure behavior at the owning interface. | Controlled seam tests plus identified real-system evidence. |

Testing through a higher interface does not automatically make lower adapter tests obsolete. Keep
the lowest tests that prove distinct safety, ABI, platform, or failure-handling invariants.

## Compare Alternatives When It Matters

For a public, cross-domain, FFI, or difficult-to-reverse interface, compare at least two credible
designs before selecting one. Make the alternatives materially different, for example:

- minimum caller surface;
- maximum explicit control;
- simplest common path while preserving an escape hatch.

Compare them on caller burden, progressive control, ownership, locality, test seams, error behavior,
runtime costs, compatibility, and migration. Prefer the smallest interface that satisfies the
current use cases without hiding decisions VMNL promises callers they can control.

## Decision Output

A design is decision-ready when it identifies:

- the callers and use cases;
- the proposed interface and its complete observable contract;
- the complexity hidden behind it;
- the seam and adapter ownership;
- the explicit and raw control retained by callers;
- rejected alternatives and their concrete failure modes;
- the tests, probes, and environment evidence required to validate it;
- unresolved choices that still require authority or backend evidence.
