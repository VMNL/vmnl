# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

import importlib.util
import json
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
        artifact = json.dumps({
            "reason": "compiler-artifact", "target": {"name": "vmnl-input-client"},
            "executable": "/custom-target/debug/vmnl-input-client",
        })
        with (
            patch.object(input_test, "check_prerequisites") as check,
            patch.object(
                input_test.subprocess, "run", side_effect=[
                    SimpleNamespace(returncode=0, stdout=artifact),
                    SimpleNamespace(returncode=0),
                ]
            ) as run,
            patch("builtins.print"),
        ):
            self.assertEqual(input_test.run_vmnl_input_test("x11"), 0)

        check.assert_called_once_with("x11")
        self.assertIn("vmnl-input-client", run.call_args_list[0].args[0])
        command = run.call_args.args[0]
        self.assertIn("vmnl-gpu-tests", command)
        self.assertIn("native_input", command)
        self.assertIn("vmnl_public_native_keyboard_and_mouse_events_update_input", command)
        self.assertIn("--ignored", command)
        self.assertIn("--exact", command)
        self.assertEqual(
            run.call_args.kwargs["env"]["VMNL_INPUT_CLIENT"],
            "/custom-target/debug/vmnl-input-client",
        )

    def test_vmnl_client_build_failure_does_not_start_injection(self) -> None:
        with (
            patch.object(input_test, "check_prerequisites"),
            patch.object(input_test.subprocess, "run", return_value=SimpleNamespace(returncode=101)) as run,
            patch("builtins.print"),
        ):
            self.assertEqual(input_test.run_vmnl_input_test("x11"), 101)
        run.assert_called_once()

    def test_vmnl_client_artifact_rejects_missing_duplicate_or_malformed_records(self) -> None:
        artifact = json.dumps({
            "reason": "compiler-artifact", "target": {"name": "vmnl-input-client"},
            "executable": "/target/debug/vmnl-input-client",
        })
        for output in ["", "not JSON", "[]", artifact + "\n" + artifact]:
            with self.subTest(output=output), self.assertRaises(input_test.PrerequisiteError):
                input_test.vmnl_client_executable(output)

    def test_joystick_missing_access_does_not_build_or_inject(self) -> None:
        with (
            patch.object(input_test.platform, "system", return_value="Linux"),
            patch.object(input_test.os, "open", side_effect=PermissionError("denied")),
            patch.object(input_test.subprocess, "run") as run,
        ):
            with self.assertRaisesRegex(input_test.PrerequisiteError, "uinput/input access"):
                input_test.run_joystick_test("x11", "runtime")
        run.assert_not_called()

    def test_joystick_is_opt_in_and_other_hosts_remain_unqualified(self) -> None:
        for backend, host in [("win32", "Windows"), ("cocoa", "Darwin"), ("x11", "Darwin")]:
            with self.subTest(backend=backend, host=host), patch.object(input_test.platform, "system", return_value=host):
                with self.assertRaisesRegex(input_test.PrerequisiteError, "Linux-only"):
                    input_test.check_joystick_prerequisites(backend)

    def test_joystick_launches_only_requested_public_path(self) -> None:
        artifact = json.dumps({
            "reason": "compiler-artifact", "target": {"name": "vmnl-joystick-client"},
            "executable": "/custom-target/debug/vmnl-joystick-client",
        })
        for mode, crate in [("runtime", "vmnl-platform-tests"), ("window", "vmnl-gpu-tests")]:
            with (
                self.subTest(mode=mode),
                patch.object(input_test, "check_joystick_prerequisites"),
                patch.object(input_test.subprocess, "run", side_effect=[
                    SimpleNamespace(returncode=0, stdout=artifact), SimpleNamespace(returncode=17),
                ]) as run,
            ):
                self.assertEqual(input_test.run_joystick_test("x11", mode), 17)
                command = run.call_args.args[0]
                self.assertIn(crate, command)
                self.assertIn("native_joystick", command)
                self.assertIn("--ignored", command)
                self.assertIn("--exact", command)
                env = run.call_args.kwargs["env"]
                self.assertEqual(env["VMNL_JOYSTICK_CLIENT"], "/custom-target/debug/vmnl-joystick-client")
                self.assertEqual(env["XDG_SESSION_TYPE"], "x11")
                self.assertNotIn("WAYLAND_DISPLAY", env)

    def test_joystick_build_failure_never_starts_the_driver(self) -> None:
        with (
            patch.object(input_test, "check_joystick_prerequisites"),
            patch.object(input_test.subprocess, "run", return_value=SimpleNamespace(returncode=101)) as run,
        ):
            self.assertEqual(input_test.run_joystick_test("x11", "runtime"), 101)
        run.assert_called_once()


if __name__ == "__main__":
    unittest.main()
