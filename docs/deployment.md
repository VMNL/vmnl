# Deployment

VMNL releases are manual. Automation must not create or push tags, GitHub releases, or crates.io
publications.

## Current publication blocker

VMNL temporarily depends on `https://github.com/VMNL/glfw-rs` at an exact Git revision. This fork
makes GLFW 3.4 error conversion total and prevents unknown or newly introduced error codes from
being converted with `transmute`.

No VMNL crate may be published to crates.io while this Git dependency remains. Before the first
publication:

1. merge the correction upstream and wait for an official `glfw` release containing it;
2. replace the Git dependency with that crates.io version;
3. regenerate and review the GLFW portability audit;
4. run the complete validation sequence;
5. run `cargo publish --dry-run` for publishable crates in dependency order.

A successful GitHub CI run does not remove this blocker and does not prove that the crates.io
dependency graph is publishable.

## Manual release protocol

After all blockers are removed, the release operator updates version metadata and release notes,
runs the required dry-runs, creates and pushes the tag, creates the GitHub release, and publishes
each crate manually. Credentials remain outside the repository.
