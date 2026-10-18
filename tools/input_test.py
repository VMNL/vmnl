# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT
"""Run the selected native input contract after checking its prerequisites."""

from __future__ import annotations

import argparse
import ctypes
import json
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
        build = subprocess.run(
            ["cargo", "build", "-p", "vmnl-input-client", "--locked",
             "--message-format=json-render-diagnostics"],
            cwd=ROOT, check=False, stdout=subprocess.PIPE, text=True,
        )
        if build.returncode != 0:
            return build.returncode
        executable = vmnl_client_executable(build.stdout)
        environment = os.environ.copy()
        environment["VMNL_INPUT_CLIENT"] = executable
        result = subprocess.run(command, cwd=ROOT, env=environment, check=False)
    except OSError as error:
        print(f"could not start the VMNL native input test: {error}", file=sys.stderr)
        return 1
    return result.returncode


def vmnl_client_executable(cargo_output: str, target: str = "vmnl-input-client") -> str:
    """Use Cargo's artifact path, including a caller's CARGO_TARGET_DIR override."""
    try:
        records = [json.loads(line) for line in cargo_output.splitlines() if line.strip()]
    except json.JSONDecodeError as error:
        raise PrerequisiteError(f"invalid Cargo artifact output: {error}") from error
    if any(not isinstance(record, dict) for record in records):
        raise PrerequisiteError("Cargo artifact records must be JSON objects")
    executables = [
        record["executable"] for record in records
        if record.get("reason") == "compiler-artifact"
        and record.get("target", {}).get("name") == target
        and record.get("executable")
    ]
    if len(executables) != 1:
        raise PrerequisiteError("Cargo did not report exactly one VMNL input client executable")
    return executables[0]


def check_joystick_prerequisites(backend: str) -> None:
    """Check access without creating a device or changing the host."""
    if platform.system() != "Linux" or backend not in ("x11", "wayland"):
        raise PrerequisiteError("virtual joystick injection is currently Linux-only")
    try:
        descriptor = os.open("/dev/uinput", os.O_WRONLY | os.O_NONBLOCK)
        os.close(descriptor)
        with os.scandir("/dev/input"):
            pass
    except OSError as error:
        raise PrerequisiteError(
            f"uinput/input access unavailable: {error}; no host permissions were changed"
        ) from error
    if backend == "wayland":
        _check_wayland_socket(_wayland_socket(os.environ))
    elif not os.environ.get("DISPLAY"):
        raise PrerequisiteError("DISPLAY is unset")


def run_joystick_test(backend: str, mode: str) -> int:
    """Reuse the public-client launcher; the Rust driver owns the uinput device."""
    if mode not in ("runtime", "window"):
        raise PrerequisiteError("joystick mode must be runtime or window")
    check_joystick_prerequisites(backend)
    target = "vmnl-joystick-client"
    try:
        build = subprocess.run(
            ["cargo", "build", "-p", "vmnl-input-client", "--bin", target, "--locked",
             "--message-format=json-render-diagnostics"],
            cwd=ROOT, check=False, stdout=subprocess.PIPE, text=True,
        )
        if build.returncode != 0:
            return build.returncode
        environment = os.environ.copy()
        environment["VMNL_JOYSTICK_CLIENT"] = vmnl_client_executable(build.stdout, target)
        environment["VMNL_JOYSTICK_BACKEND"] = backend
        environment["XDG_SESSION_TYPE"] = backend
        if backend == "x11":
            environment.pop("WAYLAND_DISPLAY", None)
            environment.pop("WAYLAND_SOCKET", None)
        crate, test = (
            ("vmnl-platform-tests", "public_runtime_observes_virtual_joystick")
            if mode == "runtime" else
            ("vmnl-gpu-tests", "public_windows_observe_virtual_joystick")
        )
        command = ["cargo", "test", "-p", crate, "--locked", "--test", "native_joystick",
                   test, "--", "--ignored", "--exact", "--nocapture"]
        return subprocess.run(command, cwd=ROOT, env=environment, check=False).returncode
    except OSError as error:
        print(f"could not start the VMNL joystick test: {error}", file=sys.stderr)
        return 1


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    scenario = parser.add_mutually_exclusive_group()
    scenario.add_argument(
        "--vmnl",
        action="store_true",
        help="run the Vulkan-backed public VMNL scenario instead of the GLFW NoApi probe",
    )
    scenario.add_argument("--joystick", choices=("runtime", "window"),
                          help="run an opt-in Linux virtual joystick scenario")
    parser.add_argument("backend", choices=BACKEND_SYSTEM)
    arguments = parser.parse_args(argv)
    try:
        if arguments.joystick:
            return run_joystick_test(arguments.backend, arguments.joystick)
        if arguments.vmnl:
            return run_vmnl_input_test(arguments.backend)
        return run_input_test(arguments.backend)
    except PrerequisiteError as error:
        print(f"input-test prerequisite failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
