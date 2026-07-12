# VMNL Tests

Test crates are split by execution boundary:

- `api`: headless public behavior through the `vmnl` facade.
- `smoke`: executable startup checks without opening a window.
- `gpu`: Vulkan/display checks, run explicitly.

Commands:

```bash
./run -ut
./run -at
./run -st
./run -gt
./run -t
```

See [docs/testing.md](../docs/testing.md) for the full policy.
