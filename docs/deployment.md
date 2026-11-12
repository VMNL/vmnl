# Deployment

VMNL Graphics releases are manual. Automation must not create or push tags, GitHub releases, or crates.io
publications.

## Version and compatibility policy

The current version is `0.1.0` for both `vmnl_graphics` and its internal `vmnl_macros` dependency.
This is the initial graphics API baseline, not a claim that every planned feature or platform is
qualified. Registry availability and release prerequisites are tracked below.

The minimum supported Rust version is 1.95. Changes to this minimum are documented and validated
before release; no fixed schedule for increasing it is promised.

The `0.1.0` baseline starts a compatibility commitment stronger than Cargo's default treatment
of pre-1.0 versions:

- published Rust source APIs, documented behavior, defaults, and Cargo feature names remain
  compatible throughout `0.x`;
- a resolved default is part of the compatibility contract when documentation promises its value;
- replacement APIs are introduced before old APIs are deprecated;
- deprecated `0.x` APIs remain available until `1.0.0` and may be removed there with a migration
  guide;
- undocumented implementation details and backend interoperability explicitly marked unstable are
  excluded from this commitment;
- C ABI and asset formats become compatibility surfaces only when their own
  documentation declares them stable.

Fixing behavior that violates a documented contract restores compatibility. Changing an existing
documented contract, observable default, ownership rule, or error behavior requires compatibility
analysis even when the Rust signature is unchanged.

## Release milestones

`0.1.0` is the current window/input, 2D shapes, and minimal public raw graphics baseline.
Its documented contracts define the initial compatibility surface. The
[feature status](api/appendices/feature_status.md), [architecture limits](architecture.md#current-implementation-boundaries),
and [platform qualification](platform_support.md) remain applicable; 3D rendering is scaffolded.
Release review must cover its public API, defaults, tests, examples, and documentation, including
the [stable contract requirements](architecture.md#stable-contract-requirements). The version
number alone does not satisfy these release requirements.

`0.2.0` adds textures without breaking the published `0.1.0` contracts. Text and batching
development may proceed in parallel after that baseline.

`1.0.0` requires:

- a complete stable graphics API, consumed directly through `vmnl_graphics`;
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

## Publication status and prerequisites

Version `0.1.0` is the repository baseline. Publication under the canonical package names
`vmnl_graphics` and `vmnl_macros` is pending. The GitHub repository remains `VMNL/vmnl-graphics`;
Rust import paths remain `vmnl_graphics` and `vmnl_macros`. A repository version does not itself
publish a crate, tag, or GitHub release; each distribution step remains manual.

The earlier registry names were `vmnl-graphics` and `vmnl-macros`. Changing a manifest does not
rename a published crate. Before publication under the underscore names, the release operator
must check registry eligibility, name availability, and any deletion cooldown. Registry deletion
and republication are separate manual operations; merging this change performs neither.

The workspace now uses crates.io `glfw 0.63.0`, which includes the upstream fix for total error
conversion. Platform qualification must be assessed for this dependency source and version;
historical fork results do not qualify it.

The graphics crate depends on the internal `vmnl_macros` package. Clients depend only on
`vmnl_graphics`; Cargo resolves the macros package as a dependency. The graphics manifest pins the
matching macros version exactly because the derives generate code against the graphics crate.
Before publishing graphics, publish the matching macros package and verify the registry distribution
and required dry-runs in dependency order.

A successful GitHub CI run does not establish registry availability or native platform
qualification. Required archive inspections, dry-runs, and registry verification remain release
checks even though the GLFW Git dependency has been removed.

## Manual release protocol

For `0.1.0`, the release operator verifies matching version metadata, reviews release notes and
qualification limits, and runs the required dry-runs. Each crate is published manually, followed
by registry verification; the tag and GitHub release identify the reviewed release commit.
Credentials remain outside the repository.

Use `cargo package --locked -p vmnl_macros -p vmnl_graphics` to verify both archives together.
Inspect their normalized manifests, license files, and packaged resources before release. Publish
the matching macros version first; verify graphics against the actual registry after it becomes
available. Packaging and dry-runs do not establish native platform qualification.
