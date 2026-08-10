# VMNL API Tests

Test policy lives in [docs/testing.md](../../docs/testing.md).

API tests validate public behavior across the facade crate.

They are headless and assertion-based:

```bash
just test-api
```

Unit tests remain separate:

```bash
just test-unit
```

`just test` runs unit, API, and smoke tests together.
