////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Monitor discovery and monitor metadata helpers.
////////////////////////////////////////////////////////////////////////////////
extern crate glfw;

/// Video mode supported by a monitor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoMode {
    /// The width of the video mode in pixels.
    pub width: u32,
    /// The height of the video mode in pixels.
    pub height: u32,
    /// The number of bits per color channel (**red**, *green*, *blue*) for the video mode.
    pub red_bits: u32,
    /// The number of bits per color channel (*red*, **green**, *blue*) for the video mode.
    pub green_bits: u32,
    /// The number of bits per color channel (*red*, *green*, **blue**) for the video mode.
    pub blue_bits: u32,
    /// The refresh rate of the video mode in hertz (Hz).
    pub refresh_rate: u32,
}

/// Converts a `glfw::VidMode` into a `VideoMode`.
impl From<glfw::VidMode> for VideoMode {
    fn from(mode: glfw::VidMode) -> Self {
        Self {
            width: mode.width,
            height: mode.height,
            red_bits: mode.red_bits,
            green_bits: mode.green_bits,
            blue_bits: mode.blue_bits,
            refresh_rate: mode.refresh_rate,
        }
    }
}

/// Snapshot of one connected monitor and its display capabilities.
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// The name of the monitor, if available.
    pub name: Option<String>,
    /// The position of the monitor in virtual screen coordinates (x, y).
    pub position: (i32, i32),
    /// The physical size of the monitor in millimeters (width, height).
    pub physical_size_mm: (i32, i32),
    /// The content scale of the monitor (x, y).
    pub content_scale: (f32, f32),
    /// The work area of the monitor, defined as (x, y, width, height) in virtual screen coordinates.
    pub workarea: (i32, i32, i32, i32),
    /// The current video mode of the monitor, if available.
    pub current_mode: Option<VideoMode>,
    /// A list of all available video modes supported by the monitor.
    pub available_modes: Vec<VideoMode>,
    /// Indicates whether this monitor is the primary monitor.
    pub is_primary: bool,
}

/// Collection of connected monitor snapshots.
#[derive(Debug, Clone)]
pub struct Monitors {
    /// A vector containing information about all connected monitors.
    info: Vec<MonitorInfo>,
}

impl Monitors {
    /// Creates a new `Monitors` instance by querying the connected monitors from GLFW.
    ///
    /// # Arguments
    /// - `glfw`: A mutable reference to the GLFW instance to access monitor information.
    ///
    /// # Returns
    /// A `Monitors` instance containing information about all connected monitors.
    pub(crate) fn new(glfw: &mut glfw::Glfw) -> Self {
        Self {
            info: glfw.with_connected_monitors(|_, monitors| {
                monitors
                    .iter()
                    .enumerate()
                    .map(|(index, monitor)| MonitorInfo {
                        name: monitor.get_name(),
                        position: monitor.get_pos(),
                        physical_size_mm: monitor.get_physical_size(),
                        content_scale: monitor.get_content_scale(),
                        workarea: monitor.get_workarea(),
                        current_mode: monitor.get_video_mode().map(VideoMode::from),
                        available_modes: monitor
                            .get_video_modes()
                            .into_iter()
                            .map(VideoMode::from)
                            .collect(),
                        is_primary: index == 0,
                    })
                    .collect()
            }),
        }
    }

