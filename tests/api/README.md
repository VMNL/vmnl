# VMNL API Tests

Test policy lives in [docs/testing.md](../../docs/testing.md).

API tests validate public behavior across the facade crate.

They are headless and assertion-based:

```bash
./run -at
```

Unit tests remain separate:

```bash
./run -ut
```

`./run -ft` remains as a compatibility alias for `./run -at`.
