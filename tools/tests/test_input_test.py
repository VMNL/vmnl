# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

import importlib.util
import socket
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

SCRIPT = Path(__file__).parents[1] / "input_test.py"
SPEC = importlib.util.spec_from_file_location("input_test", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
input_test = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(input_test)


class InputTestPrerequisiteTests(unittest.TestCase):
    def test_rejects_unknown_backend(self) -> None:
        with self.assertRaisesRegex(input_test.PrerequisiteError, "unknown backend"):
            input_test.check_prerequisites("null", system="Linux", environ={})

    def test_rejects_backend_for_another_host(self) -> None:
        with self.assertRaisesRegex(input_test.PrerequisiteError, "requires Linux"):
            input_test.check_prerequisites("x11", system="Windows", environ={})

    def test_x11_requires_a_display(self) -> None:
        with self.assertRaisesRegex(input_test.PrerequisiteError, "DISPLAY is unset"):
            input_test.check_prerequisites("x11", system="Linux", environ={})

    def test_wayland_requires_its_socket(self) -> None:
        with tempfile.TemporaryDirectory() as runtime_dir:
            environment = {
                "DISPLAY": ":99",
                "XDG_RUNTIME_DIR": runtime_dir,
                "WAYLAND_DISPLAY": "wayland-test",
            }
            with self.assertRaisesRegex(input_test.PrerequisiteError, "socket is unavailable"):
                input_test.check_prerequisites("wayland", system="Linux", environ=environment)

    def test_wayland_rejects_a_socket_that_does_not_accept_clients(self) -> None:
        with tempfile.TemporaryDirectory() as runtime_dir:
            socket_path = Path(runtime_dir) / "wayland-test"
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
                server.bind(str(socket_path))
                environment = {
                    "DISPLAY": ":99",
                    "XDG_RUNTIME_DIR": runtime_dir,
                    "WAYLAND_DISPLAY": "wayland-test",
                }
                with self.assertRaisesRegex(
                    input_test.PrerequisiteError, "not accepting clients"
                ):
                    input_test.check_prerequisites(
                        "wayland", system="Linux", environ=environment
                    )

    def test_x11_requires_an_ewmh_window_manager(self) -> None:
        with patch.object(input_test, "_xprop", return_value="no such atom"):
            with self.assertRaisesRegex(input_test.PrerequisiteError, "no EWMH window manager"):
                input_test.check_prerequisites(
                    "x11", system="Linux", environ={"DISPLAY": ":99"}
                )

    def test_wayland_requires_a_nested_weston_window(self) -> None:
        responses = iter(
            [
                "_NET_SUPPORTING_WM_CHECK(WINDOW): window id # 0x12",
                "_NET_CLIENT_LIST(WINDOW): window id # 0x34",
                'WM_CLASS(STRING) = "other", "other"',
            ]
        )
        with tempfile.TemporaryDirectory() as runtime_dir:
            socket_path = Path(runtime_dir) / "wayland-test"
            with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
                server.bind(str(socket_path))
                server.listen(1)
                environment = {
                    "DISPLAY": ":99",
                    "XDG_RUNTIME_DIR": runtime_dir,
                    "WAYLAND_DISPLAY": "wayland-test",
                }
                with patch.object(
                    input_test, "_xprop", side_effect=lambda *_: next(responses)
                ):
                    with self.assertRaisesRegex(
                        input_test.PrerequisiteError, "Weston X11-backend"
                    ):
                        input_test.check_prerequisites(
                            "wayland", system="Linux", environ=environment
                        )

    def test_cocoa_requires_event_posting_access_when_supported(self) -> None:
        with patch.object(input_test, "_cocoa_post_event_access", return_value=False):
            with self.assertRaisesRegex(input_test.PrerequisiteError, "Accessibility permission"):
                input_test.check_prerequisites("cocoa", system="Darwin", environ={})

    def test_win32_requires_an_accessible_input_desktop(self) -> None:
        with patch.object(input_test, "_windows_input_desktop_access", return_value=False):
            with self.assertRaisesRegex(input_test.PrerequisiteError, "interactive Windows"):
                input_test.check_prerequisites("win32", system="Windows", environ={})

    def test_runs_only_the_selected_backend_contract(self) -> None:
        with (
            patch.object(input_test, "check_prerequisites") as check,
            patch.object(
                input_test.subprocess, "run", return_value=SimpleNamespace(returncode=0)
            ) as run,
            patch("builtins.print"),
        ):
            self.assertEqual(input_test.run_input_test("x11"), 0)

        check.assert_called_once_with("x11")
        command = run.call_args.args[0]
        self.assertIn("selected_backend_contract", command)
        self.assertIn("--exact", command)
        self.assertEqual(run.call_args.kwargs["env"]["VMNL_PLATFORM_TEST_BACKEND"], "x11")

    def test_vmnl_scenario_rejects_unimplemented_backend(self) -> None:
        with self.assertRaisesRegex(input_test.PrerequisiteError, "currently supports only X11"):
            input_test.run_vmnl_input_test("wayland")

    def test_runs_only_the_public_vmnl_gpu_scenario(self) -> None:
        with (
            patch.object(input_test, "check_prerequisites") as check,
            patch.object(
                input_test.subprocess, "run", return_value=SimpleNamespace(returncode=0)
            ) as run,
            patch("builtins.print"),
        ):
            self.assertEqual(input_test.run_vmnl_input_test("x11"), 0)

        check.assert_called_once_with("x11")
        command = run.call_args.args[0]
        self.assertIn("vmnl-gpu-tests", command)
        self.assertIn("native_input", command)
        self.assertIn("vmnl_public_native_keyboard_and_mouse_events_update_input", command)
        self.assertIn("--ignored", command)
        self.assertIn("--exact", command)


if __name__ == "__main__":
    unittest.main()
