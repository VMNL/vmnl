# VMNL GPU Tests

Test policy lives in [docs/testing.md](../../docs/testing.md).

GPU tests require Vulkan and a GLFW display.

They are ignored by default and serialize GLFW access inside the process. Each
test uses only the public `vmnl` facade.

The opt-in native input scenario launches the separate `input_client` package,
whose only direct dependency is `vmnl`. Its X11 injector and process driver run
outside that client. Run it with `just input-test-vmnl x11`; setup and evidence
limits are documented in [Testing](../../docs/testing.md#native-input-contract).

```bash
just test-gpu
```
