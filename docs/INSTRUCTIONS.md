# General Instructions

- Write clear code, as simple as possible, with good maintainability
- Cognitive complexity should be as low as possible
- Code must be self-explanatory through variable and method names reflecting domain concepts

## Coding Standards

- `rustfmt` and `clippy -D warnings` are mandatory.
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
