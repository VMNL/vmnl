# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

import importlib.util
from pathlib import Path
import sys
import unittest


MODULE_PATH = Path(__file__).parents[1] / "commit_message.py"
SPEC = importlib.util.spec_from_file_location("commit_message", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
commit_message = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = commit_message
SPEC.loader.exec_module(commit_message)


class CommitMessageValidationTests(unittest.TestCase):
    def test_accepts_project_types_scopes_and_breaking_changes(self) -> None:
        messages = tuple(
            f"{commit_type}: validate commit messages"
            for commit_type in commit_message.ALLOWED_TYPES
        ) + (
            "fix(ci): handle an empty commit range",
            "feat(api)!: remove the legacy entry point\n\nBREAKING CHANGE: use Context",
        )

        for message in messages:
            with self.subTest(message=message):
                self.assertEqual(commit_message.validate_message(message), [])

    def test_rejects_unknown_type(self) -> None:
        errors = commit_message.validate_message("ci: validate commit messages")

        self.assertTrue(any("allowed types" in error for error in errors))

    def test_rejects_invalid_description_style(self) -> None:
        uppercase_errors = commit_message.validate_message("fix: Reject uppercase")
        period_errors = commit_message.validate_message("fix: reject trailing period.")

        self.assertTrue(any("lowercase" in error for error in uppercase_errors))
        self.assertTrue(any("period" in error for error in period_errors))

    def test_rejects_long_subject(self) -> None:
        message = "docs: " + "a" * 67

        self.assertTrue(
            any("at most 72" in error for error in commit_message.validate_message(message))
        )

    def test_rejects_body_without_blank_separator(self) -> None:
        errors = commit_message.validate_message("fix: reject malformed body\nbody")

        self.assertTrue(any("blank line" in error for error in errors))

    def test_rejects_long_body_line(self) -> None:
        errors = commit_message.validate_message("docs: update guidance\n\n" + "a" * 81)

        self.assertTrue(any("line 3" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
