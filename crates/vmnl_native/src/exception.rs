////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// * Exception handling for the VMNL library, defining custom error types and result aliases.
////////////////////////////////////////////////////////////////////////////////

use std::{
    error::Error,
    fmt
};

/**
 * * Defines the VMNLError enum, which represents various error conditions that can occur within the VMNL library.
 */
#[derive(Debug)]
pub enum VMNLError
{
    VMNLInitFailed,
    WindowCreationFailed,
    VulkanInitFailed,
    InvalidState(&'static str),
}

/**
 * * Implements the Display trait for VMNLError to provide human-readable error messages.
 * * Each variant of the VMNLError enum is matched to a specific error message that describes the
 * * nature of the error, making it easier for developers to understand what went wrong when an error occurs.
 */
impl fmt::Display for VMNLError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            Self::VMNLInitFailed =>                     write!(f, "VMNL Error: VMNL initialization failed"),
            Self::WindowCreationFailed =>               write!(f, "VMNL Error: window creation failed"),
            Self::VulkanInitFailed =>                   write!(f, "VMNL Error: Vulkan initialization failed"),
            Self::InvalidState(msg) =>   write!(f, "VMNL Error: invalid state: {msg}"),
        }
    }
}

/**
 * * Implements the Error trait for VMNLError, allowing it to be used as a standard error type in Rust.
 */
impl Error for VMNLError {}
/**
 * * Defines a type alias VMNLResult<T> for Result<T, VMNLError>, simplifying the return type for functions that may produce a VMNLError.
 */
pub type VMNLResult<T> = Result<T, VMNLError>;
