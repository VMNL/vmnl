# Errors and recovery

Fallible operations return `VMNLResult<T>`, whose error stores a `VMNLErrorKind` plus the call-site location captured by `VMNLError::new`. Match error kinds conservatively: `VMNLErrorKind` is non-exhaustive.

Recoverable runtime conditions include swapchain out-of-date/surface changes and invalid client input. Device-lost, incompatible-driver, and initialization failures normally require rebuilding higher-level state or terminating the graphical path. Exact recovery is backend- and application-dependent unless a reference page specifies it.

VMNL documents no public method as intentionally panicking. Rust allocation failures, poisoned synchronization primitives, platform/library bugs, and violated unsafe trait contracts remain outside the typed error model.

See the [errors matrix](../appendices/errors_matrix.md).
