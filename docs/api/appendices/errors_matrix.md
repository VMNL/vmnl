# Errors matrix

| Operation | Principal failure contract | Typical decision |
|---|---|---|
| `Context::new` | Vulkan initialization/device/queue/allocator categories | Abort graphics initialization or retry after environment change |
| `WindowBuilder::build` | invalid size; GLFW/surface/swapchain/shader/render-target failures; unsupported strict present mode | Fix configuration or environment; choose preferred/portable present mode |
| Runtime window setters | invalid size/range/aspect (`InvalidWindowSize`/`InvalidState`) | Correct input before retry |
| Shape/mesh build | invalid/overflowing geometry; vertex/index allocation failure | Correct geometry or release/reduce resources |
| Raw pipeline build | missing/read/compile/interface/layout/pipeline failures | Correct shaders/layout; verify device support |
| Raw geometry/uniform build | invalid counts/indices or buffer allocation | Correct data/reduce allocation |
| Raw resources build | context/device/layout/set/binding/type/array/duplicate/missing mismatch; descriptor allocation | Align shader and resource contract |
| Frame submit | 3D scaffold `InvalidState`; device/render-pass/resource mismatch; out-of-date/zero-size; record/submit/present/device errors | Never submit 3D; retry transient resize; rebuild/terminate on hard device failure |
| GLFW callbacks/infallible setters | No typed result; routed through callback/default backend handling | Log/classify callback and reconcile observed state |

`VMNLErrorKind` is non-exhaustive. This table groups public behavior; exact per-item variants remain in Rustdoc/reference pages.

