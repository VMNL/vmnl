# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest


MODULE_PATH = Path(__file__).parents[1] / "api_docs.py"
SPEC = importlib.util.spec_from_file_location("api_docs", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
api_docs = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = api_docs
SPEC.loader.exec_module(api_docs)
FIXTURE = Path(__file__).parent / "fixtures" / "public_api.txt"


class SurfaceTests(unittest.TestCase):
    def test_parses_reexports_generics_methods_variants_and_macros(self) -> None:
        surface = api_docs.parse_surface(FIXTURE.read_text(encoding="utf-8"))

        self.assertIn(api_docs.PublicItem("struct", "vmnl::Context"), surface.items)
        self.assertIn(api_docs.PublicItem("struct", "vmnl::raw::Geometry"), surface.items)
        self.assertIn(api_docs.PublicItem("macro", "vmnl::raw::Vertex"), surface.items)
        self.assertIn(api_docs.PublicItem("trait", "vmnl::raw::Vertex"), surface.items)
        self.assertIn(api_docs.PublicItem("alias", "vmnl::VMNLResult"), surface.items)
        self.assertEqual(
            surface.methods,
            (
                api_docs.PublicMethod("vmnl::Context", "new"),
                api_docs.PublicMethod("vmnl::raw::Geometry", "builder"),
            ),
        )

    def test_rejects_unknown_inventory_format(self) -> None:
        with self.assertRaisesRegex(api_docs.ApiDocsError, "unknown"):
            api_docs.parse_surface("pub bewildering vmnl::Thing\n")

    def test_parses_facade_reexports(self) -> None:
        exports = api_docs.parse_facade_exports(FIXTURE.read_text(encoding="utf-8"))

        self.assertEqual(exports, ("vmnl::Context",))


class MatrixTests(unittest.TestCase):
    def write_matrix(self, root: Path, rows: str) -> Path:
        matrix = root / "coverage_matrix.md"
        matrix.write_text(
            "# Matrix\n\n| Kind | Symbol | Canonical page | Rustdoc | Evidence |\n"
            "|---|---|---|---|---|\n" + rows,
            encoding="utf-8",
        )
        return matrix

    def test_rejects_duplicate(self) -> None:
        with tempfile.TemporaryDirectory() as value:
            root = Path(value)
            rows = (
                "| struct | `vmnl::Context` | [Context](context.md) | [Rustdoc](context.html) | test |\n"
                "| struct | `vmnl::Context` | [Context](context.md) | [Rustdoc](context.html) | test |\n"
            )
            with self.assertRaisesRegex(api_docs.ApiDocsError, "duplicate"):
                api_docs.parse_matrix(self.write_matrix(root, rows))

    def test_rejects_missing_page_and_anchor(self) -> None:
        with tempfile.TemporaryDirectory() as value:
            root = Path(value)
            rows = api_docs.parse_matrix(
                self.write_matrix(
                    root,
                    "| struct | `vmnl::Context` | [Context](missing.md#new) | [Rustdoc](context.html) | test |\n",
                )
            )
            with self.assertRaisesRegex(api_docs.ApiDocsError, "missing canonical page"):
                api_docs.validate_matrix_files(rows, root / "coverage_matrix.md")

            (root / "missing.md").write_text("# Context\n", encoding="utf-8")
            with self.assertRaisesRegex(api_docs.ApiDocsError, "missing page anchor"):
                api_docs.validate_matrix_files(rows, root / "coverage_matrix.md")

    def test_rejects_missing_method_anchor(self) -> None:
        with tempfile.TemporaryDirectory() as value:
            root = Path(value)
            rustdoc = root / "context.html"
            rustdoc.write_text('<main id="top"></main>', encoding="utf-8")
            row = api_docs.CoverageRow(
                api_docs.PublicItem("struct", "vmnl::Context"),
                "Context",
                "context.md",
                "Rustdoc",
                str(rustdoc),
                "test",
            )
            surface = api_docs.Surface(
                "surface\n",
                (row.item,),
                (api_docs.PublicMethod("vmnl::Context", "new"),),
            )
            old_matrix = api_docs.MATRIX_PATH
            try:
                api_docs.MATRIX_PATH = root / "coverage_matrix.md"
                with self.assertRaisesRegex(api_docs.ApiDocsError, "method anchor"):
                    api_docs.validate_method_anchors(surface, (row,))
            finally:
                api_docs.MATRIX_PATH = old_matrix


if __name__ == "__main__":
    unittest.main()
