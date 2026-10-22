# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT
"""Repeat the existing Linux joystick scenario and reject incomplete JSON evidence."""

from __future__ import annotations

import argparse
import json
import os
import platform
import signal
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Pin the acceptance workload independently of the driver's self-reported plan.
WORKLOAD_VERSION = 2
COMMON_SCENARIOS_V1 = frozenset({
    "Connect and identify unmapped device",
    "Controlled mapping",
    "Late subscriber does not replay connection",
    "Six distinct axis values",
    "Six alternate axis values",
    "All axes at minimum",
    "All axes at maximum",
    "Map hat to gamepad Dpad",
    "Restore button Dpad mapping",
    "Neutral raw state",
    "Hold mapped A",
    "Replace mapping with A/B permutation",
    "Mapping unavailable with raw button held",
    "Restore mapping while raw button held",
    "Disconnect while held",
    "Next absent batch clears release",
    "Reconnect by identity without slot assumption",
    "Reconnected mapped press",
    "Reconnected mapped release",
    "Disconnect released fixture",
} | {f"Button {index} {action}" for index in range(15) for action in ("press", "release")}
  | {f"Hat {position}" for position in (
      "Centered", "Up", "Right", "Down", "Left", "Right up", "Right down", "Left up", "Left down")})
WINDOW_SCENARIOS_V2 = frozenset({
    "Disable tracking immediately",
    "Re-enable tracking without invented press",
    "Disable event delivery independently",
    "Enable delivery without replay",
    "Window-first connection fan-out",
    "Window-first disconnection fan-out",
    "Stick builder configurations",
    "Stick left setter isolation",
    "Stick right setter isolation",
    "Stick paired setter isolation",
    "Stick dead-zone boundaries",
    "Stick diagonal angles without rescaling",
    "Stick neutral and configuration reset",
})


def expected_workload(backend: str, mode: str) -> set[tuple[str, str]]:
    scenarios = COMMON_SCENARIOS_V1
    if mode == "window":
        scenarios |= WINDOW_SCENARIOS_V2
    checks = {(name, "Observation") for name in scenarios}
    checks.update({("Environment", "Readiness"), ("Environment", "Client shutdown")})
    if backend == "wayland":
        checks.add(("Environment", "Wayland roundtrip"))
    return checks


def validate_report(report: object, backend: str, mode: str) -> int:
    """CI accepts only complete, unexcluded observations from all four profiles."""
    if not isinstance(report, dict) or (
        type(report.get("schema_version")) is not int or report.get("schema_version") != 1
        or report.get("qualified") is not True
        or report.get("evidence") != "linux-uinput"
        or report.get("requested_backend") != backend
        or report.get("scenario") != mode
        or report.get("fixture_guid") != "030000000df000009400000091000000"
    ):
        raise ValueError("incorrect or unqualified joystick report metadata")
    profiles = {
        f"{mode} / hat_buttons={hats} / order={order}"
        for hats in ("false", "true") for order in ("ab", "ba")
    }
    checks = report.get("checks")
    if not isinstance(checks, list) or not checks:
        raise ValueError("joystick report has no checks")
    identities: set[tuple[str, str, str]] = set()
    for check in checks:
        if not isinstance(check, dict) or check.get("status") != "PASS":
            raise ValueError("joystick report contains failed, excluded or unexecuted checks")
        identity = tuple(check.get(key) for key in ("section", "input", "check"))
        if not all(isinstance(value, str) and value for value in identity):
            raise ValueError("joystick check has an invalid identity")
        if identity[0] not in profiles or identity in identities:
            raise ValueError("joystick check has an unexpected profile or duplicate identity")
        identities.add(identity)
    expected = expected_workload(backend, mode)
    for profile in profiles:
        actual = {(name, check) for section, name, check in identities if section == profile}
        if actual != expected:
            raise ValueError(
                f"joystick workload v{WORKLOAD_VERSION} mismatch: {profile}; "
                f"missing={sorted(expected - actual)!r}; unexpected={sorted(actual - expected)!r}")
    observations = report.get("backend_observations")
    if not isinstance(observations, list):
        raise ValueError("missing backend observations array")
    if backend == "wayland":
        observed_profiles: set[str] = set()
        pids: set[int] = set()
        for observation in observations:
            if not isinstance(observation, dict):
                raise ValueError("invalid Wayland observation")
            profile = observation.get("profile")
            pid = observation.get("client_pid")
            protocol = observation.get("protocol")
            if (not isinstance(profile, str) or profile not in profiles
                    or profile in observed_profiles or type(pid) is not int or pid <= 0
                    or pid in pids or observation.get("observed_backend") != "wayland"
                    or not isinstance(protocol, list) or len(protocol) != 3
                    or not all(isinstance(line, str) and line.strip() for line in protocol)):
                raise ValueError("incomplete, duplicate or incorrect Wayland backend evidence")
            observed_profiles.add(profile)
            pids.add(pid)
        if observed_profiles != profiles:
            raise ValueError("missing independent Wayland evidence for a client profile")
    elif observations:
        raise ValueError("unexpected Wayland evidence in an X11 report")
    return len(checks)


