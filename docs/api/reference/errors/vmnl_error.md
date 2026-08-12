# `VMNLError`

## Public path and maturity

Import path: `vmnl::VMNLError`. Status: experimental, operational error type.

## Purpose and use cases

Carries a VMNL error category and the source location at which VMNL constructed the error. It implements `Debug`, `Display`, and `std::error::Error`.

## Public API

| Method | Contract |
|---|---|
| `new(kind)` | Construct an error and capture `Location::caller()` (`#[track_caller]`). |
| `kind()` | Borrow the `VMNLErrorKind`. |
| `location()` | Copy the `VMNLErrorLocation`. |
| `report()` | Allocate a string containing the display text and `file:line:column`. |

## Construction, defaults, and validation

There is no default. `new` accepts every error-kind value and performs no validation.

## Units, coordinates, and valid ranges

Line and column values are one-based compiler source coordinates when available.

## Ownership, lifecycle, and threading

The error owns its kind, including any `String` in `InvalidState`, and copies its static location metadata. No external resource is retained.

## Errors, panics, and failure conditions

Creating and formatting an error has no typed failure path. `report` allocates; allocation failure follows Rust's process-level behavior.

## Allocation, transfers, synchronization, and GPU cost

`new` normally performs no allocation except ownership already present in the kind. `report` allocates one `String`. No GPU or synchronization work occurs.

## Platform, Vulkan, and display constraints

None; variants may describe platform/GPU failures without accessing those systems.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::{VMNLError, VMNLErrorKind};

let error = VMNLError::new(VMNLErrorKind::InvalidWindowSize);
assert!(error.report().contains("invalid window size"));
```

Related: [`VMNLErrorKind`](vmnl_error_kind.md), [`VMNLErrorLocation`](vmnl_error_location.md), and [`VMNLResult`](vmnl_result.md).

