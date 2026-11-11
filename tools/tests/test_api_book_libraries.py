# SPDX-FileCopyrightText: 2026 VMNL contributors
# SPDX-License-Identifier: MIT

import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "api_book_libraries", Path(__file__).parents[1] / "api_book_libraries.py")
libraries = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(libraries)


class LibraryStagingTests(unittest.TestCase):
    def test_old_builds_are_excluded_and_cargo_outputs_are_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary)
            deps = target / "debug/deps"
            deps.mkdir(parents=True)
            (target / "debug/libvmnl_graphics.rlib").write_text("current")
            old = deps / "libvmnl_graphics-old.rlib"
            old.write_text("old")
            (deps / "libvmnl_graphics-check.rmeta").write_text("metadata")
            (deps / "libvmnl-old.rlib").write_text("old facade")
            (deps / "libdependency.rlib").write_text("dependency")
            directory = target / "api-book-libs"
            directory.mkdir()
            (directory / old.name).symlink_to(old)
            for _ in range(2):
                libraries.stage_libraries(target)
                self.assertEqual(sorted(path.name for path in directory.iterdir()),
                                 ["libdependency.rlib", "libvmnl_graphics.rlib"])
                self.assertEqual((directory / "libvmnl_graphics.rlib").read_text(), "current")
                self.assertEqual(old.read_text(), "old")

    def test_unexpected_files_are_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            target = Path(temporary)
            (target / "debug").mkdir()
            (target / "debug/libvmnl_graphics.rlib").write_text("current")
            directory = target / "api-book-libs"
            directory.mkdir()
            path = directory / "keep.txt"
            path.write_text("user data")
            with self.assertRaisesRegex(RuntimeError, "non-symlink"):
                libraries.stage_libraries(target)
            self.assertEqual(path.read_text(), "user data")
