---
name: brainstorm-vmnl
description: Run explicit, read-only VMNL brainstorms that compare design options before implementation.
---

# Brainstorm VMNL

## Outcome

Produce a decision-ready analysis without implementing the decision. Distinguish established facts,
applicable invariants, assumptions, options, trade-offs, unresolved decisions, and evidence still
needed.

Do not edit code, documentation, configuration, issues, pull requests, or other external state during
the brainstorm. A later explicit implementation request starts a separate change workflow under the
applicable repository and domain instructions.

## Establish the Decision Context

1. State the concrete decision being examined and what would change depending on the answer.
2. Inspect the current worktree and only the repository sources needed to establish current behavior.
3. Follow [`AGENTS.md`](../../../AGENTS.md) and its domain routing. For a feature, public API,
   architecture, module, or roadmap decision, read
   [`docs/architecture.md`](../../../docs/architecture.md).
   For a module interface, crate boundary, facade, raw API, backend adapter, or FFI seam, also read
   [`module-design.md`](../../references/module-design.md).
4. Treat canonical documentation as intended contract and tests or implementation as evidence of
   current behavior. Report contradictions and their owner instead of silently resolving them.
5. Mark claims as facts, assumptions, or unknowns. Cite the file, test, specification, measurement,
   or user decision supporting each material fact.

## Explore the Design Space

- Extract the applicable project invariants from their canonical owners instead of duplicating them
  in the brainstorm.
- Identify affected domains, ownership boundaries, public contracts, lifecycle, threading,
  synchronization, allocations, performance costs, error behavior, platform requirements, and
  validation boundaries when relevant.
- Compare only credible options, including retaining the current design when that is viable.
- For each option, explain benefits, costs, failure modes, reversibility, migration impact, and the
  additional control it exposes or removes.
- Reject speculative abstractions that have no current use case. Identify the smallest experiment or
  measurement that could resolve a material uncertainty.
- Ask a targeted question only when the answer would materially change the viable options or
  recommendation. Otherwise, state the working assumption and continue.

## Produce the Decision Record

Use only the sections relevant to the decision:

- **Established facts**: current repository or externally verified evidence.
- **Invariants**: constraints owned by existing project contracts.
- **Options**: credible choices and their trade-offs.
- **Recommendation**: the best current choice and why.
- **Unresolved decisions**: choices that require the user's authority or missing evidence.
- **Measurements**: focused probes needed before deciding or implementing.

Stop when the user can make the decision from the reported evidence. Do not prolong the brainstorm
to exhaust hypothetical edge cases. After agreement, summarize the selected option, rejected
alternatives, remaining uncertainty, and the smallest coherent implementation step; do not execute
that step until explicitly requested.
