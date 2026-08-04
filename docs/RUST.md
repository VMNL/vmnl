# Rust Guidelines

## Best Practices

### Code Quality

- Write idiomatic Rust
- Prefer simple, readable, and maintainable code
- Keep cognitive complexity as low as possible
- Functions should have one clear responsibility
- Use meaningful names reflecting domain concepts
- Prefer explicit code over hidden behavior
- Avoid unnecessary abstractions
- Favor composition over inheritance-style designs

---

## Ownership and Borrowing

- Prefer borrowing over unnecessary ownership transfers
- Prefer:
  - `&T`
  - `&mut T`
  - slices (`&[T]`)
  - iterators

- Avoid unnecessary:
  - `clone()`
  - allocations
  - copies
  - ownership moves

- Make ownership and lifetime rules explicit
- Resource cleanup must be deterministic

---

## Memory Management

- Avoid heap allocations in performance-sensitive code
- Reuse memory whenever possible
- Prefer:
  - buffer reuse
  - pre-allocation
  - object pools
  - arenas when appropriate

- Do not allocate inside:
  - render loops
  - audio callbacks
---

## Performance

- Prefer zero-cost abstractions
- Avoid unnecessary runtime overhead
- Prefer compile-time checks when possible
- Avoid dynamic dispatch unless runtime polymorphism is required
- Keep data layouts cache-friendly
- Minimize unnecessary copies

Do not infer a performance improvement from build or lint commands. Define a workload and metric, then measure before and after under the same build profile, machine, GPU/driver, and rendering configuration.

For non-mutating code validation:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## Unsafe Rust

- Minimize the use of `unsafe`
- Unsafe code must be isolated
- Every `unsafe` block requires a safety comment

Example:

```rust
// SAFETY:
// The buffer is valid because it is created and owned by the allocator
unsafe {
    operation();
}
```

The comment must explain:
- the invariant being relied on
- why the operation is safe
- what assumptions must remain true

---

## Traits and Generics

- Create traits only when they provide real value
- Avoid traits with a single implementation unless abstraction is intentional
- Prefer generics when:
  - performance matters
  - the type is known at compile time

Avoid unnecessary generic complexity

---

## Module Organization

- Keep modules focused
- Expose the smallest possible public API
- Prefer private visibility by default

Use:

```rust
pub(crate)
```

instead of:

```rust
pub
```

when external access is not required

Avoid:
- circular dependencies
- large files with unrelated responsibilities
- leaking internal implementation details

---

## Validation

Before submitting changes:

```bash
cargo build --workspace --all-targets
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
./run -d
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
./run -ut
./run -at
./run -st
cargo test -p vmnl-gpu-tests --no-run # GPU-facing changes
./run -gt                              # when the environment supports it
```

Keep this order for every applicable completion check. Document the technical reason for any required deviation.

All warnings must be resolved

Do not disable Clippy warnings unless there is a documented reason

---

# Error Management

## General Rules

- Library code must not panic
- Use `Result<T, E>` for recoverable failures
- Use dedicated error types
- Avoid string-based errors

Prefer:

```rust
enum Error {
    InvalidResource,
    DeviceLost,
    AllocationFailed,
}
```

Avoid:

```rust
return Err("operation failed");
```

---

## unwrap / expect Rules

Forbidden by default:

```rust
unwrap()
unwrap_*()
expect()
panic!()
```

The use of **forbidden functions** is strictly limited to **unit tests only**.

Please make sure to:
- Use them only in the appropriate test files.
- Avoid using them in production code or regular application logic.
- Review your changes carefully before submitting to ensure no forbidden functions are introduced outside of unit tests.

## Error Propagation

- Never silently ignore errors
- Preserve original errors when possible
- Add context when propagating errors
- Error messages must explain:
  - what failed
  - why it failed
  - possible resolution

---

## Error Design

Errors should:

- be actionable
- represent real failure cases
- avoid exposing unnecessary internal details

Public errors are part of the API design

---

# Forbidden Use

## Forbidden Rust Patterns

Do not use:

- `unwrap()` without documented invariant
- `expect()` without documented invariant
- `panic!()` inside library code
- unnecessary `unsafe`
- global mutable state
- hidden allocations
- unnecessary cloning
- dead code
- ignored warnings

---

## Forbidden Architecture Patterns

Avoid:

- abstractions without a concrete use case
- premature optimization
- unnecessary design patterns
- unnecessary managers or factories
- exposing internal implementation details
- breaking public APIs without justification

---

## Forbidden Performance Patterns

Avoid inside hot paths:

- heap allocations
- blocking operations
- filesystem access
- expensive formatting
- unnecessary locks

Avoid:

```rust
Arc<Mutex<T>>
```

unless synchronization is explicitly required

---

## Forbidden Dependency Usage

Before adding a dependency:

Check:

- maintenance status
- license compatibility
- compile impact
- binary size impact
- whether the standard library or existing code is sufficient

Prefer fewer, well-maintained dependencies

---

## Forbidden Style

Do not:

- add comments explaining obvious code
- use unclear abbreviations
- create overly generic names
- duplicate existing functionality
- ignore existing project conventions

Comments should explain:

- why something exists
- why a non-obvious choice was made
- important invariants
