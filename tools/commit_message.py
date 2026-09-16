# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

"""Validate VMNL commit messages."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import subprocess
import sys


ALLOWED_TYPES = (
    "build",
    "cicd",
    "chore",
    "docs",
    "feat",
    "fix",
    "perf",
    "refactor",
    "style",
    "test",
)
MAX_BODY_LINE_LENGTH = 80
MAX_SUBJECT_LENGTH = 72
HEADER_PATTERN = re.compile(
    rf"^(?:{'|'.join(ALLOWED_TYPES)})"
    r"(?:\([a-z0-9][a-z0-9._/-]*\))?"
    r"!?: (?P<description>.+)$"
)


def validate_message(message: str) -> list[str]:
    lines = message.splitlines()
    if not lines:
        return ["message must not be empty"]

    errors: list[str] = []
    subject = lines[0]

    if len(subject) > MAX_SUBJECT_LENGTH:
        errors.append(
            f"subject must be at most {MAX_SUBJECT_LENGTH} characters "
            f"(found {len(subject)})"
        )

    match = HEADER_PATTERN.fullmatch(subject)
    if match is None:
        allowed_types = ", ".join(ALLOWED_TYPES)
        errors.append(
            "subject must match '<type>[optional scope][!]: <description>'; "
            f"allowed types: {allowed_types}"
        )
    else:
        description = match.group("description")
        if not (description[0].islower() or description[0].isdigit()):
            errors.append("description must start with a lowercase letter or digit")
        if description.endswith("."):
            errors.append("subject must not end with a period")

    if len(lines) > 1 and lines[1]:
        errors.append("body must be separated from the subject by a blank line")

    for line_number, line in enumerate(lines[1:], start=2):
        if len(line) > MAX_BODY_LINE_LENGTH:
            errors.append(
                f"line {line_number} must be at most {MAX_BODY_LINE_LENGTH} "
                f"characters (found {len(line)})"
            )

    return errors


def read_commit_message(commit: str) -> str:
    result = subprocess.run(
        ["git", "show", "--no-patch", "--format=%B", commit],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def commits_in_range(base: str, head: str) -> list[str]:
    result = subprocess.run(
        ["git", "rev-list", "--reverse", f"{base}..{head}"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.splitlines()


def report_errors(label: str, errors: list[str]) -> None:
    print(f"invalid commit message: {label}", file=sys.stderr)
    for error in errors:
        print(f"  - {error}", file=sys.stderr)


def check_file(path: Path) -> int:
    errors = validate_message(path.read_text(encoding="utf-8"))
    if errors:
        report_errors(str(path), errors)
        return 1

    return 0


def check_range(base: str, head: str) -> int:
    has_errors = False
    for commit in commits_in_range(base, head):
        message = read_commit_message(commit)
        errors = validate_message(message)
        if errors:
            report_errors(commit, errors)
            has_errors = True

    return int(has_errors)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    file_parser = subparsers.add_parser("file", help="validate a message file")
    file_parser.add_argument("path", type=Path)

    range_parser = subparsers.add_parser("range", help="validate a Git commit range")
    range_parser.add_argument("base")
    range_parser.add_argument("head")

    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.command == "file":
            return check_file(args.path)
        return check_range(args.base, args.head)
    except (OSError, subprocess.CalledProcessError, UnicodeError) as error:
        print(f"cannot validate commit messages: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
