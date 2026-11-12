#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 VMNL contributors
# SPDX-License-Identifier: MIT

"""Build/run device-free external clients with only a normal or renamed graphics dependency."""

import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def locked_packages(path: Path) -> set[tuple[str, str, str]]:
    """Read identities from Cargo's generated lockfile blocks, without resolving again."""
    packages = set()
    for block in path.read_text(encoding="utf-8").split("[[package]]")[1:]:
        fields = dict(re.findall(r'(?m)^(name|version|source) = "([^"]+)"$', block))
        packages.add((fields["name"], fields["version"], fields.get("source", "")))
    return packages


def main() -> None:
    baseline = locked_packages(ROOT / "Cargo.lock")
    if any(name == "vmnl" for name, _, _ in baseline):
        raise RuntimeError("the removed facade remains in Cargo.lock")
    source = (ROOT / "tools/tests/fixtures/graphics_consumer.rs").read_text(encoding="utf-8")
    environment = os.environ.copy()
    environment["CARGO_TARGET_DIR"] = str(ROOT / "target")
    for dependency, import_name in (("vmnl_graphics", "vmnl_graphics"), ("graphics_backend", "graphics_backend")):
        with tempfile.TemporaryDirectory(prefix="vmnl-graphics-client-") as directory:
            client = Path(directory)
            (client / "src").mkdir()
            (client / "src/main.rs").write_text(source.replace("GRAPHICS_DEP", import_name), encoding="utf-8")
            package = "vmnl-graphics-external-client"
            package_field = '' if dependency == "vmnl_graphics" else 'package = "vmnl_graphics", '
            (client / "Cargo.toml").write_text(
                f'[package]\nname = "{package}"\nversion = "0.0.0"\nedition = "2021"\n\n'
                '[dependencies]\n'
                f'{dependency} = {{ {package_field}path = '
                f'{json.dumps((ROOT / "crates/vmnl_graphics").as_posix())} }}\n\n'
                '[profile.dev]\nopt-level = 3\n', encoding="utf-8")
            shutil.copy2(ROOT / "Cargo.lock", client / "Cargo.lock")
            # Cargo prunes workspace-only packages and adds this temporary client. All dependency
            # versions are seeded from the reviewed lockfile and checked below; no repository writes.
            subprocess.run(["cargo", "run", "--offline", "--manifest-path", str(client / "Cargo.toml")],
                           cwd=client, env=environment, check=True)
            unexpected = locked_packages(client / "Cargo.lock") - baseline - {(package, "0.0.0", "")}
            if unexpected:
                raise RuntimeError(f"external client changed dependency versions: {sorted(unexpected)}")
            metadata = json.loads(subprocess.check_output(
                ["cargo", "metadata", "--locked", "--offline", "--no-deps", "--format-version", "1"],
                cwd=client, env=environment, text=True))
            dependencies = metadata["packages"][0]["dependencies"]
            if len(dependencies) != 1 or dependencies[0]["name"] != "vmnl_graphics":
                raise RuntimeError("external client must depend only on the graphics package")
            print(f"External client ({dependency}): public API and derives passed; no native device opened", flush=True)


if __name__ == "__main__":
    main()
