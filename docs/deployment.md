# Deployment

VMNL releases are manual. Automation must not create or push tags, GitHub releases, or crates.io
publications.

## Version and compatibility policy

VMNL has no public release yet. The current `0.1.0` workspace version is development metadata, not
a published compatibility promise. Public contracts may change until the first `0.1.0` release is
prepared and reviewed.

The first public `0.1.0` starts a compatibility commitment stronger than Cargo's default treatment
of pre-1.0 versions:

- published Rust source APIs, documented behavior, defaults, and Cargo feature names remain
  compatible throughout `0.x`;
- a resolved default is part of the compatibility contract when documentation promises its value;
- replacement APIs are introduced before old APIs are deprecated;
- deprecated `0.x` APIs remain available until `1.0.0` and may be removed there with a migration
  guide;
- undocumented implementation details and backend interoperability explicitly marked unstable are
  excluded from this commitment;
- C ABI, asset formats, and network protocols become compatibility surfaces only when their own
  documentation declares them stable.

Fixing behavior that violates a documented contract restores compatibility. Changing an existing
documented contract, observable default, ownership rule, or error behavior requires compatibility
analysis even when the Rust signature is unchanged.

## Release milestones

`0.1.0` requires stable window/input behavior, 2D shapes, the minimal public raw graphics path, and
an operational audio path whose acceptance contract is approved by the audio maintainers. Its
public API, defaults, tests, examples, and documentation must be reviewed as the initial
compatibility baseline.

`0.2.0` adds textures without breaking the published `0.1.0` contracts. Text, batching, and network
development may proceed in parallel after that baseline.

`1.0.0` requires:

- independently selectable graphics, audio, and network domains;
- a high-level path capable of supporting a small game;
- a raw path capable of supporting a customized engine;
- reference applications using only public VMNL APIs;
- qualified Linux support and documented Windows/macOS support boundaries;
- measured performance evidence for every advertised performance guarantee;
- a declared stable C ABI and an idiomatic C++ wrapper;
- complete public contracts, migration guidance, and release documentation.

## Distribution targets

- Rust crates are published through crates.io.
- Versioned C libraries, headers, and C++ wrappers are distributed through GitHub releases first.
- C++ is the priority foreign-language interface, implemented over the stable C ABI rather than a
  compiler-specific C++ ABI.
- APT and DNF repository distribution is considered only after the GitHub artifacts, ABI policy,
  supported target triples, and maintenance process are qualified.

## Release notes

Internal development before the first public release does not require one changelog entry per
change. The `0.1.0` release prepares one reviewed summary of its public contract and notable
capabilities. After `0.1.0`, `CHANGELOG.md` records only user-visible additions, changes, fixes, and
deprecations; internal refactors, tests, and documentation-only maintenance are omitted.

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
