#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 VMNL contributors
# SPDX-License-Identifier: MIT

"""Stage snippet libraries without exposing old graphics builds or deleting Cargo outputs."""

from pathlib import Path


def stage_libraries(target: Path) -> Path:
    library = target / "debug/libvmnl_graphics.rlib"
    if not library.is_file():
        raise RuntimeError("missing current graphics library; build vmnl_graphics first")
    directory = target / "api-book-libs"
    if directory.is_symlink():
        raise RuntimeError("API-book library directory must not be a symlink")
    directory.mkdir(exist_ok=True)
    # This directory is owned by this staging operation. Only links may be replaced;
    # Cargo's actual artifacts in debug/ and debug/deps/ remain untouched.
    if any(not path.is_symlink() for path in directory.iterdir()):
        raise RuntimeError("unexpected non-symlink file in API-book library directory")
    for path in directory.iterdir():
        path.unlink()
    for artifact in (target / "debug/deps").iterdir():
        if artifact.is_file() and not artifact.name.startswith(("libvmnl_graphics-", "libvmnl-")):
            (directory / artifact.name).symlink_to(artifact.resolve())
    (directory / library.name).symlink_to(library.resolve())
    return directory


if __name__ == "__main__":
    stage_libraries(Path(__file__).resolve().parents[1] / "target")
