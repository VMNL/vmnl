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

        self.assertIn(api_docs.PublicItem("struct", "vmnl_graphics::Context"), surface.items)
        self.assertIn(api_docs.PublicItem("struct", "vmnl_graphics::raw::Geometry"), surface.items)
        self.assertIn(api_docs.PublicItem("macro", "vmnl_graphics::raw::Vertex"), surface.items)
        self.assertIn(api_docs.PublicItem("trait", "vmnl_graphics::raw::Vertex"), surface.items)
        self.assertIn(api_docs.PublicItem("alias", "vmnl_graphics::VMNLResult"), surface.items)
        self.assertEqual(
            surface.methods,
            (
                api_docs.PublicMethod("vmnl_graphics::Context", "new"),
                api_docs.PublicMethod("vmnl_graphics::raw::Geometry", "builder"),
            ),
        )

    def test_rejects_unknown_inventory_format(self) -> None:
        with self.assertRaisesRegex(api_docs.ApiDocsError, "unknown"):
            api_docs.parse_surface("pub bewildering vmnl_graphics::Thing\n")

    def test_parses_graphics_reexports(self) -> None:
        exports = api_docs.parse_reexports(FIXTURE.read_text(encoding="utf-8"))

        self.assertEqual(exports, ("vmnl_graphics::Context", "vmnl_graphics::raw::Vertex"))

    def test_keeps_graphics_namespace_and_rejects_legacy_namespace(self) -> None:
        surface = api_docs.parse_surface("pub struct vmnl_graphics::Context\n")
        self.assertEqual(surface.items, (api_docs.PublicItem("struct", "vmnl_graphics::Context"),))
        with self.assertRaisesRegex(api_docs.ApiDocsError, "unknown"):
            api_docs.parse_surface("pub struct vmnl::Context\n")

    def test_private_macros_require_actual_graphics_reexports(self) -> None:
        text = FIXTURE.read_text(encoding="utf-8")
        text += "pub proc macro vmnl_macros::#[derive(Pod)]\n"
        surface = api_docs.parse_surface(text)
        with self.assertRaisesRegex(api_docs.ApiDocsError, "not reexported"):
            api_docs.validate_public_reexports(surface)
        surface = api_docs.parse_surface(text + "pub use vmnl_graphics::raw::Pod\n")
        api_docs.validate_public_reexports(surface)

    def test_shader_aliases_share_one_item_and_method_owner(self) -> None:
        surface = api_docs.parse_surface(
            "pub enum vmnl_graphics::common::ShaderSource\n"
            "pub enum vmnl_graphics::ShaderSource\n"
            "pub enum vmnl_graphics::raw::ShaderSource\n"
            "pub fn vmnl_graphics::ShaderSource::clone(&self) -> Self\n"
        )
        self.assertEqual(surface.items, (api_docs.PublicItem("enum", "vmnl_graphics::common::ShaderSource"),))
        self.assertEqual(surface.methods, (api_docs.PublicMethod("vmnl_graphics::common::ShaderSource", "clone"),))

    def test_distinct_types_with_the_same_name_are_not_merged(self) -> None:
        surface = api_docs.parse_surface(
            "pub struct vmnl_graphics::d2::Value\n"
            "pub struct vmnl_graphics::d3::Value\n"
        )
        self.assertEqual(len(surface.items), 2)


class MatrixTests(unittest.TestCase):
    def write_matrix(self, root: Path, rows: str) -> Path:
        matrix = root / "coverage_matrix.md"
        matrix.write_text(
            "# Matrix\n\n| Kind | Symbol | Canonical page | Rustdoc | Evidence |\n"
            "|---|---|---|---|---|\n" + rows,
            encoding="utf-8",
        )
        return matrix

    def test_missing_and_stale_matrix_items_fail(self) -> None:
        item = api_docs.PublicItem("struct", "vmnl_graphics::Context")
        row = api_docs.CoverageRow(item, "Context", "context.md", "Rustdoc", "context.html", "test")
        with self.assertRaisesRegex(api_docs.ApiDocsError, "missing from matrix"):
            api_docs.validate_coverage(api_docs.Surface("surface", (item,), ()), ())
        with self.assertRaisesRegex(api_docs.ApiDocsError, "not present"):
            api_docs.validate_coverage(api_docs.Surface("surface", (), ()), (row,))

    def test_stale_generated_output_is_not_rewritten(self) -> None:
        with tempfile.TemporaryDirectory() as value:
            path = Path(value) / "snapshot.md"
            path.write_text("old\n", encoding="utf-8")
            with self.assertRaisesRegex(api_docs.ApiDocsError, "stale"):
                api_docs.verify_outputs({path: "new\n"})
            self.assertEqual(path.read_text(encoding="utf-8"), "old\n")

    def test_method_without_an_exported_owner_fails(self) -> None:
        surface = api_docs.Surface("surface", (), (api_docs.PublicMethod("vmnl_graphics::Missing", "new"),))
        with self.assertRaisesRegex(api_docs.ApiDocsError, "no owning matrix item"):
            api_docs.render_method_index(surface, ())

    def test_rejects_duplicate(self) -> None:
        with tempfile.TemporaryDirectory() as value:
            root = Path(value)
            rows = (
                "| struct | `vmnl_graphics::Context` | [Context](context.md) | [Rustdoc](context.html) | test |\n"
                "| struct | `vmnl_graphics::Context` | [Context](context.md) | [Rustdoc](context.html) | test |\n"
            )
            with self.assertRaisesRegex(api_docs.ApiDocsError, "duplicate"):
                api_docs.parse_matrix(self.write_matrix(root, rows))

    def test_rejects_missing_page_and_anchor(self) -> None:
        with tempfile.TemporaryDirectory() as value:
            root = Path(value)
            rows = api_docs.parse_matrix(
                self.write_matrix(
                    root,
                    "| struct | `vmnl_graphics::Context` | [Context](missing.md#new) | [Rustdoc](context.html) | test |\n",
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
                api_docs.PublicItem("struct", "vmnl_graphics::Context"),
                "Context",
                "context.md",
                "Rustdoc",
                str(rustdoc),
                "test",
            )
            surface = api_docs.Surface(
                "surface\n",
                (row.item,),
                (api_docs.PublicMethod("vmnl_graphics::Context", "new"),),
            )
            old_matrix = api_docs.MATRIX_PATH
            try:
                api_docs.MATRIX_PATH = root / "coverage_matrix.md"
                with self.assertRaisesRegex(api_docs.ApiDocsError, "method anchor"):
                    api_docs.validate_method_anchors(surface, (row,))
            finally:
                api_docs.MATRIX_PATH = old_matrix

    def test_routes_sticky_key_methods_to_input_modes(self) -> None:
        row = api_docs.CoverageRow(
            api_docs.PublicItem("struct", "vmnl_graphics::Window"),
            "Window",
            "../reference/window/window.md",
            "Rustdoc",
            "window.html",
            "test",
        )

        for name in ("is_sticky_keys_enabled", "set_sticky_keys"):
            self.assertEqual(
                api_docs.method_page(
                    row,
                    api_docs.PublicMethod("vmnl_graphics::Window", name),
                ),
                ("Input modes", "../reference/window/input/modes.md"),
            )


if __name__ == "__main__":
    unittest.main()
