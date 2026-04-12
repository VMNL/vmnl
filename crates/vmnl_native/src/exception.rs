////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Exception handling for the VMNL library, defining custom error types and result aliases.
////////////////////////////////////////////////////////////////////////////////

use std::{
    error::Error,
    fmt,
    panic::Location,
};

/**
 * * Represents the location in the source code where a VMNL error occurred.
 */
#[derive(Debug, Clone, Copy)]
pub struct VMNLErrorLocation
{
    /// * The file in which the error occurred.
    file:   &'static str,
    /// * The line number at which the error occurred.
    line:   u32,
    /// * The column number at which the error occurred.
    column: u32,
}

impl VMNLErrorLocation
{
    /**
     * * Creates a new VMNLErrorLocation instance with the specified file, line, and column information.
     *
     * ! Returns:
     * - A new instance of VMNLErrorLocation containing the provided file, line, and column information.
     */
    pub fn file(&self) -> &'static str { self.file }

    /**
     * * Retrieves the line number where the error occurred.
     *
     * ! Returns:
     * - The line number associated with the error location.
     */
    pub fn line(&self) -> u32 { self.line }

    /**
     * * Retrieves the column number where the error occurred.
     *
     * ! Returns:
     * - The column number associated with the error location.
     */
    pub fn column(&self) -> u32 { self.column }
}

/**
 * * Enum representing various kinds of errors that can occur within the VMNL library.
 */
#[derive(Debug)]
#[non_exhaustive]
pub enum VMNLErrorKind
{
    /// * Vulkan-related errors, covering a wide range of potential issues that can arise during Vulkan initialization, resource creation, and operation.
    VulkanInitFailed,
    /// * Vulkan surface creation failed, indicating an issue with creating a rendering surface for Vulkan.
    VulkanSurfaceCreationFailed,
    /// * Vulkan swapchain creation failed, indicating an issue with creating the swapchain for presenting rendered images.
    VulkanSwapchainCreationFailed,
    /// * Vulkan shader module creation failed, indicating an issue with creating shader modules for rendering.
    VulkanShaderModuleCreationFailed,
    /// * Vulkan pipeline creation failed, indicating an issue with creating rendering pipelines.
    VulkanPipelineCreationFailed,
    /// * Vulkan buffer creation failed, indicating an issue with creating GPU buffers.
    VulkanBufferCreationFailed,
    /// * Vulkan memory allocation failed, indicating an issue with allocating GPU memory.
    VulkanMemoryAllocationFailed,
    /// * Vulkan command buffer creation failed, indicating an issue with creating command buffers for GPU execution.
    VulkanCommandBufferCreationFailed,
    /// * Vulkan descriptor set creation failed, indicating an issue with creating descriptor sets for binding resources.
    VulkanDescriptorSetCreationFailed,
    /// * Vulkan semaphore creation failed, indicating an issue with creating semaphores for synchronization.
    VulkanSemaphoreCreationFailed,
    /// * Vulkan fence creation failed, indicating an issue with creating fences for synchronization.
    VulkanFenceCreationFailed,
    /// * Vulkan framebuffer creation failed, indicating an issue with creating framebuffers for rendering.
    VulkanFramebufferCreationFailed,
    /// * Vulkan render pass creation failed, indicating an issue with creating render passes for rendering.
    VulkanRenderPassCreationFailed,
    /// * Vulkan image creation failed, indicating an issue with creating images for rendering.
    VulkanImageCreationFailed,
    /// * Vulkan image view creation failed, indicating an issue with creating image views for sampling.
    VulkanImageViewCreationFailed,
    /// * Vulkan sampler creation failed, indicating an issue with creating samplers for texture sampling.
    VulkanSamplerCreationFailed,
    /// * Vulkan descriptor pool creation failed, indicating an issue with creating descriptor pools for managing descriptor sets.
    VulkanDescriptorPoolCreationFailed,
    /// * Vulkan descriptor set layout creation failed, indicating an issue with creating descriptor set layouts for defining resource bindings.
    VulkanDescriptorSetLayoutCreationFailed,
    /// * Vulkan pipeline layout creation failed, indicating an issue with creating pipeline layouts for defining pipeline resource interfaces.
    VulkanPipelineLayoutCreationFailed,
    /// * Vulkan shader compilation failed, indicating an issue with compiling shaders for rendering.
    VulkanShaderCompilationFailed,
    /// * Vulkan validation failed, indicating an issue with validating Vulkan API usage.
    VulkanValidationFailed,
    /// * Vulkan unsupported feature, indicating an issue with using a Vulkan feature that is not supported by the current environment.
    VulkanUnsupportedFeature,
    /// * Vulkan out of memory, indicating an issue with allocating GPU memory.
    VulkanOutOfMemory,
    /// * Vulkan device lost, indicating an issue with the Vulkan device being lost.
    VulkanDeviceLost,
    /// * Vulkan surface lost, indicating an issue with the Vulkan surface being lost.
    VulkanSurfaceLost,
    /// * Vulkan extension not present, indicating an issue with a required Vulkan extension not being present.
    VulkanExtensionNotPresent,
    /// * Vulkan layer not present, indicating an issue with a required Vulkan layer not being present.
    VulkanLayerNotPresent,
    /// * Vulkan incompatible driver, indicating an issue with the Vulkan driver being incompatible.
    VulkanIncompatibleDriver,
    /// * Vulkan too many objects, indicating an issue with creating too many Vulkan objects.
    VulkanTooManyObjects,
    /// * Vulkan format not supported, indicating an issue with a requested Vulkan format not being supported.
    VulkanFormatNotSupported,
    /// * Vulkan fragmentation, indicating an issue with GPU memory fragmentation.
    VulkanFragmentation,
    /// * Vulkan unknown error, indicating an unspecified Vulkan error.
    VulkanUnknownError,
    /// * GLFW initialization failed, indicating an issue with initializing the GLFW library.
    GlfwInitFailed,
    /// * GLFW window creation failed, indicating an issue with creating a GLFW window.
    GlfwWindowCreationFailed,
    /// * GLFW context creation failed, indicating an issue with creating a GLFW context.
    GlfwContextCreationFailed,
    /// * GLFW unsupported platform, indicating an issue with running GLFW on the current platform.
    GlfwUnsupportedPlatform,
    /// * GLFW version mismatch, indicating an issue with a mismatch between the expected and actual GLFW version.
    GlfwVersionMismatch,
    /// * GLFW platform error, indicating an issue with a platform-specific error in GLFW.
    GlfwPlatformError,
    /// * GLFW unknown error, indicating an unspecified GLFW error.
    GlfwUnknownError,
    /// * Invalid state error, indicating that the library is in an invalid state for the attempted operation.
    InvalidState(&'static str),
}

/**
 * * Represents an error that can occur within the VMNL library,
 *   encapsulating both the kind of error and the location in the source code where it occurred.
 */
#[derive(Debug)]
pub struct VMNLError
{
    /// * The specific kind of error that occurred.
    kind:   VMNLErrorKind,
    /// * The location in the source code where the error occurred.
    caller: VMNLErrorLocation,
}

impl VMNLError
{
    /**
     * * Creates a new VMNLError instance with the specified error kind and caller location.
     *
     * ! Parameters:
     * - `kind`: The specific kind of error that occurred.
     *
     * ! Returns:
     * - A new instance of VMNLError containing the provided error kind and caller location.
     */
    #[track_caller]
    pub fn new(kind: VMNLErrorKind) -> Self
    {
        let caller = Location::caller();

        Self {
            kind,
            caller: VMNLErrorLocation {
                file: caller.file(),
                line: caller.line(),
                column: caller.column(),
            },
        }
    }

