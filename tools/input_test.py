# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT
"""Run the selected native input contract after checking its prerequisites."""

from __future__ import annotations

import argparse
import ctypes
import os
import platform
import re
import shutil
import socket
import subprocess
import sys
from pathlib import Path
from typing import Mapping, Sequence

ROOT = Path(__file__).resolve().parents[1]
BACKEND_SYSTEM = {
    "x11": "Linux",
    "wayland": "Linux",
    "win32": "Windows",
    "cocoa": "Darwin",
}


class PrerequisiteError(RuntimeError):
    """A required native input-test environment is unavailable."""


def _xprop(display: str, arguments: Sequence[str]) -> str:
    executable = shutil.which("xprop")
    if executable is None:
        raise PrerequisiteError(
            "xprop is required to check the X11 window manager; install x11-utils"
        )

    try:
        result = subprocess.run(
            [executable, "-display", display, *arguments],
            capture_output=True,
            check=False,
            text=True,
        )
    except OSError as error:
        raise PrerequisiteError(f"could not run xprop: {error}") from error
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        raise PrerequisiteError(f"xprop could not query DISPLAY={display}: {detail}")
    return result.stdout


def _check_x11(display: str, *, require_weston: bool) -> None:
    window_manager = _xprop(display, ["-root", "_NET_SUPPORTING_WM_CHECK"])
    if not re.search(r"window id #\s*0x[0-9a-fA-F]+", window_manager):
        raise PrerequisiteError(
            f"DISPLAY={display} has no EWMH window manager; start an EWMH-compatible "
            "window manager before the native input test"
        )

    if not require_weston:
        return

    clients = _xprop(display, ["-root", "_NET_CLIENT_LIST"])
    windows = re.findall(r"0x[0-9a-fA-F]+", clients)
    for window in windows:
        try:
            window_class = _xprop(display, ["-id", window, "WM_CLASS"])
        except PrerequisiteError:
            continue
        if "Weston Compositor" in window_class:
            return
    raise PrerequisiteError(
        "the Wayland probe needs a mapped Weston X11-backend window on DISPLAY; "
        "start nested Weston on this X server first"
    )


def _wayland_socket(environ: Mapping[str, str]) -> Path:
    display_name = environ.get("WAYLAND_DISPLAY", "")
    if not display_name:
        raise PrerequisiteError("WAYLAND_DISPLAY is unset; select a running Wayland compositor")
    if display_name.startswith("/"):
        return Path(display_name)

    runtime_dir = environ.get("XDG_RUNTIME_DIR", "")
    if not runtime_dir:
        raise PrerequisiteError("XDG_RUNTIME_DIR is unset for the relative WAYLAND_DISPLAY")
    return Path(runtime_dir) / display_name


def _check_wayland_socket(path: Path) -> None:
    if not path.is_socket():
        raise PrerequisiteError(f"Wayland socket is unavailable at {path}; check the compositor")

    try:
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.settimeout(1)
            connection.connect(str(path))
    except OSError as error:
        raise PrerequisiteError(
            f"Wayland socket at {path} is not accepting clients: {error}"
        ) from error


def _cocoa_post_event_access() -> bool | None:
    framework = "/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics"
    try:
        core_graphics = ctypes.CDLL(framework)
        preflight = core_graphics.CGPreflightPostEventAccess
    except (AttributeError, OSError):
        return None
    preflight.restype = ctypes.c_bool
    return bool(preflight())


def _windows_input_desktop_access() -> bool:
    try:
        user32 = ctypes.WinDLL("user32", use_last_error=True)
    except (AttributeError, OSError):
        return False

    open_input_desktop = user32.OpenInputDesktop
    open_input_desktop.argtypes = [ctypes.c_uint32, ctypes.c_int, ctypes.c_uint32]
    open_input_desktop.restype = ctypes.c_void_p
    desktop = open_input_desktop(0, False, 0x0001 | 0x0080)
    if not desktop:
        return False

    user32.CloseDesktop.argtypes = [ctypes.c_void_p]
    user32.CloseDesktop.restype = ctypes.c_int
    user32.CloseDesktop(desktop)
    return True


def check_prerequisites(
    backend: str,
    *,
    system: str | None = None,
    environ: Mapping[str, str] | None = None,
) -> None:
    """Fail with setup guidance before launching a visible native probe."""
    if backend not in BACKEND_SYSTEM:
        choices = ", ".join(BACKEND_SYSTEM)
        raise PrerequisiteError(f"unknown backend {backend!r}; choose one of: {choices}")

    current_system = system or platform.system()
    expected_system = BACKEND_SYSTEM[backend]
    if current_system != expected_system:
        raise PrerequisiteError(
            f"backend {backend!r} requires {expected_system}; current host is {current_system}"
        )

    values = os.environ if environ is None else environ
    if backend in ("x11", "wayland"):
        display = values.get("DISPLAY", "")
        if not display:
            raise PrerequisiteError("DISPLAY is unset; an X11 server is required")
        if backend == "wayland":
            _check_wayland_socket(_wayland_socket(values))
        _check_x11(display, require_weston=backend == "wayland")
    elif backend == "cocoa" and _cocoa_post_event_access() is False:
        raise PrerequisiteError(
            "macOS denied posting input events; grant Accessibility permission to the "
            "terminal or app running just, then retry"
        )
    elif backend == "win32" and not _windows_input_desktop_access():
        raise PrerequisiteError(
            "no accessible interactive Windows input desktop; use an unlocked desktop "
            "session, with the target app at the same integrity level"
        )


def run_input_test(backend: str) -> int:
    check_prerequisites(backend)
    environment = os.environ.copy()
    environment["VMNL_PLATFORM_TEST_BACKEND"] = backend
    command = [
        "cargo",
        "test",
        "-p",
        "vmnl-platform-tests",
        "--locked",
        "--test",
        "backend_contract",
        "selected_backend_contract",
        "--",
        "--ignored",
        "--exact",
        "--nocapture",
    ]
    print(f"Running native input contract for {backend} on the active desktop.", flush=True)
    try:
        result = subprocess.run(command, cwd=ROOT, env=environment, check=False)
    except OSError as error:
        print(f"could not start the native input test: {error}", file=sys.stderr)
        return 1
    return result.returncode


def run_vmnl_input_test(backend: str) -> int:
    if backend != "x11":
        raise PrerequisiteError(
            "the public VMNL input scenario currently supports only X11"
        )
    check_prerequisites(backend)
    command = [
        "cargo",
        "test",
        "-p",
        "vmnl-gpu-tests",
        "--locked",
        "--test",
        "native_input",
        "vmnl_public_native_keyboard_and_mouse_events_update_input",
        "--",
        "--ignored",
        "--exact",
        "--nocapture",
    ]
    print(
        "Running the public VMNL input scenario on X11; Vulkan and XTEST are also required.",
        flush=True,
    )
    try:
        result = subprocess.run(command, cwd=ROOT, check=False)
    except OSError as error:
        print(f"could not start the VMNL native input test: {error}", file=sys.stderr)
        return 1
    return result.returncode


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--vmnl",
        action="store_true",
        help="run the Vulkan-backed public VMNL scenario instead of the GLFW NoApi probe",
    )
    parser.add_argument("backend", choices=BACKEND_SYSTEM)
    arguments = parser.parse_args(argv)
    try:
        if arguments.vmnl:
            return run_vmnl_input_test(arguments.backend)
        return run_input_test(arguments.backend)
    except PrerequisiteError as error:
        print(f"input-test prerequisite failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
