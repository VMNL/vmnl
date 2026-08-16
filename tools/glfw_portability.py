#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT
"""Validate and generate VMNL's GLFW portability inventory."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

ALLOWED_STATUSES = {
    "supported",
    "unsupported",
    "conditional",
    "best-effort",
    "unverified",
}
ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "tools/data/glfw_platform_inventory.toml"
GENERATED_INVENTORY = ROOT / "docs/api/maintenance/glfw_platform_inventory.md"
GENERATED_PUBLIC = ROOT / "docs/api/reference/window/platform_compatibility.md"

# Rust wrapper calls used by VMNL. Adding one of these calls requires a matching C symbol in the
# inventory. Raw GLFW calls are separately restricted to the private adapter.
RUST_CALL_TO_C = {
    "glfw::init(": "glfwInit",
    ".get_platform(": "glfwGetPlatform",
    ".window_hint(": "glfwWindowHint",
    ".create_window(": "glfwCreateWindow",
    ".get_required_instance_extensions(": "glfwGetRequiredInstanceExtensions",
    ".with_connected_monitors(": "glfwGetMonitors",
    ".get_physical_size(": "glfwGetMonitorPhysicalSize",
    ".get_workarea(": "glfwGetMonitorWorkarea",
    ".get_video_modes(": "glfwGetVideoModes",
    ".get_video_mode(": "glfwGetVideoMode",
    ".get_name(": "glfwGetMonitorName",
    ".get_pos(": "glfwGetWindowPos",
    ".set_pos(": "glfwSetWindowPos",
    ".set_title(": "glfwSetWindowTitle",
    ".set_size(": "glfwSetWindowSize",
    ".get_framebuffer_size(": "glfwGetFramebufferSize",
    ".get_content_scale(": "glfwGetWindowContentScale",
    ".set_size_limits(": "glfwSetWindowSizeLimits",
    ".iconify(": "glfwIconifyWindow",
    ".restore(": "glfwRestoreWindow",
    ".maximize(": "glfwMaximizeWindow",
    ".show(": "glfwShowWindow",
    ".hide(": "glfwHideWindow",
    ".focus(": "glfwFocusWindow",
    ".set_opacity(": "glfwSetWindowOpacity",
    ".get_opacity(": "glfwGetWindowOpacity",
    ".is_iconified(": "glfwGetWindowAttrib",
    ".is_maximized(": "glfwGetWindowAttrib",
    ".is_visible(": "glfwGetWindowAttrib",
    ".is_focused(": "glfwGetWindowAttrib",
    ".set_should_close(": "glfwSetWindowShouldClose",
    ".should_close(": "glfwWindowShouldClose",
    ".poll_events(": "glfwPollEvents",
    ".wait_events(": "glfwWaitEvents",
    ".wait_events_timeout(": "glfwWaitEventsTimeout",
    ".post_empty_event(": "glfwPostEmptyEvent",
    ".get_time(": "glfwGetTime",
    ".set_time(": "glfwSetTime",
    ".get_timer_value(": "glfwGetTimerValue",
    ".get_timer_frequency(": "glfwGetTimerFrequency",
    ".get_key(": "glfwGetKey",
    ".get_mouse_button(": "glfwGetMouseButton",
    ".set_error_callback(": "glfwSetErrorCallback",
    ".unset_error_callback(": "glfwSetErrorCallback",
    ".set_char_polling(": "glfwSetCharCallback",
    ".set_char_mods_polling(": "glfwSetCharModsCallback",
    ".set_close_polling(": "glfwSetWindowCloseCallback",
    ".set_content_scale_polling(": "glfwSetWindowContentScaleCallback",
    ".set_cursor_enter_polling(": "glfwSetCursorEnterCallback",
    ".set_cursor_pos_polling(": "glfwSetCursorPosCallback",
    ".set_drag_and_drop_polling(": "glfwSetDropCallback",
    ".set_focus_polling(": "glfwSetWindowFocusCallback",
    ".set_framebuffer_size_polling(": "glfwSetFramebufferSizeCallback",
    ".set_iconify_polling(": "glfwSetWindowIconifyCallback",
    ".set_key_polling(": "glfwSetKeyCallback",
    ".set_maximize_polling(": "glfwSetWindowMaximizeCallback",
    ".set_mouse_button_polling(": "glfwSetMouseButtonCallback",
    ".set_refresh_polling(": "glfwSetWindowRefreshCallback",
    ".set_scroll_polling(": "glfwSetScrollCallback",
    ".set_size_polling(": "glfwSetWindowSizeCallback",
}


def load_inventory(path: Path = INVENTORY) -> dict[str, Any]:
    with path.open("rb") as source:
        return tomllib.load(source)


def validate_inventory(data: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    audit = data.get("audit")
    if not isinstance(audit, dict):
        return ["missing [audit] table"]
    for field in ("glfw", "glfw_sys", "glfw_c", "fork_url", "fork_rev"):
        if not audit.get(field):
            errors.append(f"audit.{field} is required")

    functions = data.get("function")
    if not isinstance(functions, list) or not functions:
        return errors + ["at least one [[function]] entry is required"]

    seen: set[str] = set()
    for index, entry in enumerate(functions, start=1):
        symbol = entry.get("c_symbol")
        label = symbol or f"entry #{index}"
        if not isinstance(symbol, str) or not symbol.startswith("glfw"):
            errors.append(f"{label}: invalid or missing c_symbol")
            continue
        if symbol in seen:
            errors.append(f"{symbol}: duplicate function entry")
        seen.add(symbol)
        for field in ("rust_wrappers", "category", "introduced", "vmnl_usage"):
            if field not in entry:
                errors.append(f"{symbol}: missing {field}")
        backends = entry.get("backends")
        if not isinstance(backends, list) or not backends:
            errors.append(f"{symbol}: at least one backend record is required")
        else:
            backend_seen: set[str] = set()
            for backend in backends:
                name = backend.get("name")
                status = backend.get("status")
                if not name:
                    errors.append(f"{symbol}: backend name is required")
                elif name in backend_seen:
                    errors.append(f"{symbol}: duplicate backend {name}")
                else:
                    backend_seen.add(name)
                if status not in ALLOWED_STATUSES:
                    errors.append(f"{symbol}/{name}: unknown status {status!r}")
                for field in ("conditions", "behavior", "errors", "sentinel", "source"):
                    if field not in backend or backend[field] in (None, "", []):
                        errors.append(f"{symbol}/{name}: missing {field}")
        proofs = entry.get("proofs")
        if not isinstance(proofs, list) or not proofs:
            errors.append(f"{symbol}: test evidence or explicit justification is required")
    return errors


def inventory_symbols(data: dict[str, Any]) -> set[str]:
    return {entry["c_symbol"] for entry in data.get("function", []) if "c_symbol" in entry}


def validate_source_coverage(data: dict[str, Any], root: Path = ROOT) -> list[str]:
    errors: list[str] = []
    symbols = inventory_symbols(data)
    source_root = root / "crates/vmnl_graphics/src"
    for path in source_root.rglob("*.rs"):
        text = path.read_text(encoding="utf-8")
        relative = path.relative_to(root)
        for rust_call, c_symbol in RUST_CALL_TO_C.items():
            if rust_call in text and c_symbol not in symbols:
                errors.append(f"{relative}: {rust_call} requires inventory entry {c_symbol}")
        if path.name != "glfw_backend.rs" and re.search(r"glfw::ffi::glfw[A-Za-z0-9_]+", text):
            errors.append(f"{relative}: raw runtime GLFW call must be in glfw_backend.rs")
    return errors


def cargo_metadata(root: Path = ROOT) -> dict[str, Any]:
    completed = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode:
        raise RuntimeError(completed.stderr.strip() or "cargo metadata failed")
    return json.loads(completed.stdout)


def validate_dependency_audit(
    data: dict[str, Any], root: Path = ROOT, *, require_fork: bool = True
) -> list[str]:
    errors: list[str] = []
    audit = data["audit"]
    manifest = (root / "Cargo.toml").read_text(encoding="utf-8")
    dependency = re.search(r"(?m)^glfw\s*=\s*\{([^\n]+)\}", manifest)
    if dependency is None:
        return ["workspace glfw dependency is missing"]
    declaration = dependency.group(1)
    expected_url = f'git = "{audit["fork_url"]}"'
    expected_rev = f'rev = "{audit["fork_rev"]}"'
    if require_fork and (expected_url not in declaration or expected_rev not in declaration):
        errors.append("workspace glfw dependency does not match the audited fork URL and revision")
    if '"src-build"' not in declaration or '"prebuilt-libs"' in declaration:
        errors.append("workspace glfw dependency must force the audited bundled C sources")

    try:
        metadata = cargo_metadata(root)
    except RuntimeError as error:
        return errors + [str(error)]
    packages = {package["name"]: package for package in metadata["packages"]}
    versions = {name: package["version"] for name, package in packages.items()}
    if versions.get("glfw") != audit["glfw"]:
        errors.append(f"resolved glfw version is {versions.get('glfw')!r}, expected {audit['glfw']}")
    if versions.get("glfw-sys") != audit["glfw_sys"]:
        errors.append(
            f"resolved glfw-sys version is {versions.get('glfw-sys')!r}, expected {audit['glfw_sys']}"
        )
    glfw_source = packages.get("glfw", {}).get("source") or ""
    if require_fork and (
        audit["fork_url"] not in glfw_source or audit["fork_rev"] not in glfw_source
    ):
        errors.append("resolved glfw source does not match the audited fork revision")

    glfw_sys = packages.get("glfw-sys")
    if glfw_sys is not None:
        manifest = Path(glfw_sys["manifest_path"])
        header = manifest.parent / "glfw/include/GLFW/glfw3.h"
        if not header.exists():
            errors.append(f"bundled GLFW header not found at {header}")
        else:
            errors.extend(validate_glfw_header(header.read_text(encoding="utf-8"), audit))
    return errors


def validate_glfw_header(header: str, audit: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    expected_version = tuple(int(part) for part in audit["glfw_c"].split("."))
    actual_version = tuple(
        _define_integer(header, name)
        for name in ("GLFW_VERSION_MAJOR", "GLFW_VERSION_MINOR", "GLFW_VERSION_REVISION")
    )
    if actual_version != expected_version:
        errors.append(
            f"bundled GLFW header version is {actual_version}, expected {expected_version}"
        )

    expected_codes = {
        "GLFW_NO_ERROR": 0,
        "GLFW_NOT_INITIALIZED": 0x00010001,
        "GLFW_NO_CURRENT_CONTEXT": 0x00010002,
        "GLFW_INVALID_ENUM": 0x00010003,
        "GLFW_INVALID_VALUE": 0x00010004,
        "GLFW_OUT_OF_MEMORY": 0x00010005,
        "GLFW_API_UNAVAILABLE": 0x00010006,
        "GLFW_VERSION_UNAVAILABLE": 0x00010007,
        "GLFW_PLATFORM_ERROR": 0x00010008,
        "GLFW_FORMAT_UNAVAILABLE": 0x00010009,
        "GLFW_NO_WINDOW_CONTEXT": 0x0001000A,
        "GLFW_CURSOR_UNAVAILABLE": 0x0001000B,
        "GLFW_FEATURE_UNAVAILABLE": 0x0001000C,
        "GLFW_FEATURE_UNIMPLEMENTED": 0x0001000D,
        "GLFW_PLATFORM_UNAVAILABLE": 0x0001000E,
    }
    for name, expected in expected_codes.items():
        actual = _define_integer(header, name)
        if actual != expected:
            errors.append(f"bundled GLFW constant {name} is {actual!r}, expected {expected}")
    return errors


def _define_integer(header: str, name: str) -> int | None:
    match = re.search(rf"(?m)^#define\s+{re.escape(name)}\s+([^\s/]+)", header)
    if match is None:
        return None
    try:
        return int(match.group(1), 0)
    except ValueError:
        return None


def render_inventory(data: dict[str, Any]) -> str:
    audit = data["audit"]
    lines = [
        "<!-- Generated by tools/glfw_portability.py. Do not edit. -->",
        "# GLFW platform inventory",
        "",
        f"Audited surface: `glfw {audit['glfw']}`, `glfw-sys {audit['glfw_sys']}`, "
        f"GLFW C `{audit['glfw_c']}`, fork `{audit['fork_rev']}`.",
        "",
        "A status is scoped to the named backend and stated conditions. `unverified` is not a "
        "compatibility guarantee.",
        "",
        "| C symbol | Rust wrappers | VMNL usage | Backend status | Evidence |",
        "|---|---|---|---|---|",
    ]
    for entry in sorted(data["function"], key=lambda item: item["c_symbol"]):
        backends = "; ".join(
            f"{item['name']}: {item['status']} ({item['behavior']})" for item in entry["backends"]
        )
        lines.append(
            "| `{}` | {} | {} | {} | {} |".format(
                entry["c_symbol"],
                ", ".join(f"`{item}`" for item in entry["rust_wrappers"]) or "C API only",
                ", ".join(f"`{item}`" for item in entry["vmnl_usage"]) or "Not used",
                backends.replace("|", "\\|"),
                ", ".join(entry["proofs"]),
            )
        )
    lines.extend(["", "Sources are stored per backend in the canonical TOML inventory.", ""])
    return "\n".join(lines)


def render_public(data: dict[str, Any]) -> str:
    entries = [
        entry
        for entry in data["function"]
        if entry.get("public_api", entry["vmnl_usage"])
    ]
    lines = [
        "<!-- Generated by tools/glfw_portability.py. Do not edit. -->",
        "# Window platform compatibility",
        "",
        "This matrix covers VMNL's public window API. It reports measured or documented behavior, "
        "not a universal guarantee for every compositor, window manager, or runner image.",
        "",
        "Distinct outcomes must not be conflated: a callback reports an error; a no-op leaves state "
        "unchanged; a sentinel is a fallback getter value.",
        "",
        "| VMNL API | GLFW operation | Backend status |",
        "|---|---|---|",
    ]
    for entry in sorted(
        entries,
        key=lambda item: (item.get("public_api", item["vmnl_usage"])[0], item["c_symbol"]),
    ):
        public_api = entry.get("public_api", entry["vmnl_usage"])
        status = "; ".join(
            f"{item['name']}: {item['status']} — {item['behavior']}" for item in entry["backends"]
        )
        lines.append(
            "| {} | `{}` | {} |".format(
                ", ".join(f"`{item}`" for item in public_api),
                entry["c_symbol"],
                status.replace("|", "\\|"),
            )
        )
    lines.extend(
        [
            "",
            "## Interpretation rules",
            "",
            "- Window positions use GLFW screen coordinates; they are not necessarily physical pixels.",
            "- Wayland does not expose global window positions. Position setters are no-ops and getters "
            "use `(0, 0)` when GLFW cannot provide a value.",
            "- Wayland window opacity and programmatic iconification are unavailable or compositor-dependent.",
            "- X11 focus, attention, maximization and floating behavior depend on the active window manager.",
            "- `Window::focus` is a request. A normal return never guarantees that focus was granted.",
            "",
        ]
    )
    return "\n".join(lines)


def generated_outputs(data: dict[str, Any]) -> dict[Path, str]:
    return {
        GENERATED_INVENTORY: render_inventory(data),
        GENERATED_PUBLIC: render_public(data),
    }


def update(data: dict[str, Any]) -> None:
    for path, content in generated_outputs(data).items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")


def stale_outputs(data: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    for path, expected in generated_outputs(data).items():
        if not path.exists() or path.read_text(encoding="utf-8") != expected:
            errors.append(f"generated file is missing or stale: {path.relative_to(ROOT)}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("check", "update"))
    parser.add_argument(
        "--skip-dependency-audit",
        action="store_true",
        help="development-only escape hatch while preparing an unpublished fork commit",
    )
    arguments = parser.parse_args()
    data = load_inventory()
    errors = validate_inventory(data) + validate_source_coverage(data)
    errors.extend(
        validate_dependency_audit(data, require_fork=not arguments.skip_dependency_audit)
    )
    if errors:
        for error in errors:
            print(f"error: {error}", file=sys.stderr)
        return 1
    if arguments.command == "update":
        update(data)
    else:
        errors = stale_outputs(data)
        if errors:
            for error in errors:
                print(f"error: {error}", file=sys.stderr)
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
