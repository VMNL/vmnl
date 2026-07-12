////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Shared shader source descriptors.
////////////////////////////////////////////////////////////////////////////////
use std::path::PathBuf;

/// Shader input, either as inline GLSL source or as a path to a GLSL source file.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShaderSource {
    /// Inline GLSL source.
    Src(String),
    /// Path to a GLSL source file.
    Path(PathBuf),
}
