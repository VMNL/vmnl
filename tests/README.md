# VMNL Tests

Test crates are split by execution boundary:

- `api`: headless public behavior through the `vmnl` facade.
- `smoke`: executable startup checks without opening a window.
- `gpu`: Vulkan/display checks, run explicitly.

Commands:

```bash
just test-unit
just test-api
just test-smoke
just test-gpu
just test
```

See [docs/testing.md](../docs/testing.md) for the full policy.