    /// Returns a slice of `MonitorInfo` for all connected monitors.
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// for monitor in window.monitor().infos() {
    ///     println!("Monitor: {}", monitor.name.clone().unwrap_or("Unknown".to_string()));
    ///     println!("  Position: ({}, {})", monitor.position.0, monitor.position.1);
    ///     println!("  Physical Size: {}mm x {}mm", monitor.physical_size_mm.0, monitor.physical_size_mm.1);
    ///     println!("  Content Scale: ({}, {})", monitor.content_scale.0, monitor.content_scale.1);
    ///     println!("  Work Area: ({}x{} at ({}, {}))", monitor.workarea.2, monitor.workarea.3, monitor.workarea.0, monitor.workarea.1);
    ///     println!("  Current Mode: {}x{} @ {}Hz ({} bits per channel)", monitor.current_mode.as_ref().map_or(0, |mode| mode.width), monitor.current_mode.as_ref().map_or(0, |mode| mode.height), monitor.current_mode.as_ref().map_or(0, |mode| mode.refresh_rate), monitor.current_mode.as_ref().map_or(0, |mode| mode.red_bits + mode.green_bits + mode.blue_bits));
    ///     println!("  Available Modes:");
    ///     for mode in &monitor.available_modes {
    ///         println!("    - {}x{} @ {}Hz", mode.width, mode.height, mode.refresh_rate);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn infos(&self) -> &[MonitorInfo] {
        &self.info
    }

    /// Returns a vector of monitor names for all connected monitors.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// let monitor_names = window.monitor().names();
    /// println!("Connected monitors: {}", monitor_names.iter().map(|name| name.clone().unwrap_or("Unknown".to_string())).collect::<Vec<String>>().join(", "));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn names(&self) -> Vec<Option<String>> {
        self.info.iter().map(|info| info.name.clone()).collect()
    }

    /// Returns a reference to the primary monitor's information, if available.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let window = Window::builder().build(&context)?;
    /// if let Some(primary_monitor) = window.monitor().primary() {
    ///     println!("Primary Monitor: {}", primary_monitor.name.clone().unwrap_or("Unknown".to_string()));
    ///     println!("  Position: ({}, {})", primary_monitor.position.0, primary_monitor.position.1);
    ///     println!("  Physical Size: {}mm x {}mm", primary_monitor.physical_size_mm.0, primary_monitor.physical_size_mm.1);
    ///     println!("  Content Scale: ({}, {})", primary_monitor.content_scale.0, primary_monitor.content_scale.1);
    ///     println!("  Work Area: ({}x{} at ({}, {}))", primary_monitor.workarea.2, primary_monitor.workarea.3, primary_monitor.workarea.0, primary_monitor.workarea.1);
    ///     if let Some(mode) = &primary_monitor.current_mode {
    ///         println!(
    ///             "  Current Mode: {}x{} @ {}Hz",
    ///             mode.width, mode.height, mode.refresh_rate
    ///         );
    ///     }
    ///     println!("  Available Modes:");
    ///     for mode in &primary_monitor.available_modes {
    ///         println!("    - {}x{} @ {}Hz", mode.width, mode.height, mode.refresh_rate);
    ///     }
    /// } else {
    ///     println!("No primary monitor detected.");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn primary(&self) -> Option<&MonitorInfo> {
        self.info.iter().find(|info| info.is_primary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(name: Option<&str>, is_primary: bool) -> MonitorInfo {
        MonitorInfo {
            name: name.map(str::to_string),
            position: (0, 0),
            physical_size_mm: (600, 340),
            content_scale: (1.0, 1.0),
            workarea: (0, 0, 1920, 1080),
            current_mode: None,
            available_modes: Vec::new(),
            is_primary,
        }
    }

    #[test]
    fn video_mode_from_glfw_preserves_fields() {
        let mode: glfw::VidMode = glfw::VidMode {
            width: 1920,
            height: 1080,
            red_bits: 8,
            green_bits: 8,
            blue_bits: 8,
            refresh_rate: 144,
        };

        assert_eq!(
            VideoMode::from(mode),
            VideoMode {
                width: 1920,
                height: 1080,
                red_bits: 8,
                green_bits: 8,
                blue_bits: 8,
                refresh_rate: 144,
            }
        );
    }

    #[test]
    fn monitors_names_and_primary_are_derived_from_info() {
        let monitors: Monitors = Monitors {
            info: vec![monitor(Some("Primary"), true), monitor(None, false)],
        };

        assert_eq!(monitors.infos().len(), 2);
        assert_eq!(monitors.names(), vec![Some("Primary".to_string()), None]);
        assert_eq!(
            monitors.primary().map(|monitor| monitor.name.as_deref()),
            Some(Some("Primary"))
        );
    }

    #[test]
    fn primary_returns_none_when_no_monitor_is_marked_primary() {
        let monitors: Monitors = Monitors {
            info: vec![monitor(Some("Secondary"), false)],
        };

        assert!(monitors.primary().is_none());
    }
}
