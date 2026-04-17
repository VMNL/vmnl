////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Brief
////////////////////////////////////////////////////////////////////////////////

extern crate glfw;

/// Monitor information and utilities for the VMNL window module.
///
/// This module defines the `Monitor` struct, which encapsulates information about connected monitors,
/// including their video modes, physical characteristics, and primary status. It provides methods
/// to access this information in a structured way.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoMode
{
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
impl From<glfw::VidMode> for VideoMode
{
    fn from(mode: glfw::VidMode) -> Self
    {
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

/// Represents detailed information about a monitor, including its name, position, physical size,
/// content scale, work area, current video mode, available video modes, and whether it is the primary monitor.
#[derive(Debug, Clone)]
pub struct MonitorInfo
{
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

/// Represents the collection of connected monitors and their information.
#[derive(Debug, Clone)]
pub struct Monitors
{
    /// A vector containing information about all connected monitors.
    info: Vec<MonitorInfo>,
}

impl Monitors
{
    /// Creates a new `Monitors` instance by querying the connected monitors from GLFW.
    ///
    /// # Arguments
    /// - `glfw`: A mutable reference to the GLFW instance to access monitor information.
    ///
    /// # Returns
    /// A `Monitors` instance containing information about all connected monitors.
    pub(crate) fn new(glfw: &mut glfw::Glfw) -> Self
    {
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
            })
        }
    }

    /// Returns a slice of `MonitorInfo` for all connected monitors.
    /// # Example
    /// ```rust
    /// for monitor in win.monitor().infos() {
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
    /// ```
    pub fn infos(&self) -> &[MonitorInfo]
    {
        &self.info
    }

    /// Returns a vector of monitor names for all connected monitors.
    ///
    /// # Example
    /// ```rust
    /// let monitor_names = win.monitor().names();
    /// println!("Connected monitors: {}", monitor_names.iter().map(|name| name.clone().unwrap_or("Unknown".to_string())).collect::<Vec<String>>().join(", "));
    /// ```
    pub fn names(&self) -> Vec<Option<String>>
    {
        self.info.iter().map(|info| info.name.clone()).collect()
    }

    /// Returns a reference to the primary monitor's information, if available.
    ///
    /// # Example
    /// ```rust
    /// if let Some(primary_monitor) = win.monitor().primary() {
    ///     println!("Primary Monitor: {}", primary_monitor.name.clone().unwrap_or("Unknown".to_string()));
    ///     println!("  Position: ({}, {})", primary_monitor.position.0, primary_monitor.position.1);
    ///     println!("  Physical Size: {}mm x {}mm", primary_monitor.physical_size_mm.0, primary_monitor.physical_size_mm.1);
    ///     println!("  Content Scale: ({}, {})", primary_monitor.content_scale.0, primary_monitor.content_scale.1);
    ///     println!("  Work Area: ({}x{} at ({}, {}))", primary_monitor.workarea.2, primary_monitor.workarea.3, primary_monitor.workarea.0, primary_monitor.workarea.1);
    ///     println!("  Current Mode: {}x{} @ {}Hz ({} bits per channel)", primary_monitor.current_mode.as_ref().map_or(0, |mode| mode.width),
    ///     primary_monitor.current_mode.as_ref().map_or(0, |mode| mode.height),
    ///     primary_monitor.current_mode.as_ref().map_or(0, |mode| mode.refresh_rate),
    ///     primary_monitor.current_mode.as_ref().map_or(0, |mode|) mode.red_bits + mode.green_bits + mode.blue_bits));
    ///     println!("  Available Modes:");
    ///     for mode in &primary_monitor.available_modes {
    ///         println!("    - {}x{} @ {}Hz", mode.width, mode.height, mode.refresh_rate);
    ///     }
    /// } else {
    ///     println!("No primary monitor detected.");
    /// }
    /// ```
    pub fn primary(&self) -> Option<&MonitorInfo>
    {
        self.info.iter().find(|info| info.is_primary)
    }
}
