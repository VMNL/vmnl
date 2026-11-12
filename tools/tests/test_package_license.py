# SPDX-FileCopyrightText: 2026 VMNL contributors
# SPDX-License-Identifier: MIT

"""Keep the license distributed with each Cargo package consistent."""

import unittest
from pathlib import Path


class PackageLicenseTests(unittest.TestCase):
    def test_package_licenses_match_repository_license(self) -> None:
        root = Path(__file__).resolve().parents[2]
        expected = (root / "LICENSE").read_bytes()
        for package in ("vmnl_graphics", "vmnl_macros"):
            with self.subTest(package=package):
                self.assertEqual((root / "crates" / package / "LICENSE").read_bytes(), expected)


if __name__ == "__main__":
    unittest.main()
