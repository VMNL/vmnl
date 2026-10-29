# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location("facade_features", Path(__file__).parents[1] / "test_facade_features.py")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class FeatureGraphTests(unittest.TestCase):
    def test_accepts_isolated_audio_and_empty_clients(self):
        MODULE.validate_graph({"vmnl", "vmnl_audio", "cpal", "alsa"}, ["audio"])
        MODULE.validate_graph({"vmnl"}, [])

    def test_rejects_transitive_graphics_leak_without_a_graphics_domain(self):
        with self.assertRaisesRegex(RuntimeError, "graphics dependencies"):
            MODULE.validate_graph({"vmnl", "vmnl_audio", "glfw-sys"}, ["audio"])

    def test_rejects_missing_or_unrequested_domains(self):
        for packages, expected in [({"vmnl"}, ["graphics"]), ({"vmnl", "vmnl_audio"}, [])]:
            with self.assertRaises(RuntimeError):
                MODULE.validate_graph(packages, expected)
