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

/// Represents the location in the source code where a VMNL error occurred.
#[derive(Debug, Clone, Copy)]
pub struct VMNLErrorLocation
{
    /// The file in which the error occurred.
    file:   &'static str,
    /// The line number at which the error occurred.
    line:   u32,
    /// The column number at which the error occurred.
    column: u32,
}

impl VMNLErrorLocation
{
    /// Returns the file in which the error occurred.
    #[inline]
    pub fn file(&self) -> &'static str
    {
        self.file
    }

    /// Returns the line number where the error occurred.
    #[inline]
    pub fn line(&self) -> u32
    {
        self.line
    }

    /// Returns the column number where the error occurred.
    #[inline]
    pub fn column(&self) -> u32
    {
        self.column
    }
}

/// Enum representing various kinds of errors that can occur within the VMNL library.
#[derive(Debug)]
#[non_exhaustive]
pub enum VMNLErrorKind
{
    /// Vulkan initialization failed.
    VulkanInitFailed,
    /// Vulkan surface creation failed.
    VulkanSurfaceCreationFailed,
    /// Vulkan swapchain creation failed.
    VulkanSwapchainCreationFailed,
    /// Vulkan shader module creation failed.
    VulkanShaderModuleCreationFailed,
    /// Vulkan pipeline creation failed.
    VulkanPipelineCreationFailed,
    /// Vulkan vertex buffer creation failed.
    VulkanVertexBufferCreationFailed,
    /// Vulkan index buffer creation failed.
    VulkanIndexBufferCreationFailed,
    /// Vulkan memory allocation failed.
    VulkanMemoryAllocationFailed,
    /// Vulkan command buffer creation failed.
    VulkanCommandBufferCreationFailed,
    /// Vulkan descriptor set creation failed.
    VulkanDescriptorSetCreationFailed,
    /// Vulkan semaphore creation failed.
    VulkanSemaphoreCreationFailed,
    /// Vulkan fence creation failed.
    VulkanFenceCreationFailed,
    /// Vulkan framebuffer creation failed.
    VulkanFramebufferCreationFailed,
    /// Vulkan render pass creation failed.
    VulkanRenderPassCreationFailed,
    /// Vulkan image creation failed.
    VulkanImageCreationFailed,
    /// Vulkan image view creation failed.
    VulkanImageViewCreationFailed,
    /// Vulkan sampler creation failed.
    VulkanSamplerCreationFailed,
    /// Vulkan descriptor pool creation failed.
    VulkanDescriptorPoolCreationFailed,
    /// Vulkan descriptor set layout creation failed.
    VulkanDescriptorSetLayoutCreationFailed,
    /// Vulkan pipeline layout creation failed.
    VulkanPipelineLayoutCreationFailed,
    /// Vulkan shader compilation failed.
    VulkanShaderCompilationFailed,
    /// Vulkan validation failed.
    VulkanValidationFailed,
    /// Vulkan unsupported feature encountered.
    VulkanUnsupportedFeature,
    /// Vulkan out of memory.
    VulkanOutOfMemory,
    /// Vulkan device lost.
    VulkanDeviceLost,
    /// Vulkan surface lost.
    VulkanSurfaceLost,
    /// Required Vulkan extension not present.
    VulkanExtensionNotPresent,
    /// Required Vulkan layer not present.
    VulkanLayerNotPresent,
    /// Vulkan driver is incompatible.
    VulkanIncompatibleDriver,
    /// Too many Vulkan objects created.
    VulkanTooManyObjects,
    /// Requested Vulkan format not supported.
    VulkanFormatNotSupported,
    /// GPU memory fragmentation.
    VulkanFragmentation,
    /// Unknown Vulkan error.
    VulkanUnknownError,
    /// GLFW initialization failed.
    GlfwInitFailed,
    /// GLFW window creation failed.
    GlfwWindowCreationFailed,
    /// GLFW context creation failed.
    GlfwContextCreationFailed,
    /// GLFW unsupported platform.
    GlfwUnsupportedPlatform,
    /// GLFW version mismatch.
    GlfwVersionMismatch,
    /// Platform-specific GLFW error.
    GlfwPlatformError,
    /// Unknown GLFW error.
    GlfwUnknownError,
    /// Invalid state error with message.
    InvalidState(&'static str),
}

/// Represents an error that can occur within the VMNL library,
/// encapsulating both the kind of error and the location in the source code where it occurred.
#[derive(Debug)]
pub struct VMNLError
{
    /// The specific kind of error that occurred.
    kind:   VMNLErrorKind,
    /// The location in the source code where the error occurred.
    caller: VMNLErrorLocation,
}

impl VMNLError
{
    /// Create a new `VMNLError` with the specified error kind and caller location.
    ///
    /// # Parameters
    /// - `kind`: The specific kind of error that occurred.
    ///
    /// # Returns
    /// A new `VMNLError` containing the provided error kind and caller location.
    #[track_caller]
    pub fn new(kind: VMNLErrorKind) -> Self
    {
        let caller: &Location = Location::caller();

        Self {
            kind,
            caller: VMNLErrorLocation {
                file: caller.file(),
                line: caller.line(),
                column: caller.column(),
            },
        }
    }

    /// Returns the kind of error that occurred.
    #[inline]
    pub fn kind(&self) -> &VMNLErrorKind
    {
        &self.kind
    }

    /// Returns the location in the source code where the error occurred.
    #[inline]
    pub fn location(&self) -> VMNLErrorLocation
    {
        self.caller
    }

    /// Format the error message along with location information for reporting.
    ///
    /// # Returns
    /// A formatted string containing the error message and its location in the source code.
    #[inline]
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
    /// Format the error message based on the specific kind of error, providing a human-readable description.
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
            VMNLErrorKind::VulkanVertexBufferCreationFailed =>
                f.write_str("vulkan vertex buffer creation failed"),
            VMNLErrorKind::VulkanIndexBufferCreationFailed =>
                f.write_str("vulkan index buffer creation failed"),
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

/// Utility function for logging messages within the VMNL library,
/// prefixing them with a consistent tag for easier identification in logs.
///
/// # Arguments
/// - `message`: The message to be logged, any type implementing `AsRef<str>`.
///
/// # Returns
/// A formatted string containing the log message prefixed with "[VMNL Log]".
#[inline]
pub fn vmnl_log<S: AsRef<str>>(
    message: S
) -> String
{
    format!("[VMNL Log] {}", message.as_ref())
}

/// Type alias for results returned by functions in the VMNL library using `VMNLError`.
pub type VMNLResult<T> = Result<T, VMNLError>;
