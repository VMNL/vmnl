---
name: plan-vmnl-issues
description: Turn an approved VMNL plan, specification, or conversation into a reviewed graph of independently verifiable issues. Draft before any external mutation and publish only with separate authorization. Do not use for brainstorming unresolved design, implementation, single-issue editing, or issue triage.
---

# Plan VMNL Issues

## Outcome

Produce an approved, acyclic issue graph whose nodes are coherent VMNL deliverables and whose edges
represent real blocking dependencies. Each issue must state an observable result, its contract, and
the evidence required to accept it.

Planning is read-only. Invoking this skill does not authorize creating or editing GitHub issues,
labels, relationships, milestones, projects, or other external state. Publishing is a separate phase
that requires explicit authorization after the user approves the complete draft.

## Establish the Source Contract

1. Identify the approved plan, specification, issue, or conversation to decompose.
2. Read [`AGENTS.md`](../../../AGENTS.md), the relevant canonical documentation, and only enough
   implementation and tests to establish the current boundary.
3. For a feature, public API, architecture, module, or roadmap change, read
   [`docs/architecture.md`](../../../docs/architecture.md).
4. Inspect current issues when needed to avoid duplicates and preserve existing ownership or
   dependencies. If an issue or pull request is supplied, read its body and relevant comments.
5. Separate established decisions from unresolved choices. Do not disguise an unresolved product or
   architecture decision as an implementation issue.

If the source contract is incomplete, report the exact missing decision. Create a decision issue only
when the user wants that uncertainty tracked as its own deliverable; otherwise stop before drafting
dependent implementation work.

## Slice the Work

Prefer independently deliverable vertical slices. Each issue should produce one observable behavior,
contract, qualification, or migration step that can be reviewed and accepted without relying on
uncommitted work from another issue.

- Include the applicable public API, implementation, deterministic tests, documentation, examples,
  platform evidence, and release impact in the same issue when they form one delivery contract.
- Keep separate features or fixes independent when either can ship without the other.
- Extract a shared contract or substrate only when it is itself verifiable and genuinely blocks two
  or more consumers; connect the consumers with explicit blocking edges.
- Use expand, migrate, and contract issues for a wide mechanical change only when an independently
  green vertical slice is impractical.
- Do not create speculative prefactoring, abstraction, manager, or framework issues without a
  current use case.
- Distinguish work owned by VMNL from work owned by another repository. The C network backend is
  read-only to VMNL agents; do not plan edits or issue mutations in that repository without separate
  authority.

Size an issue as the smallest coherent change that can be implemented, validated, reviewed, and
merged independently. Do not split solely by source layer, file, or implementation phase.

## Build the Dependency Graph

For every issue, list only hard blockers: work whose delivered contract is required before this issue
can start or be validated. A preferred order, shared topic, or opportunity for parallel coordination
is not a blocking edge.

The graph must be acyclic. Identify its initial frontier: every issue with no unresolved blocker. When
two issues share a contract but remain independently deliverable, reference the common contract and
coordinate them without inventing a dependency.

For submodules and cross-repository work, distinguish:

- backend behavior and revision owned by the backend repository;
- version pin, build integration, safe wrapper, facade, and validation owned by VMNL.

## Draft Each Issue

Use this information, omitting fields that do not apply:

```markdown
## Objective

<observable capability, fix, decision, or qualification delivered>

## Contract

- <behavior, ownership, error, lifecycle, or compatibility requirement>

## Acceptance criteria

- [ ] <independently verifiable criterion>

## Validation

- <required automated, native, GPU, FFI, remote-system, or operator evidence>

## Blocked by

- <issue reference or None>
```

For the planning summary, also propose:

- a concise title;
- the currently enabled native GitHub Issue Type representing the issue's nature;
- only current labels that describe area, layer, impact, status, or resolution;
- documentation and release-note impact when applicable.

Native Issue Types are the source of truth for issue nature. Do not duplicate them with `type:*`
labels. Inspect current Issue Types, labels, existing issues, and repository permissions immediately
before any publication because that state can change.

Avoid implementation prescriptions that are not part of the approved contract. Mention a concrete
path, symbol, or type only when it owns the relevant current behavior or makes an acceptance
criterion materially less ambiguous.

## Obtain Approval

Present the proposed issues in dependency order. For each issue, show its title, Issue Type,
observable delivery, blockers, acceptance boundary, and proposed labels. Then show the initial
frontier and any work that can proceed in parallel.

Do not publish until the user has approved:

- issue granularity and scope;
- blocking edges and ownership;
- acceptance evidence;
- Issue Types and labels;
- the explicit act of creating or modifying issues.

An approval of the plan is not authorization to publish it unless the user explicitly combines both
instructions.

## Publish and Verify

When publication is explicitly authorized:

1. Re-fetch current repository identity, permissions, open issues, native Issue Types, labels, and
   supported dependency relationships.
2. Report duplicates, metadata drift, or unavailable relationships before mutating state when they
   would change the approved plan.
3. Create issues in dependency order so blocker references resolve. Do not modify an existing parent
   issue unless that mutation was included in the authorization.
4. Apply only approved Issue Types, labels, bodies, and relationships.
5. Re-read every created or modified issue and verify its title, body, type, labels, state, and
   dependency edges against the approved draft.
6. Report identifiers, links, successful verification, failures, and any relationship that could not
   be represented natively.

Stop on partial failure. Preserve successfully created issues, report the exact resulting state, and
obtain direction before retrying, editing, closing, or deleting anything.
