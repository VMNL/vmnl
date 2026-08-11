# VMNL GPU Tests

Test policy lives in [docs/testing.md](../../docs/testing.md).

GPU tests require Vulkan and a GLFW display.

They are ignored by default and serialize GLFW access inside the process. Each
test uses only the public `vmnl` facade.

```bash
just test-gpu
```
