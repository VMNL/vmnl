# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

import importlib.util
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).parents[1] / "glfw_portability.py"
SPEC = importlib.util.spec_from_file_location("glfw_portability", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def fixture() -> dict:
    return {
        "audit": {
            "glfw": "0.63.0",
            "glfw_sys": "8.0.0",
            "glfw_c": "3.4.0",
            "registry": "registry+https://github.com/rust-lang/crates.io-index",
        },
        "function": [
            {
                "c_symbol": "glfwInit",
                "rust_wrappers": ["glfw::init"],
                "category": "initialization",
                "introduced": "1.0",
                "vmnl_usage": ["Context::new"],
                "backends": [
                    {
                        "name": "all",
                        "status": "conditional",
                        "conditions": "backend available",
                        "behavior": "initializes GLFW",
                        "errors": ["GLFW_PLATFORM_UNAVAILABLE"],
                        "sentinel": "GLFW_FALSE",
                        "source": "https://www.glfw.org/docs/latest/group__init.html",
                    }
                ],
                "proofs": ["test:fixture"],
            }
        ],
    }


class InventoryValidationTests(unittest.TestCase):
    def test_accepts_complete_entry(self) -> None:
        self.assertEqual(MODULE.validate_inventory(fixture()), [])

    def test_rejects_duplicate_symbols(self) -> None:
        data = fixture()
        data["function"].append(data["function"][0].copy())
        self.assertTrue(any("duplicate" in error for error in MODULE.validate_inventory(data)))

    def test_rejects_unknown_status(self) -> None:
        data = fixture()
        data["function"][0]["backends"][0]["status"] = "sometimes"
        self.assertTrue(any("unknown status" in error for error in MODULE.validate_inventory(data)))

    def test_rejects_non_crates_io_registry(self) -> None:
        data = fixture()
        data["audit"]["registry"] = "registry+https://example.com/index"
        self.assertTrue(any("crates.io" in error for error in MODULE.validate_inventory(data)))

    def test_requires_source_and_proof(self) -> None:
        data = fixture()
        data["function"][0]["backends"][0]["source"] = ""
        data["function"][0]["proofs"] = []
        errors = MODULE.validate_inventory(data)
        self.assertTrue(any("missing source" in error for error in errors))
        self.assertTrue(any("justification" in error for error in errors))

    def test_detects_uninventoried_wrapper_call(self) -> None:
        data = fixture()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates/vmnl_graphics/src"
            source.mkdir(parents=True)
            (source / "window.rs").write_text("window.set_pos(1, 2);", encoding="utf-8")
            errors = MODULE.validate_source_coverage(data, root)
        self.assertTrue(any("glfwSetWindowPos" in error for error in errors))

    def test_detects_glfw_header_version_drift(self) -> None:
        data = fixture()
        header = "\n".join(
            [
                "#define GLFW_VERSION_MAJOR 3",
                "#define GLFW_VERSION_MINOR 5",
                "#define GLFW_VERSION_REVISION 0",
            ]
        )
        errors = MODULE.validate_glfw_header(header, data["audit"])
        self.assertTrue(any("header version" in error for error in errors))

    def test_registry_provenance_and_version_are_checked(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace.dependencies]\nglfw = { version = "=0.63.0", '
                'default-features = false, features = ["src-build", "x11", "wayland", '
                '"vulkan", "raw-window-handle-v0-6"] }\n',
                encoding="utf-8",
            )
            sys_root = root / "glfw-sys"
            header = sys_root / "glfw/include/GLFW/glfw3.h"
            header.parent.mkdir(parents=True)
            header.write_text("fixture", encoding="utf-8")
            packages = [
                {"name": "glfw", "version": "0.63.0", "source": fixture()["audit"]["registry"]},
                {
                    "name": "glfw-sys",
                    "version": "8.0.0",
                    "source": fixture()["audit"]["registry"],
                    "manifest_path": str(sys_root / "Cargo.toml"),
                },
            ]
            with mock.patch.object(MODULE, "cargo_metadata", return_value={"packages": packages}), mock.patch.object(
                MODULE, "validate_glfw_header", return_value=[]
            ):
                self.assertEqual(MODULE.validate_dependency_audit(fixture(), root), [])
                packages[0]["source"] = "git+https://example.com/glfw#abc"
                self.assertTrue(any("source" in e for e in MODULE.validate_dependency_audit(fixture(), root)))
                packages[0]["source"] = fixture()["audit"]["registry"]
                packages[0]["version"] = "0.63.1"
                self.assertTrue(any("version" in e for e in MODULE.validate_dependency_audit(fixture(), root)))
                packages[0]["version"] = "0.63.0"
                packages[1]["source"] = "path+file:///tmp/glfw-sys"
                self.assertTrue(
                    any("glfw-sys source" in e for e in MODULE.validate_dependency_audit(fixture(), root))
                )
                packages[1]["source"] = fixture()["audit"]["registry"]
                packages.append(packages[0].copy())
                self.assertTrue(any("multiple" in e for e in MODULE.validate_dependency_audit(fixture(), root)))

    def test_registry_declaration_must_pin_version_and_build_from_source(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text(
                '[workspace.dependencies]\nglfw = { version = "0.63.0", '
                'git = "https://example.com/glfw", features = ["prebuilt-libs"] }\n',
                encoding="utf-8",
            )
            with mock.patch.object(MODULE, "cargo_metadata", side_effect=RuntimeError("fixture")):
                errors = MODULE.validate_dependency_audit(fixture(), root)
            self.assertTrue(any("registry" in e for e in errors))
            self.assertTrue(any("version" in e for e in errors))
            self.assertTrue(any("source" in e for e in errors))


if __name__ == "__main__":
    unittest.main()
