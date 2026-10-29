// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Prepare the owned X11 probe's pointer before authorizing scenario input.

use std::{
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection as _,
    protocol::{
        xproto::{AtomEnum, ConnectionExt as _, MOTION_NOTIFY_EVENT},
        xtest::ConnectionExt as _,
    },
};

pub(super) fn requested_window(signal: &str) -> Result<Option<u32>, String> {
    let Some(request) = signal
        .strip_prefix("POINTER ")
        .and_then(|s| s.strip_suffix('\n'))
    else {
        return Ok(None);
    };
    request
        .parse::<u32>()
        .ok()
        .filter(|window| *window != 0)
        .map(Some)
        .ok_or_else(|| format!("invalid X11 pointer preparation request: {signal:?}"))
}

pub(super) fn prepare(window: u32, child_pid: u32) -> Result<(), String> {
    let (connection, screen) = x11rb::connect(None).map_err(|e| e.to_string())?;
    let root = connection.setup().roots[screen].root;
    let pid_atom = connection
        .intern_atom(false, b"_NET_WM_PID")
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?
        .atom;
    let property = connection
        .get_property(false, window, pid_atom, AtomEnum::CARDINAL, 0, 1)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    let pid = property.value32().and_then(|mut values| values.next());
    let focus = connection
        .get_input_focus()
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?
        .focus;
    if pid != Some(child_pid) || focus != window {
        return Err(format!("X11 pointer preparation requires the focused owned probe: window={window:#x}, pid={pid:?}, expected_pid={child_pid}, focus={focus:#x}"));
    }
    let geometry = connection
        .get_geometry(window)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    let (x, y) = inside_point(geometry.width, geometry.height)?;
    let target = connection
        .translate_coordinates(window, root, x, y)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    if !target.same_screen {
        return Err("X11 probe and pointer root are on different screens".to_owned());
    }
    connection
        .xtest_get_version(2, 2)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;
    // A client XWarpPointer can update coordinates without delivering a crossing on Xwayland.
    connection
        .xtest_fake_input(
            MOTION_NOTIFY_EVENT,
            0,
            0,
            root,
            target.dst_x,
            target.dst_y,
            0,
        )
        .map_err(|e| e.to_string())?
        .check()
        .map_err(|e| e.to_string())?;
    connection.flush().map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        let pointer = connection
            .query_pointer(root)
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?;
        if (pointer.root_x, pointer.root_y) == (target.dst_x, target.dst_y) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "X11 pointer preparation target=({}, {}), observed=({}, {})",
                target.dst_x, target.dst_y, pointer.root_x, pointer.root_y
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn inside_point(width: u16, height: u16) -> Result<(i16, i16), String> {
    if width == 0 || height == 0 {
        return Err("X11 probe has an empty drawable".to_owned());
    }
    // Keep the target away from the client's former center warp and window borders.
    Ok((
        i16::try_from(width / 3).map_err(|e| e.to_string())?,
        i16::try_from(height / 3).map_err(|e| e.to_string())?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{inside_point, requested_window};

    #[test]
    fn pointer_preparation_requires_a_complete_nonzero_x11_window() {
        assert_eq!(requested_window("POINTER 123\n"), Ok(Some(123)));
        for signal in [
            "",
            "POINTER ",
            "POINTER 123",
            "MAPPED\n",
            "POSITIONED\n",
            "READY\n",
        ] {
            assert_eq!(requested_window(signal), Ok(None));
        }
        for signal in [
            "POINTER 0\n",
            "POINTER -1\n",
            "POINTER 4294967296\n",
            "POINTER x\n",
            "POINTER 1 2\n",
        ] {
            assert!(requested_window(signal).is_err());
        }
    }

    #[test]
    fn preparation_point_is_inside_every_nonempty_drawable() {
        for (width, height) in [(1, 1), (160, 120), (1416, 1719), (u16::MAX, u16::MAX)] {
            let (x, y) = inside_point(width, height).expect("nonempty drawable");
            assert!(x >= 0 && i32::from(x) < i32::from(width));
            assert!(y >= 0 && i32::from(y) < i32::from(height));
        }
        assert!(inside_point(0, 120).is_err());
        assert!(inside_point(160, 0).is_err());
    }
}