    /**
     * * Retrieves the kind of error that occurred.
     *
     * ! Returns:
     * - A reference to the VMNLErrorKind associated with this error.
     */
    pub fn kind(&self) -> &VMNLErrorKind
    {
        &self.kind
    }

    /**
    * * Retrieves the location in the source code where the error occurred.
    *
    * ! Returns:
    * - A VMNLErrorLocation struct containing the file, line, and column information of the error location.
    */
    pub fn location(&self) -> VMNLErrorLocation
    {
        self.caller
    }

        /**
    * * Formats the error message along with the location information for reporting.
    *
    * ! Returns:
    * - A formatted string containing the error message and its location in the source code.
    */
    pub fn report(&self) -> String
    {
        format!(
            "{} (at {}:{}:{})",
            self,
            self.caller.file,
            self.caller.line,
            self.caller.column
        )
    }
}

impl fmt::Display for VMNLError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match &self.kind {
            VMNLErrorKind::VulkanInitFailed =>
                f.write_str("vulkan initialization failed"),
            VMNLErrorKind::VulkanSurfaceCreationFailed =>
                f.write_str("vulkan surface creation failed"),
            VMNLErrorKind::VulkanSwapchainCreationFailed =>
                f.write_str("vulkan swapchain creation failed"),
            VMNLErrorKind::VulkanShaderModuleCreationFailed =>
                f.write_str("vulkan shader module creation failed"),
            VMNLErrorKind::VulkanPipelineCreationFailed =>
                f.write_str("vulkan pipeline creation failed"),
            VMNLErrorKind::VulkanBufferCreationFailed =>
                f.write_str("vulkan buffer creation failed"),
            VMNLErrorKind::VulkanMemoryAllocationFailed =>
                f.write_str("vulkan memory allocation failed"),
            VMNLErrorKind::VulkanCommandBufferCreationFailed =>
                f.write_str("vulkan command buffer creation failed"),
            VMNLErrorKind::VulkanDescriptorSetCreationFailed =>
                f.write_str("vulkan descriptor set creation failed"),
            VMNLErrorKind::VulkanSemaphoreCreationFailed =>
                f.write_str("vulkan semaphore creation failed"),
            VMNLErrorKind::VulkanFenceCreationFailed =>
                f.write_str("vulkan fence creation failed"),
            VMNLErrorKind::VulkanFramebufferCreationFailed =>
                f.write_str("vulkan framebuffer creation failed"),
            VMNLErrorKind::VulkanRenderPassCreationFailed =>
                f.write_str("vulkan render pass creation failed"),
            VMNLErrorKind::VulkanImageCreationFailed =>
                f.write_str("vulkan image creation failed"),
            VMNLErrorKind::VulkanImageViewCreationFailed =>
                f.write_str("vulkan image view creation failed"),
            VMNLErrorKind::VulkanSamplerCreationFailed =>
                f.write_str("vulkan sampler creation failed"),
            VMNLErrorKind::VulkanDescriptorPoolCreationFailed =>
                f.write_str("vulkan descriptor pool creation failed"),
            VMNLErrorKind::VulkanDescriptorSetLayoutCreationFailed =>
                f.write_str("vulkan descriptor set layout creation failed"),
            VMNLErrorKind::VulkanPipelineLayoutCreationFailed =>
                f.write_str("vulkan pipeline layout creation failed"),
            VMNLErrorKind::VulkanShaderCompilationFailed =>
                f.write_str("vulkan shader compilation failed"),
            VMNLErrorKind::VulkanValidationFailed =>
                f.write_str("vulkan validation failed"),
            VMNLErrorKind::VulkanUnsupportedFeature =>
                f.write_str("vulkan unsupported feature"),
            VMNLErrorKind::VulkanOutOfMemory =>
                f.write_str("vulkan out of memory"),
            VMNLErrorKind::VulkanDeviceLost =>
                f.write_str("vulkan device lost"),
            VMNLErrorKind::VulkanSurfaceLost =>
                f.write_str("vulkan surface lost"),
            VMNLErrorKind::VulkanExtensionNotPresent =>
                f.write_str("vulkan extension not present"),
            VMNLErrorKind::VulkanLayerNotPresent =>
                f.write_str("vulkan layer not present"),
            VMNLErrorKind::VulkanIncompatibleDriver =>
                f.write_str("vulkan incompatible driver"),
            VMNLErrorKind::VulkanTooManyObjects =>
                f.write_str("vulkan too many objects"),
            VMNLErrorKind::VulkanFormatNotSupported =>
                f.write_str("vulkan format not supported"),
            VMNLErrorKind::VulkanFragmentation =>
                f.write_str("vulkan fragmentation"),
            VMNLErrorKind::VulkanUnknownError =>
                f.write_str("vulkan unknown error"),
            VMNLErrorKind::GlfwInitFailed =>
                f.write_str("glfw initialization failed"),
            VMNLErrorKind::GlfwWindowCreationFailed =>
                f.write_str("glfw window creation failed"),
            VMNLErrorKind::GlfwContextCreationFailed =>
                f.write_str("glfw context creation failed"),
            VMNLErrorKind::GlfwUnsupportedPlatform =>
                f.write_str("glfw unsupported platform"),
            VMNLErrorKind::GlfwVersionMismatch =>
                f.write_str("glfw version mismatch"),
            VMNLErrorKind::GlfwPlatformError =>
                f.write_str("glfw platform error"),
            VMNLErrorKind::GlfwUnknownError =>
                f.write_str("glfw unknown error"),
            VMNLErrorKind::InvalidState(msg) =>
                write!(f, "invalid state: {msg}"),
        }
    }
}

impl Error for VMNLError {}

/**
 * * Utility function for logging messages within the VMNL library, prefixing them with a consistent tag for easier identification in logs.
 *
 * ! Parameters:
 * - `message`: The message to be logged, which can be any type that implements `AsRef<str>`,
 *   allowing for flexible string inputs (e.g., `String`, `&str`).
 *
 * ! Returns:
 * - A formatted string containing the log message prefixed with "[VMNL Log]" for consistent logging throughout the library.
 */
pub fn vmnl_log<S: AsRef<str>>(
    message: S
) -> String
{
    format!("[VMNL Log] {}", message.as_ref())
}

/// * A type alias for results returned by functions in the VMNL library, using the custom `VMNLError` type for error handling.
pub type VMNLResult<T> = Result<T, VMNLError>;
