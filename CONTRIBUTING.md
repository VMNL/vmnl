# Contributing

Follow the project rules in [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md).

## Before Submission

Run every applicable automatic check in this order:

```bash
just build-workspace
just check-fmt
just check-clippy
just doctest
just docs
just docs-api-check
just test-unit
just test-api
just test-smoke
just test-platform         # GLFW error conversion and Null backend
just test-platform-compile # every OS for GLFW-facing changes
just test-platform-wayland # when a qualified Wayland compositor is available
just test-platform-x11     # when a qualified X11 + EWMH environment is available
just test-gpu-compile # GPU-facing changes
just test-gpu         # when the environment supports it
```

Do not reorder applicable completion checks without documenting the technical reason. Run `just test-gpu` only on a machine with a Vulkan-capable GPU and a display server. `just test` is the combined headless suite, not an additional suite.

- Public API changes update Rustdoc and API tests.
- Public API changes follow the [API change protocol](docs/api/maintenance/api_change_protocol.md),
  update the coverage matrix, then run and review `just docs-api-update` before the non-mutating
  `just docs-api-check`.
- Bug fixes include a regression test.
- New or changed GLFW calls follow the
  [GLFW portability protocol](docs/api/maintenance/glfw_portability_protocol.md), update the
  canonical TOML inventory, and add a backend test or explicit justification.
- Visual workflows belong in `examples/`; headless checks belong in `tests/`.
- Every feature or fix must assess Rustdoc, technical and user documentation, examples, `CHANGELOG.md`, and documentation navigation. Do not create artificial documentation changes; explain in the final report when no update is required.

## Manual Graphics Validation

Automatic validation does not replace graphical validation performed by the human operator.

For a feature affecting rendering or observable graphics behavior, the operator must:

- execute a new manual graphical scenario specific to the feature;
- check relevant existing graphical examples or scenarios for visible regressions.

For a fix affecting rendering or observable graphics behavior, the operator must:

- reproduce or manually check the defective scenario;
- verify that the defect is no longer observable after the fix;
- check relevant existing graphical examples or scenarios for visible regressions.

Never invent or assume a manual validation. A graphical feature or fix is not fully validated until the operator explicitly reports the required checks and their observations.

## Pull Request Descriptions

Every VMNL pull request description uses exactly this structure:

```markdown
## Context

<Context and reason for the pull request>

## Changes

- <Relevant change>
- <Tests added or updated>
- <Documentation added or updated>

## Validation

- <Manual graphical validation performed by the operator>
```

- `Context` explains the need, reason for the change, and expected result; it is never empty.
- `Changes` lists all relevant behavior, API, architecture, test, example, and documentation changes without describing unchanged behavior.
- `Validation` contains only manual graphical validations explicitly performed and reported by the human operator.
- Do not repeat compilation, formatting, Clippy, doctests, documentation builds, unit, API/functional, smoke, GPU, or other automatic results in `Validation`; those results belong to the PR checks or CI.
- When required manual graphical evidence is missing, leave `Validation` empty and request the operator's results instead of inventing or assuming them.

## Commit Messages

Format:

```text
<type>[optional scope][!]: <description>
```

- Use imperative mood, start the description lowercase, keep the subject at most 72 characters,
  and do not add a trailing period.
- Use an optional body after a blank line; each line stays under 80 characters.
- Do not use gitmoji or legacy project scopes.
- Add `BREAKING CHANGE:` in the body when a public contract is incompatible.
- For more information, see [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

Allowed types are defined in [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md#commit-type-usage-guidelines).

Install the repository-owned `commit-msg` hook once per clone:

```bash
just hooks-install
```

The hook provides immediate local feedback. CI validates every commit added by a pull request;
configure the GitHub `Commit messages` check as required on `main` to prevent a failing pull
request from being merged.
