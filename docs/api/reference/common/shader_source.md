# `ShaderSource`

## Public path and maturity

Import paths: `vmnl::ShaderSource`, `vmnl::common::ShaderSource`, and `vmnl::raw::ShaderSource`. Status: experimental.

## Purpose and use cases

Carries inline GLSL source or a path to a GLSL file for window-default and raw pipelines.

## Public API

Variants: `Src(String)` and `Path(PathBuf)`. Derives: `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, and `Ord`.

## Construction, defaults, and validation

There is no default. Source reading and shader compilation occur when a window or pipeline is built, not when the enum is constructed. VMNL expects a `main` entry point.

## Units, coordinates, and valid ranges

Text encoding follows Rust `String` UTF-8. Path interpretation follows the process filesystem and working-directory rules.

## Ownership, lifecycle, and threading

Both variants own their data. Pipelines retain compiled Vulkan objects rather than requiring the source value to remain alive.

## Errors, panics, and failure conditions

Build may fail for unreadable paths, invalid GLSL, missing `main`, stage/interface mismatch, unsupported resources, or Vulkan module/pipeline creation.

## Allocation, transfers, synchronization, and GPU cost

`Src` and `Path` own heap-backed values. Build may read the filesystem, invoke shaderc, allocate SPIR-V, and create Vulkan objects. Cost guarantees are not specified.

## Platform, Vulkan, and display constraints

Filesystem paths must be available at build time. GLSL support is constrained by shaderc, VMNL's pipeline contract, and the selected Vulkan device.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::ShaderSource;

let source = ShaderSource::Src("#version 450\nvoid main() {}".into());
assert!(matches!(source, ShaderSource::Src(_)));
```

Related: [`WindowBuilder`](../window/window_builder.md) and [`PipelineSpec`](../raw/pipeline/pipeline_spec.md).
