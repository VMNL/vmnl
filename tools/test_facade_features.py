#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

"""Test independent Cargo clients; writes only target/ fixtures and build artifacts.

Each manifest has its own workspace and seeds its lockfile from the repository's
lockfile. Cargo resolves the added client package, then graph inspection is locked.
Normal dependency features are independent from the fixture's expected-API flags.
No device, package installer, publication or host configuration is invoked.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CASES = {
    "default": (True, [], ["graphics"]),
    "empty": (False, [], []),
    "graphics": (False, ["graphics"], ["graphics"]),
    "audio": (False, ["audio"], ["audio"]),
    "both": (False, ["graphics", "audio"], ["graphics", "audio"]),
}
GRAPHICS_PACKAGES = {"vmnl-graphics", "glfw", "glfw-sys", "vulkano", "vulkano-macros", "ash", "vk-sys"}


def validate_graph(packages: set[str], expected: list[str]) -> None:
    if "vmnl" not in packages:
        raise RuntimeError("client graph does not contain the facade")
    if ("vmnl-graphics" in packages) != ("graphics" in expected):
        raise RuntimeError("graphics feature does not match the selected domain")
    if "graphics" not in expected and (leaked := packages & GRAPHICS_PACKAGES):
        raise RuntimeError(f"headless client acquired graphics dependencies: {sorted(leaked)}")
    if ("vmnl_audio" in packages) != ("audio" in expected):
        raise RuntimeError("audio feature does not match the selected domain")


def run_case(name: str, clippy: bool) -> None:
    defaults, dependency_features, expected = CASES[name]
    work = ROOT / "target" / "facade-feature-clients" / name
    work.mkdir(parents=True, exist_ok=True)
    dependency = (
        f'vmnl = {{ path = {json.dumps((ROOT / "crates/vmnl").as_posix())}, '
        f'default-features = {str(defaults).lower()}, features = {json.dumps(dependency_features)} }}\n'
    )
    manifest = (
        f'[package]\nname = "vmnl-facade-{name}-probe"\nversion = "0.0.0"\n'
        'edition = "2021"\nrust-version = "1.87"\npublish = false\n'
        '[profile.dev]\nopt-level = 3\n'
        '[workspace]\nresolver = "2"\n[features]\ngraphics = []\naudio = []\n'
        f'[lib]\npath = {json.dumps((ROOT / "tests/api/facade_features.rs").as_posix())}\n'
        '[dependencies]\n' + dependency
    )
    if "audio" in expected:
        manifest += f'[dev-dependencies]\nvmnl_audio = {{ path = {json.dumps((ROOT / "crates/vmnl_audio").as_posix())} }}\n'
    path = work / "Cargo.toml"
    path.write_text(manifest, encoding="utf-8")
    shutil.copyfile(ROOT / "Cargo.lock", work / "Cargo.lock")
    arguments = ["--manifest-path", str(path)]
    if expected:
        arguments += ["--features", ",".join(expected)]
    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target"))
    command = ["cargo", "clippy" if clippy else "test", *arguments]
    if clippy:
        command += ["--all-targets", "--", "-D", "warnings"]
    print(f"Facade client: {name} ({'Clippy' if clippy else 'tests'})", flush=True)
    subprocess.run(command, cwd=ROOT, env=env, check=True)
    tree = subprocess.run(
        ["cargo", "tree", *arguments, "--locked", "--edges", "normal,build", "--prefix", "none", "--format", "{p}"],
        cwd=ROOT, env=env, check=True, text=True, stdout=subprocess.PIPE,
    ).stdout
    packages = {line.split()[0] for line in tree.splitlines() if line.strip()}
    validate_graph(packages, expected)
    (work / "dependency-tree.txt").write_text(tree, encoding="utf-8")
    print(f"Facade graph: {name} PASS ({len(packages)} packages)", flush=True)
    if not clippy:
        # Test the facade's own feature-sensitive Rustdoc, not just client source.
        docs = ["cargo", "test", "--manifest-path", str(ROOT / "crates/vmnl/Cargo.toml"), "--doc", "--locked"]
        if not defaults:
            docs.append("--no-default-features")
        if dependency_features:
            docs += ["--features", ",".join(dependency_features)]
        subprocess.run(docs, cwd=ROOT, env=env, check=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=CASES, action="append")
    parser.add_argument("--clippy", action="store_true")
    options = parser.parse_args()
    for name in options.case or CASES:
        run_case(name, options.clippy)


if __name__ == "__main__":
    main()
