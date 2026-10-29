# `VMNLErrorKind`

## Public path and maturity

Import path: `vmnl::VMNLErrorKind`. Status: experimental and `#[non_exhaustive]`.

## Purpose and use cases

Classifies initialization, windowing, Vulkan, validation, and client-state failures. External matches require a wildcard arm.

## Public API

| Group | Variants |
|---|---|
| Vulkan creation | `VulkanInitFailed`, `VulkanSurfaceCreationFailed`, `VulkanSwapchainCreationFailed`, `VulkanShaderModuleCreationFailed`, `VulkanPipelineCreationFailed`, `VulkanVertexBufferCreationFailed`, `VulkanIndexBufferCreationFailed`, `VulkanFrameUboBufferCreationFailed`, `VulkanMemoryAllocationFailed`, `VulkanCommandBufferCreationFailed`, `VulkanDescriptorSetCreationFailed`, `VulkanSemaphoreCreationFailed`, `VulkanFenceCreationFailed`, `VulkanFramebufferCreationFailed`, `VulkanRenderPassCreationFailed`, `VulkanImageCreationFailed`, `VulkanImageViewCreationFailed`, `VulkanSamplerCreationFailed`, `VulkanDescriptorPoolCreationFailed`, `VulkanDescriptorSetLayoutCreationFailed`, `VulkanPipelineLayoutCreationFailed`, `VulkanShaderCompilationFailed` |
| Vulkan runtime/status | `VulkanValidationFailed`, `VulkanUnsupportedFeature`, `VulkanOutOfMemory`, `VulkanOutOfDate`, `VulkanDeviceLost`, `VulkanSurfaceLost`, `VulkanExtensionNotPresent`, `VulkanLayerNotPresent`, `VulkanIncompatibleDriver`, `VulkanTooManyObjects`, `VulkanFormatNotSupported`, `VulkanFragmentation`, `VulkanUnknownError` |
| GLFW | `GlfwInitFailed`, `GlfwInitializationConfigConflict { active_hat_buttons, requested_hat_buttons }`, `GlfwInputOperationFailed { operation, message }`, `GlfwWindowCreationFailed`, `GlfwContextCreationFailed`, `GlfwUnsupportedPlatform`, `GlfwVersionMismatch`, `GlfwPlatformError`, `GlfwUnknownError` |
| Client/state | `InvalidWindowSize`, `InvalidGamepadMapping(String)`, `InvalidState(String)` |

Only `Debug` is derived. Human-readable text is provided through `VMNLError`'s `Display` implementation.

## Construction, defaults, and validation

Variants are constructed directly. `GlfwInputOperationFailed` owns the operation's backend detail;
`InvalidGamepadMapping` owns the rejected input reason; `InvalidState` owns application-specific
detail. There is no default and no validator.

## Units, coordinates, and valid ranges

Not applicable.

## Ownership, lifecycle, and threading

Unit variants own no resources. Message-bearing variants own their strings; the initialization
conflict carries two booleans and owns no heap data.

## Errors, panics, and failure conditions

Constructing a variant is infallible. The enum represents failures rather than causing them.

## Allocation, transfers, synchronization, and GPU cost

Message-bearing variants may own heap allocations. The initialization conflict carries two booleans
and owns no heap data. No GPU work occurs.

## Platform, Vulkan, and display constraints

Variant availability does not imply that every backend emits every category. Mapping of third-party errors is intentionally coarser than native Vulkan/GLFW error payloads.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::VMNLErrorKind;

fn retryable(kind: &VMNLErrorKind) -> bool {
    matches!(kind, VMNLErrorKind::VulkanOutOfDate)
}
assert!(retryable(&VMNLErrorKind::VulkanOutOfDate));
```

Related: [`VMNLError`](vmnl_error.md) and the [errors matrix](../../appendices/errors_matrix.md).
