# `VMNLErrorLocation`

## Public path and maturity

Import path: `vmnl::VMNLErrorLocation`. Status: experimental, operational value type.

## Purpose and use cases

Reports the file, line, and column captured when `VMNLError::new` was called.

## Public API

`file() -> &'static str`, `line() -> u32`, and `column() -> u32`; the type derives `Debug`, `Clone`, and `Copy`. Fields are private.

## Construction, defaults, and validation

Clients receive it from `VMNLError::location`; direct construction and `Default` are unavailable.

## Units, coordinates, and valid ranges

`line` and `column` are source coordinates reported by Rust's caller location. `file` is compiler-provided and may be relative or absolute depending on the build.

## Ownership, lifecycle, and threading

The value is copied and contains a static string reference; it owns no external resource.

## Errors, panics, and failure conditions

Accessors are infallible and do not intentionally panic.

## Allocation, transfers, synchronization, and GPU cost

No allocation, transfer, synchronization, or GPU work.

## Platform, Vulkan, and display constraints

None. Paths may differ between build environments.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{VMNLError, VMNLErrorKind};

let location = VMNLError::new(VMNLErrorKind::InvalidWindowSize).location();
assert!(!location.file().is_empty());
assert!(location.line() > 0);
```

Related: [`VMNLError`](vmnl_error.md).

