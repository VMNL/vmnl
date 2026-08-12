# `PresentMode`

## Public path and maturity

Import path: `vmnl::PresentMode`. Status: experimental.

## Purpose and use cases

Selects the swapchain presentation policy used at window creation.

## Public API

Variants: `Auto` (default), `Fifo`, `Mailbox`, `Immediate`, and `FifoRelaxed`. Derives: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, and `Default`.

## Construction, defaults, and validation

Automatic priority is `Mailbox`, `Immediate`, `FifoRelaxed`, then `Fifo`, choosing the first surface-supported mode. `present_mode` is strict for non-`Auto` modes. `preferred_present_mode` falls back to the automatic order and logs a warning.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Plain copied configuration selected during swapchain construction.

## Errors, panics, and failure conditions

Strict selection returns `VulkanUnsupportedFeature` when unsupported. Automatic selection fails with the same kind if no candidate is reported.

## Allocation, transfers, synchronization, and GPU cost

The enum itself has no cost. Presentation mode affects pacing, blocking, latency, tearing, and image availability, but VMNL specifies no quantitative performance guarantee.

## Platform, Vulkan, and display constraints

`Fifo` is the portable v-sync choice. Other modes are surface/driver dependent. `Immediate` may tear; `Mailbox` may discard queued frames.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::PresentMode;

assert_eq!(PresentMode::default(), PresentMode::Auto);
```

Related: [`WindowBuilder`](window_builder.md).

