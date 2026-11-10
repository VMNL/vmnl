// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Immutable depth comparison and write policy for raw pipelines.

use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use vulkano::pipeline::graphics::depth_stencil::{
    CompareOp as VkCompareOp, DepthState as VkDepthState, DepthStencilState,
};

/// Compares incoming fragment depth with the value stored in the depth attachment.
///
/// These operators require no optional device feature. Equality compares stored
/// precision; it does not apply a tolerance. The viewport's depth range transforms
/// shader depth before comparison. See [`DepthState`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareOp {
    /// Always reject the fragment.
    Never,
    /// Pass when incoming depth is less than stored depth.
    Less,
    /// Pass when incoming depth equals stored depth.
    Equal,
    /// Pass when incoming depth is less than or equal to stored depth.
    LessOrEqual,
    /// Pass when incoming depth is greater than stored depth.
    Greater,
    /// Pass when incoming depth differs from stored depth.
    NotEqual,
    /// Pass when incoming depth is greater than or equal to stored depth.
    GreaterOrEqual,
    /// Always pass the depth test.
    Always,
}

impl CompareOp {
    const fn to_vk(self) -> VkCompareOp {
        match self {
            Self::Never => VkCompareOp::Never,
            Self::Less => VkCompareOp::Less,
            Self::Equal => VkCompareOp::Equal,
            Self::LessOrEqual => VkCompareOp::LessOrEqual,
            Self::Greater => VkCompareOp::Greater,
            Self::NotEqual => VkCompareOp::NotEqual,
            Self::GreaterOrEqual => VkCompareOp::GreaterOrEqual,
            Self::Always => VkCompareOp::Always,
        }
    }
}

/// Depth policy fixed at raw pipeline construction; defaults to [`Self::Disabled`].
///
/// [`Self::Test`] requires a window with a depth attachment, even for `Always`
/// with writes disabled. An absent attachment is rejected before shader I/O.
/// Depth is cleared once per frame using the window's render-target configuration,
/// shared by ordered raw draws, and discarded after the frame. High-level 2D draws
/// neither test nor write depth. No stencil, depth bounds, or depth bias is enabled.
/// Configuring or inspecting this CPU value performs no allocation or GPU work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DepthState {
    /// Disable both depth testing and writes, with or without an attachment.
    #[default]
    Disabled,
    /// Enable comparison and optionally write depth for passing fragments.
    ///
    /// Use `Always` with `write_enable: true` for unconditional depth writes.
    /// With writes disabled, earlier depth values remain available to later draws.
    Test {
        /// Operation comparing incoming depth against stored depth.
        compare_op: CompareOp,
        /// Whether passing fragments update the depth attachment.
        write_enable: bool,
    },
}

impl DepthState {
    pub(super) fn resolve(self, has_depth: bool) -> VMNLResult<Option<DepthStencilState>> {
        if !has_depth {
            return match self {
                Self::Disabled => Ok(None),
                Self::Test { .. } => Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "raw depth testing requires a window depth attachment".into(),
                ))),
            };
        }
        Ok(Some(DepthStencilState {
            depth: match self {
                Self::Disabled => None,
                Self::Test {
                    compare_op,
                    write_enable,
                } => Some(VkDepthState {
                    write_enable,
                    compare_op: compare_op.to_vk(),
                }),
            },
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_state_resolves_attachment_test_write_and_comparison() -> VMNLResult<()> {
        assert!(DepthState::Disabled.resolve(false)?.is_none());
        let disabled = DepthState::Disabled
            .resolve(true)?
            .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
        assert!(disabled.depth.is_none());
        assert!(disabled.stencil.is_none());
        assert!(disabled.depth_bounds.is_none());
        for (public, backend) in [
            (CompareOp::Never, VkCompareOp::Never),
            (CompareOp::Less, VkCompareOp::Less),
            (CompareOp::Equal, VkCompareOp::Equal),
            (CompareOp::LessOrEqual, VkCompareOp::LessOrEqual),
            (CompareOp::Greater, VkCompareOp::Greater),
            (CompareOp::NotEqual, VkCompareOp::NotEqual),
            (CompareOp::GreaterOrEqual, VkCompareOp::GreaterOrEqual),
            (CompareOp::Always, VkCompareOp::Always),
        ] {
            for write_enable in [false, true] {
                let state = DepthState::Test {
                    compare_op: public,
                    write_enable,
                };
                assert!(matches!(state.resolve(false), Err(error)
                    if matches!(error.kind(), VMNLErrorKind::InvalidState(_))));
                let resolved = state
                    .resolve(true)?
                    .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
                let depth = resolved
                    .depth
                    .ok_or_else(|| VMNLError::new(VMNLErrorKind::VulkanValidationFailed))?;
                assert_eq!(depth.compare_op, backend);
                assert_eq!(depth.write_enable, write_enable);
                assert!(resolved.stencil.is_none());
                assert!(resolved.depth_bounds.is_none());
            }
        }
        Ok(())
    }
}
