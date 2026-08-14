# Maturity, compatibility, and deprecation

The entire public API is experimental. Minor releases may change contracts while the project is pre-stable; review `CHANGELOG.md` and the public API snapshot during upgrades.

| Area | Maturity |
|---|---|
| Context/window/input/monitors | Experimental, operational |
| 2D shapes/rendering | Experimental, operational |
| Raw pipelines/geometry/uniforms | Experimental, operational within documented limits |
| 3D types/resources | Scaffolded; frame submission unavailable |

Deprecations should first appear in Rustdoc, this book, the snapshot review, and `CHANGELOG.md`. Removal requires an explicit reviewed API change; undocumented compatibility is not guaranteed.
