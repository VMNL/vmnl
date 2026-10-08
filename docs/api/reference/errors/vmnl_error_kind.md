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
| GLFW | `GlfwInitFailed`, `GlfwWindowCreationFailed`, `GlfwContextCreationFailed`, `GlfwUnsupportedPlatform`, `GlfwVersionMismatch`, `GlfwPlatformError`, `GlfwUnknownError` |
| Client/state | `InvalidWindowSize`, `InvalidState(String)` |
| Device requirements | `DeviceRequirementsNotMet { required_features: Vec<DeviceFeature> }` |
| Pipeline rasterization | `DeviceFeatureNotEnabled { feature: DeviceFeature }`, `InvalidLineWidth { value: f32, min: f32, max: f32 }` |

Only `Debug` is derived. Human-readable text is provided through `VMNLError`'s `Display` implementation.

## Construction, defaults, and validation

Variants are constructed directly. `InvalidState` owns application-specific detail. `DeviceRequirementsNotMet` carries all requested features in first-request order when no baseline-compatible GPU supports their combination; individual features may still be supported separately on different devices. `DeviceFeatureNotEnabled` identifies a feature not activated on the pipeline's device. `InvalidLineWidth` retains the requested value (including NaN/infinity) and the device range. These are build-time errors before shader compilation. There is no default and no validator.

## Units, coordinates, and valid ranges

Line widths and their range are in framebuffer units. Valid pipeline requests must be finite, strictly positive and within the inclusive reported range.

## Ownership, lifecycle, and threading

All unit variants own no resources; `InvalidState` owns a `String` and `DeviceRequirementsNotMet` owns a feature vector.

## Errors, panics, and failure conditions

Constructing a variant is infallible. The enum represents failures rather than causing them.

## Allocation, transfers, synchronization, and GPU cost

`InvalidState(String)` and `DeviceRequirementsNotMet` may own heap allocations. No GPU work occurs.

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
