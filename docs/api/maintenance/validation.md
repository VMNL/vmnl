# Validation

Prerequisites and exact versions are listed in [`docs/build.md`](../../build.md). Documentation-only
edits require structure, generated-file, snippet, and local-link checks. Source or tooling edits
follow the single ordered completion sequence in [`CONTRIBUTING.md`](../../../CONTRIBUTING.md#before-submission).

Before completion, regenerate and review the API book only when the public surface intentionally
changes:

```bash
just docs-api-update # reviewed public API changes only; inspect the generated diff
```

For an API-book-only change, run `just docs-api-check`. For source or tooling changes, execute the
full applicable sequence from `CONTRIBUTING.md` from the beginning; it already includes
`just docs-api-check` at the required position. Platform/GPU execution and human visual validation
remain impact- and environment-dependent; documentation compilation is not runtime or visual
evidence.
