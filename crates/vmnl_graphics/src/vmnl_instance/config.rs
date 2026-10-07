// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! CPU-side context configuration and device feature requirements.

use super::{Context, VMNLInstance};
use crate::VMNLResult;
use std::rc::Rc;
use vulkano::device::DeviceFeatures;

/// Optional Vulkan device functionality selectable through VMNL.
///
/// Requiring a feature filters physical-device candidates and enables it on the
/// selected logical device. Capability support alone does not enable a feature.
/// Device limits still constrain the sizes and modes used by shaders/pipelines.
/// This catalogue is non-exhaustive so future features can be added.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceFeature {
    /// Enable non-solid polygon fill modes, including wireframe and points.
    FillModeNonSolid,
    /// Enable line widths other than `1.0`, within the device's supported range.
    WideLines,
    /// Enable point sizes greater than `1.0`, within the device's supported range.
    LargePoints,
}

impl DeviceFeature {
    pub(super) fn is_set_in(self, features: &DeviceFeatures) -> bool {
        match self {
            Self::FillModeNonSolid => features.fill_mode_non_solid,
            Self::WideLines => features.wide_lines,
            Self::LargePoints => features.large_points,
        }
    }
}

/// CPU-side requirements for the logical device created by a [`ContextBuilder`].
///
/// The default requests no optional features. Requirements accumulate without
/// duplicates, in first-request order. No GPU or GLFW initialization occurs
/// while configuring this value; its requirement list may allocate CPU memory.
#[derive(Clone, Debug, Default)]
pub struct DeviceConfig {
    required_features: Vec<DeviceFeature>,
}

impl DeviceConfig {
    /// Adds a required feature. Repeated requests have no effect.
    #[must_use]
    pub fn require_feature(mut self, feature: DeviceFeature) -> Self {
        if !self.required_features.contains(&feature) {
            self.required_features.push(feature);
        }

        self
    }

    /// Adds required features without replacing existing requirements.
    #[must_use]
    pub fn require_features(mut self, features: impl IntoIterator<Item = DeviceFeature>) -> Self {
        for feature in features {
            self = self.require_feature(feature);
        }

        self
    }

    /// Returns the required features, in first-request order without duplicates.
    #[must_use]
    pub fn required_features(&self) -> &[DeviceFeature] {
        &self.required_features
    }

    pub(super) fn enabled_features(&self) -> DeviceFeatures {
        DeviceFeatures {
            fill_mode_non_solid: self
                .required_features
                .contains(&DeviceFeature::FillModeNonSolid),
            wide_lines: self.required_features.contains(&DeviceFeature::WideLines),
            large_points: self.required_features.contains(&DeviceFeature::LargePoints),
            ..DeviceFeatures::empty()
        }
    }
}

/// Builder for a graphics [`Context`], with automatic device and queue selection.
///
/// Construction and configuration are CPU-only. The default preserves
/// [`Context::new`]'s automatic selection policy and requests no optional device
/// features. No Vulkan objects or GLFW lifetime are acquired until [`Self::build`].
#[derive(Clone, Debug, Default)]
pub struct ContextBuilder {
    device: DeviceConfig,
}

impl ContextBuilder {
    /// Replaces the device configuration, including all feature requirements.
    #[must_use]
    pub fn device(mut self, config: DeviceConfig) -> Self {
        self.device = config;
        self
    }

    /// Returns the current CPU-side device configuration.
    #[must_use]
    pub const fn device_config(&self) -> &DeviceConfig {
        &self.device
    }

    /// Builds the complete context, enabling every requested device feature.
    ///
    /// This initializes GLFW and a Vulkan instance, filters physical devices by
    /// the baseline extensions/graphics queue and all requested features, then
    /// creates the selected logical device, queue and allocators. Existing
    /// device-type ranking and backend-dependent tie ordering are preserved.
    /// Requested features are never silently dropped. Temporary initialization
    /// objects are released on failure; clones of the result share one device.
    ///
    /// # Errors
    /// Returns `DeviceRequirementsNotMet` if no compatible GPU satisfies a
    /// nonempty feature request. An empty request preserves the existing
    /// `VulkanUnsupportedFeature` category when no compatible GPU exists.
    /// GLFW, instance enumeration/creation and device-creation failures retain
    /// their existing categories. Capability checks require the Vulkan instance;
    /// failure can occur after instance creation but before device creation.
    ///
    /// # Example
    /// ```rust,no_run
    /// use vmnl_graphics::{Context, DeviceConfig, DeviceFeature};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// let context = Context::builder()
    ///     .device(DeviceConfig::default().require_feature(DeviceFeature::WideLines))
    ///     .build()?;
    /// assert!(context.is_device_feature_enabled(DeviceFeature::WideLines));
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> VMNLResult<Context> {
        Ok(Context {
            inner: Rc::new(VMNLInstance::new(&self.device)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_requests_map_exactly_to_vulkan_flags() {
        let features = [
            DeviceFeature::FillModeNonSolid,
            DeviceFeature::WideLines,
            DeviceFeature::LargePoints,
        ];
        for mask in 0..8 {
            let config = features.iter().enumerate().fold(
                DeviceConfig::default(),
                |config, (index, &feature)| {
                    if mask & (1 << index) != 0 {
                        config.require_feature(feature)
                    } else {
                        config
                    }
                },
            );
            let enabled = config.enabled_features();
            assert_eq!(enabled.fill_mode_non_solid, mask & 1 != 0);
            assert_eq!(enabled.wide_lines, mask & 2 != 0);
            assert_eq!(enabled.large_points, mask & 4 != 0);
            assert!(!enabled.depth_clamp);
            assert!(!enabled.geometry_shader);
            assert!(!enabled.sampler_anisotropy);
            for (index, &feature) in features.iter().enumerate() {
                assert_eq!(feature.is_set_in(&enabled), mask & (1 << index) != 0);
            }
        }
    }
}
