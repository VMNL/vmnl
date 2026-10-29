# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

import copy
import contextlib
import importlib.util
import io
import json
import signal
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

SCRIPT = Path(__file__).parents[1] / "joystick_ci.py"
SPEC = importlib.util.spec_from_file_location("joystick_ci", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
joystick_ci = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(joystick_ci)


def report(backend="x11", mode="runtime"):
    result = {"schema_version": 1, "qualified": True, "evidence": "linux-uinput",
              "scenario": mode, "requested_backend": backend,
              "fixture_guid": "030000000df000009400000091000000",
              "checks": [], "backend_observations": []}
    for index, (hats, order) in enumerate(
            (hats, order) for hats in ("false", "true") for order in ("ab", "ba")):
        profile = f"{mode} / hat_buttons={hats} / order={order}"
        operations = sorted(joystick_ci.expected_workload(backend, mode))
        if backend == "wayland":
            result["backend_observations"].append({
                "profile": profile, "client_pid": index + 1, "observed_backend": "wayland",
                "protocol": ["registry request", "registry global", "callback done"],
            })
        result["checks"].extend({"section": profile, "input": name, "check": check,
                                 "status": "PASS", "reason": None}
                                for name, check in operations)
    return result


class JoystickCIReportTests(unittest.TestCase):
    def test_accepts_both_modes_and_profiles(self):
        for backend in ("x11", "wayland"):
            for mode in ("runtime", "window"):
                with self.subTest(backend=backend, mode=mode):
                    data = report(backend, mode)
                    self.assertEqual(joystick_ci.validate_report(data, backend, mode),
                                     {("x11", "runtime"): 244, ("x11", "window"): 296,
                                      ("wayland", "runtime"): 248,
                                      ("wayland", "window"): 300}[backend, mode])

    def test_rejects_each_check_omitted_from_every_profile(self):
        for backend in ("x11", "wayland"):
            for mode in ("runtime", "window"):
                baseline = report(backend, mode)
                identities = {(c["input"], c["check"]) for c in baseline["checks"]}
                for identity in identities:
                    data = copy.deepcopy(baseline)
                    data["checks"] = [c for c in data["checks"]
                                      if (c["input"], c["check"]) != identity]
                    with self.subTest(backend=backend, mode=mode, identity=identity):
                        with self.assertRaises(ValueError):
                            joystick_ci.validate_report(data, backend, mode)

    def test_rejects_unknown_checks_even_with_unchanged_counts(self):
        for backend in ("x11", "wayland"):
            for mode in ("runtime", "window"):
                for replace in (False, True):
                    data = report(backend, mode)
                    for profile in {c["section"] for c in data["checks"]}:
                        check = next(c for c in data["checks"]
                                     if c["section"] == profile and c["input"] == "Button 0 press")
                        unknown = dict(check, input="Unknown scenario")
                        if replace:
                            check.update(unknown)
                        else:
                            data["checks"].append(unknown)
                    with self.subTest(backend=backend, mode=mode, replace=replace):
                        with self.assertRaises(ValueError):
                            joystick_ci.validate_report(data, backend, mode)

    def test_window_workload_two_rejects_reports_without_stick_observations(self):
        for backend in ("x11", "wayland"):
            data = report(backend, "window")
            data["checks"] = [c for c in data["checks"] if not c["input"].startswith("Stick ")]
            with self.subTest(backend=backend), self.assertRaisesRegex(ValueError, "workload v2 mismatch"):
                joystick_ci.validate_report(data, backend, "window")

    def test_rejects_incomplete_or_incorrect_metadata(self):
        for key, value in (("schema_version", 2), ("schema_version", True),
                           ("qualified", False), ("qualified", 1),
                           ("evidence", "null"), ("scenario", "window"),
                           ("fixture_guid", "unrelated fixture"),
                           ("requested_backend", "wayland"), ("checks", []),
                           ("checks", None), ("backend_observations", None)):
            with self.subTest(key=key, value=value):
                data = report()
                data[key] = value
                with self.assertRaises(ValueError):
                    joystick_ci.validate_report(data, "x11", "runtime")

    def test_rejects_failed_excluded_missing_duplicate_and_malformed_checks(self):
        for status in ("FAIL", "EXCLUDED", "NOT RUN", "UNKNOWN"):
            data = report()
            data["checks"][0]["status"] = status
            with self.subTest(status=status), self.assertRaises(ValueError):
                joystick_ci.validate_report(data, "x11", "runtime")
        missing = report()
        missing["checks"].pop()
        duplicate = report()
        duplicate["checks"].append(duplicate["checks"][0])
        malformed = report()
        malformed["checks"][0]["section"] = []
        wrong_profile = report()
        wrong_profile["checks"][0]["section"] = "unexpected"
        no_scenario = report()
        no_scenario["checks"] = [c for c in no_scenario["checks"] if c["check"] != "Observation"]
        for data in (missing, duplicate, malformed, wrong_profile, no_scenario):
            with self.subTest(data=data), self.assertRaises(ValueError):
                joystick_ci.validate_report(data, "x11", "runtime")

    def test_wayland_requires_independent_evidence_for_every_profile(self):
        baseline = report("wayland")
        variants = []
        for key, value in (("profile", "unexpected"), ("client_pid", 0),
                           ("client_pid", True), ("observed_backend", "x11"),
                           ("protocol", []), ("protocol", ["", "global", "done"])):
            data = copy.deepcopy(baseline)
            data["backend_observations"][0][key] = value
            variants.append(data)
        missing = copy.deepcopy(baseline)
        missing["backend_observations"].pop()
        duplicate = copy.deepcopy(baseline)
        duplicate["backend_observations"][1] = duplicate["backend_observations"][0]
        for data in (*variants, missing, duplicate):
            with self.subTest(data=data), self.assertRaises(ValueError):
                joystick_ci.validate_report(data, "wayland", "runtime")

    def test_profiles_must_execute_the_same_workload(self):
        data = report()
        extra = dict(data["checks"][1], input="Axes")
        data["checks"].append(extra)
        with self.assertRaisesRegex(ValueError, "workload v2 mismatch"):
            joystick_ci.validate_report(data, "x11", "runtime")


class JoystickCIRunnerTests(unittest.TestCase):
    def run_fixture(self, directory, code=0, data=None, error=None):
        def execute(command, environment, log, timeout):
            self.assertEqual(command[-3:], ["--joystick", "runtime", "x11"])
            self.assertEqual(timeout, 30)
            log.write_text("fixture output", encoding="utf-8")
            if error is not None:
                raise error
            if data is not None:
                Path(environment["VMNL_JOYSTICK_REPORT"]).write_text(json.dumps(data))
            return code
        with (patch.object(joystick_ci.platform, "system", return_value="Linux"),
              patch.object(joystick_ci.subprocess, "check_output", return_value="commit\n"),
              patch.object(joystick_ci, "run_scenario", side_effect=execute) as launch,
              contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO())):
            result = joystick_ci.run("x11", "runtime", directory, 3, 30)
        return result, json.loads((directory / "summary.json").read_text()), launch.call_count

    def test_records_every_repetition_and_uses_new_report_paths(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "records"
            result, summary, calls = self.run_fixture(directory, data=report())
            self.assertEqual((result, calls, summary["qualified"]), (0, 3, True))
            self.assertEqual(summary["workload_version"], 2)
            self.assertEqual([r["report"] for r in summary["runs"]],
                             ["run-1.json", "run-2.json", "run-3.json"])

    def test_stops_on_failure_even_with_a_passing_json(self):
        for code, data in ((9, report()), (0, None), (0, {})):
            with self.subTest(code=code, data=data), tempfile.TemporaryDirectory() as temporary:
                result, summary, calls = self.run_fixture(Path(temporary) / "records", code, data)
                self.assertEqual((result, calls, summary["qualified"]), (1, 1, False))
                self.assertIn("error", summary)

    def test_zero_exit_with_a_common_omission_stops_repetitions(self):
        data = report()
        data["checks"] = [c for c in data["checks"] if c["input"] != "Button 0 press"]
        with tempfile.TemporaryDirectory() as temporary:
            result, summary, calls = self.run_fixture(Path(temporary) / "records", data=data)
            self.assertEqual((result, calls, summary["qualified"]), (1, 1, False))
            self.assertIn("Button 0 press", summary["error"])
            self.assertNotIn("passed", summary["runs"][0])

    def test_refuses_an_existing_artifact_directory(self):
        with tempfile.TemporaryDirectory() as temporary:
            with patch.object(joystick_ci.platform, "system", return_value="Linux"):
                with self.assertRaises(FileExistsError):
                    joystick_ci.run("x11", "runtime", Path(temporary), 3, 30)

    def test_timeout_or_interruption_leaves_an_unqualified_summary(self):
        for error in (subprocess.TimeoutExpired("fixture", 30), KeyboardInterrupt()):
            with self.subTest(error=error), tempfile.TemporaryDirectory() as temporary:
                result, summary, calls = self.run_fixture(Path(temporary) / "records", error=error)
                self.assertEqual((result, calls, summary["qualified"]), (1, 1, False))
                self.assertIn("error", summary)

    def test_invalid_configuration_never_launches_a_scenario(self):
        with (tempfile.TemporaryDirectory() as temporary,
              patch.object(joystick_ci.platform, "system", return_value="Linux"),
              patch.object(joystick_ci, "run_scenario") as launch):
            for backend, mode, repetitions, timeout in (
                    ("null", "runtime", 3, 30), ("x11", "invalid", 3, 30),
                    ("x11", "runtime", 0, 30), ("x11", "runtime", 3, 0)):
                with self.assertRaises(ValueError):
                    joystick_ci.run(backend, mode, Path(temporary) / "unused", repetitions, timeout)
            launch.assert_not_called()

    def test_timeout_terminates_the_driver_and_client_group(self):
        with tempfile.TemporaryDirectory() as temporary:
            process = MagicMock()
            process.pid = 123
            process.wait.side_effect = [subprocess.TimeoutExpired("fixture", 30), 0]
            with (patch.object(joystick_ci.subprocess, "Popen") as popen,
                  patch.object(joystick_ci.os, "killpg") as kill):
                popen.return_value.__enter__.return_value = process
                with self.assertRaises(subprocess.TimeoutExpired):
                    joystick_ci.run_scenario(["fixture"], {}, Path(temporary) / "log", 30)
                kill.assert_called_once_with(123, signal.SIGTERM)
                self.assertTrue(popen.call_args.kwargs["start_new_session"])


if __name__ == "__main__":
    unittest.main()
