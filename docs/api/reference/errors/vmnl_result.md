# `VMNLResult`

## Public path and maturity

Import path: `vmnl::VMNLResult<T>`. Status: experimental alias.

## Purpose and use cases

Uniform return type for fallible VMNL operations.

## Public API

Equivalent to `Result<T, VMNLError>` and therefore exposes the standard `Result` API; no VMNL-specific method is added.

## Construction, defaults, and validation

Construct with `Ok(value)` or `Err(VMNLError::new(kind))`.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Ownership and auto traits follow `T` and `VMNLError`.

## Errors, panics, and failure conditions

The alias represents typed failure. Standard operations such as `unwrap` can panic according to the standard library contract.

## Allocation, transfers, synchronization, and GPU cost

The alias itself adds no cost.

## Platform, Vulkan, and display constraints

None beyond the operation returning it.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{VMNLError, VMNLErrorKind, VMNLResult};

fn reject() -> VMNLResult<()> {
    Err(VMNLError::new(VMNLErrorKind::InvalidState("rejected".into())))
}
assert!(reject().is_err());
```

Related: [`VMNLError`](vmnl_error.md) and [`VMNLErrorKind`](vmnl_error_kind.md).

