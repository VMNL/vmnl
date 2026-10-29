# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

"""Exercise the workflow's compiler guards with controlled rustc output."""

import os
from pathlib import Path
import re
import subprocess
import textwrap
import unittest


WORKFLOW = Path(__file__).parents[2] / ".github/workflows/ci.yml"


def compiler_guards():
    return re.findall(
        r"      - name: Verify selected compiler(?:s)?\n        shell: bash\n        run: \|\n((?:          .*\n)+)",
        WORKFLOW.read_text(encoding="utf-8"),
    )


class CompilerGuardTests(unittest.TestCase):
    def test_every_validation_job_checks_the_selected_compiler(self):
        self.assertEqual(len(compiler_guards()), 4)

    def test_selected_version_is_accepted_and_other_versions_are_rejected(self):
        guards = compiler_guards()
        self.assertTrue(guards)
        for version, expected in (("1.87.0", 0), ("1.95.0", 1), ("1.87.1", 1)):
            for guard in guards:
                # No toolchain installation: substitute only rustc's observation.
                script = 'rustc() { printf "rustc %s (test)\\n" "$VMNL_TEST_RUSTC_VERSION"; }\n'
                script += textwrap.dedent(guard)
                with self.subTest(version=version, guard=guard):
                    result = subprocess.run(
                        ["bash", "-e", "-c", script],
                        env=dict(os.environ, VMNL_TEST_RUSTC_VERSION=version),
                        capture_output=True, text=True,
                    )
                    self.assertEqual(result.returncode, expected, result.stderr)
