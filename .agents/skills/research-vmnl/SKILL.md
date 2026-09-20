---
name: research-vmnl
description: Research external technical facts that could affect VMNL architecture, APIs, FFI, platform behavior, dependencies, compatibility, or performance using primary sources. Report cited findings without changing the repository. Do not use for debugging an observed failure, reviewing a diff, implementing a change, or ordinary repository inspection.
---

# Research VMNL

## Outcome

Answer a scoped VMNL technical question with traceable evidence. Separate verified facts,
version-dependent applicability, inferences, contradictions, and unresolved unknowns so the result
can support a later brainstorm, specification, or implementation decision.

Research is read-only. Report in the conversation by default. Do not edit code, documentation,
configuration, issues, pull requests, submodules, or other external state unless the user explicitly
requests a separate written artifact or mutation.

## Establish the Question

1. State the exact question and the VMNL decision or contract it could affect.
2. Inspect the current repository sources needed to establish VMNL's existing intent and behavior.
3. Record the relevant versions, revision, platform, backend, target, or date before applying an
   external source to the project.
4. Identify what evidence would answer the question and stop unrelated exploration.

Follow [`AGENTS.md`](../../../AGENTS.md) and its domain routing. For architecture, public API,
module, or roadmap implications, read [`docs/architecture.md`](../../../docs/architecture.md).
Canonical VMNL documentation defines intended project contracts; tests and implementation establish
current behavior only.

## Source Priority

Prefer the source that owns each claim:

1. standards, specifications, RFCs, official manuals, and official API documentation;
2. authoritative upstream headers, source code, release notes, and compatibility policy;
3. VMNL canonical documentation, public Rustdoc, tests, build metadata, and implementation;
4. maintainer issues or discussions when the primary contract is incomplete;
5. secondary articles only to locate a primary source or identify terminology.

For the planned C network backend, inspect only the pinned revision's exported headers, ABI
documentation, tests, build metadata, and implementation. Treat the backend as read-only and do not
advance its revision.

Do not promote an implementation detail, issue comment, search snippet, or secondary explanation
into a normative guarantee. When authoritative sources disagree, report the contradiction, affected
versions, and contract owner instead of silently choosing one.

## Evidence Discipline

- Cite each material external claim near the statement it supports.
- Link directly to the owning document, specification section, source revision, or official release
  note rather than a search result.
- Distinguish a quoted or documented fact from an inference drawn by applying it to VMNL.
- State when evidence is version-, target-, driver-, compiler-, platform-, or backend-specific.
- Verify current information when it can plausibly have changed; record the checked version or date.
- Use repository paths and line locations for local claims when they materially support the result.
- Do not claim runtime, portability, ABI, safety, or performance behavior from compilation or static
  inspection alone. Name the probe or measurement still required.

## Report

Use only the sections needed:

- **Findings**: direct answers supported by primary evidence.
- **Applicability to VMNL**: which current contract, layer, or decision the findings affect.
- **Conflicts and unknowns**: unresolved source contradictions or missing evidence.
- **Required verification**: the smallest experiment, build, ABI probe, or measurement still needed.

Keep citations attached to their claims. Stop when the question is answered to the requested depth
and remaining uncertainty is explicit.

If the user requests a Markdown artifact, first inspect the repository's documentation ownership and
write only to the requested or appropriate non-canonical location. Research does not become a stable
VMNL contract until the user approves that separate documentation change.
