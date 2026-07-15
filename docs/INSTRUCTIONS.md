# General Instructions

- Write clear code, as simple as possible, with good maintainability
- Cognitive complexity should be as low as possible
- Code must be self-explanatory through variable and method names reflecting domain concepts

## Coding Standards

- `rustfmt` and `clippy -D warnings` are mandatory.
- Every maintained Rust file under `crates/`, `examples/`, and `tests/` must begin with the following SPDX header:

  ```rust
  // SPDX-FileCopyrightText: 2026 VMNL
  // SPDX-License-Identifier: MIT
  ```

- Do not modify generated or third-party files solely to add SPDX metadata.
- Use a module-level `//!` Rustdoc comment when a public module boundary needs context beyond its item-level Rustdoc; this summary is separate from the SPDX metadata.
- Every `unsafe` block must include a `// SAFETY:` comment that states the verifiable invariant.
- Do not use `unwrap` or `expect` in library crates unless the documented invariant makes failure impossible.
- Prefer types, `Result`, and dedicated error types over ambiguous error strings.
- A function must have one observable responsibility. Extract a function only when it clarifies an invariant or removes meaningful duplication.

## Naming Conventions

- Crates and modules use `snake_case`.
- Types and traits use `PascalCase`.
- Functions, variables, and fields use `snake_case`.
- Constants and statics use `SCREAMING_SNAKE_CASE`.
- Boolean values use the `is_`, `has_`, `can_`, or `should_` prefix.
- Conversions use `from_*`, `to_*`, `as_*`, or `into_*` according to Rust conventions.
- Avoid opaque abbreviations. Keep established Rust and Vulkan terminology such as `Vk`, `SPIR-V`, and `ID`.

## Documentation

- Avoid unnecessary code comments or obvious comments
- Avoid comments explaining what the code does. Explain WHY when required
- Add comments that explain why a design decision was made.
- Only complex algorithms should have explanatory comments

## Code organization and structure

- Maintain alphabetical order when adding new entries
- Keep related methods adjacent to each other
- Group code by purpose using blank lines to separate logical blocks
- Add blank lines after guard statements to separate validation from business logic

## Resource Management and Localization

- Check for existing resource tags before creating new ones
- Use generic resource as much as possible

## Commit Messages

Format: `<type>: <description>` — imperative mood, lowercase, under 72 characters, no trailing period.

Optional body: blank line after subject, then a dash list of the most important changes, each under 80 characters.

For more information, see [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

## Commit Type usage guidelines

| Type | Use when |
|------|----------|
| `build` | build system or external dependency changes |
| `cicd` | CI/CD pipeline or script changes |
| `chore` | no-impact housekeeping (e.g. .gitignore, tooling config) |
| `docs` | documentation only |
| `feat` | new feature |
| `fix` | bug fix |
| `perf` | performance improvement |
| `refactor` | restructuring without behaviour change |
| `style` | formatting, whitespace, missing semicolons |
| `test` | adding or correcting tests |

## Vulkan Rules

- Never hide expensive GPU operations behind simple-looking APIs.
- Avoid implicit GPU synchronization.
- Track resource ownership explicitly.
- Do not recreate Vulkan resources unnecessarily.
- Prefer resource reuse and caching.
- Avoid GPU stalls.
- Keep Vulkan lifetime management explicit.