# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

import importlib.util
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "glfw_portability.py"
SPEC = importlib.util.spec_from_file_location("glfw_portability", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def fixture() -> dict:
    return {
        "audit": {
            "glfw": "0.62.0",
            "glfw_sys": "8.0.0",
            "glfw_c": "3.4.0",
            "fork_url": "https://github.com/VMNL/glfw-rs",
            "fork_rev": "a" * 40,
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


if __name__ == "__main__":
    unittest.main()
