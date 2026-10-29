# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

"""Check the real link-check recipe with controlled documentation discovery."""

import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
import unittest


JUSTFILE = Path(__file__).parents[2] / "Justfile"


def link_epilogue():
    recipe = JUSTFILE.read_text(encoding="utf-8").split("\n_docs-api-check:\n", 1)[1]
    recipe = recipe.split("\n# Regenerate the reviewed public API", 1)[0]
    return textwrap.dedent(recipe.split("    python3 tools/glfw_portability.py check\n", 1)[1])


class DocumentationDiscoveryTests(unittest.TestCase):
    def run_discovery(self, status, files):
        with tempfile.TemporaryDirectory() as work:
            captured = Path(work) / "arguments"
            script = '''rg() {
                printf '%s' "$VMNL_TEST_MARKDOWN_FILES"
                return "$VMNL_TEST_RG_STATUS"
            }
            lychee() { printf '%s\\n' "$@" > "$VMNL_TEST_LINK_ARGUMENTS"; }
            ''' + link_epilogue()
            result = subprocess.run(
                ["bash", "-euo", "pipefail", "-c", script],
                env=dict(os.environ, VMNL_TEST_RG_STATUS=str(status),
                         VMNL_TEST_MARKDOWN_FILES=files, VMNL_TEST_LINK_ARGUMENTS=str(captured)),
                capture_output=True, text=True,
            )
            return result.returncode, captured.read_text().splitlines() if captured.exists() else None

    def test_discovery_failure_cannot_be_reported_as_a_successful_link_check(self):
        for status in (2, 127):
            with self.subTest(status=status):
                code, arguments = self.run_discovery(status, "")
                self.assertNotEqual(code, 0)
                self.assertIsNone(arguments)

    def test_empty_documentation_inventory_is_rejected(self):
        code, arguments = self.run_discovery(0, "")
        self.assertNotEqual(code, 0)
        self.assertIsNone(arguments)

    def test_all_discovered_files_reach_lychee_as_individual_arguments(self):
        code, arguments = self.run_discovery(0, "docs/a file.md\ndocs/b.md\n")
        self.assertEqual(code, 0)
        self.assertEqual(arguments, ["--offline", "--include-fragments=full", "docs/a file.md",
                                    "docs/b.md", "CONTRIBUTING.md", "CHANGELOG.md", "README.md"])
