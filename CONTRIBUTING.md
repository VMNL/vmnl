# Contributing

Follow the project rules in [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md).

## Before Submission

Run the checks relevant to the change:

```bash
./run -w
./run -t
./run -d
```

Run `./run -gt` only on a machine with a Vulkan-capable GPU and a display server.

- Public API changes update Rustdoc and API tests.
- Bug fixes include a regression test.
- Visual workflows belong in `examples/`; headless checks belong in `tests/`.

## Commit Messages

Format:

```text
<type>: <description>
```

- Use imperative mood, lowercase, at most 72 characters, and no trailing period.
- Use an optional body after a blank line; each line stays under 80 characters.
- Do not use gitmoji or legacy project scopes.
- Add `BREAKING CHANGE:` in the body when a public contract is incompatible.
- For more information, see [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

Allowed types are defined in [docs/INSTRUCTIONS.md](docs/INSTRUCTIONS.md#commit-type-usage-guidelines).