def run_scenario(command: list[str], environment: dict[str, str], log: Path,
                 timeout: int) -> int:
    with log.open("w", encoding="utf-8") as output:
        with subprocess.Popen(command, cwd=ROOT, env=environment, stdout=output,
                              stderr=subprocess.STDOUT, start_new_session=True) as process:
            try:
                return process.wait(timeout=timeout)
            except (subprocess.TimeoutExpired, KeyboardInterrupt):
                # Kill the whole driver/client group; closing uinput destroys its owned device.
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
                raise


def run(backend: str, mode: str, directory: Path, repetitions: int, timeout: int) -> int:
    if platform.system() != "Linux":
        raise ValueError("native joystick CI requires Linux")
    if backend not in ("x11", "wayland") or mode not in ("runtime", "window"):
        raise ValueError("invalid native joystick backend or mode")
    if repetitions < 1 or timeout < 1:
        raise ValueError("repetitions and timeout must be positive")
    directory = directory.resolve()
    directory.mkdir(parents=True, exist_ok=False)
    summary = {
        "schema_version": 1, "evidence": "linux-uinput", "qualified": False,
        "workload_version": WORKLOAD_VERSION,
        "requested_backend": backend, "scenario": mode, "repetitions": repetitions,
        "source_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "source_dirty": bool(subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=ROOT, text=True).strip()),
        "host": {"os": platform.system(), "arch": platform.machine(),
                 "kernel": platform.release()},
        "environment": {key: os.environ.get(key) for key in (
            "DISPLAY", "WAYLAND_DISPLAY", "XDG_SESSION_TYPE", "VK_DRIVER_FILES",
            "ImageOS", "ImageVersion", "RUNNER_OS", "RUNNER_ARCH")},
        "runs": [],
    }
    summary_path = directory / "summary.json"
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    try:
        for repetition in range(1, repetitions + 1):
            report_path = directory / f"run-{repetition}.json"
            log = directory / f"run-{repetition}.log"
            environment = dict(os.environ, VMNL_JOYSTICK_REPORT=str(report_path))
            command = [sys.executable, str(ROOT / "tools/input_test.py"),
                       "--joystick", mode, backend]
            result = {"repetition": repetition, "command": command,
                      "report": report_path.name, "log": log.name}
            summary["runs"].append(result)
            result["exit_code"] = run_scenario(command, environment, log, timeout)
            if result["exit_code"] != 0:
                raise ValueError(f"joystick scenario exited {result['exit_code']}; see {log}")
            result["passed"] = validate_report(
                json.loads(report_path.read_text(encoding="utf-8")), backend, mode)
            summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
            print(f"{backend}/{mode} {repetition}/{repetitions}: "
                  f"{result['passed']} PASS; 0 FAIL/EXCLUDED/NOT RUN", flush=True)
        summary["qualified"] = True
    except (OSError, ValueError, subprocess.SubprocessError, KeyboardInterrupt) as error:
        summary["error"] = str(error) or type(error).__name__
        print(f"Native joystick CI failed: {summary['error']}", file=sys.stderr)
    finally:
        summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    return 0 if summary["qualified"] else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("backend", choices=("x11", "wayland"))
    parser.add_argument("mode", choices=("runtime", "window"))
    parser.add_argument("artifact_directory", type=Path)
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--timeout", type=int, default=300, help="seconds per repetition")
    args = parser.parse_args()
    if args.repetitions < 1 or args.timeout < 1:
        parser.error("repetitions and timeout must be positive")
    try:
        return run(args.backend, args.mode, args.artifact_directory,
                   args.repetitions, args.timeout)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Native joystick CI preflight failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
