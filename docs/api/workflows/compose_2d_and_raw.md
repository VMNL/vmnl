# Compose 2D and raw passes

Append `draw2d`, `draw_raw_2d`, and `draw_raw_2d_with` calls in the required logical order on one `FrameRenderer`; `RenderMode` does not reorder passes. Submit once.

```rust,no_run
# extern crate vmnl;
# use vmnl::{Context, Window};
# use vmnl::d2::Shape;
# use vmnl::raw::{Geometry, Pipeline};
# fn render<T>(window: &mut Window, shape: &Shape, pipeline: &Pipeline<T>, geometry: &Geometry<T>) -> vmnl::VMNLResult<()> {
window.render()
    .draw2d([shape])
    .draw_raw_2d(pipeline, [geometry])
    .draw2d([shape])
    .submit()?;
# Ok(())
# }
```

The canonical runnable composition is [`examples/raw/d2_composition`](../../../examples/raw/d2_composition/src/main.rs). A 3D pass must not be inserted: any recorded 3D pass makes submission fail.

## Inspect viewport and scissor policies

Run `just run raw_d2_composition`. Four pipelines are built once at startup; key presses select an existing combination.

- `V` toggles full-framebuffer viewport / fixed viewport at `[100, 50]` with extent `[450, 300]` pixels. It moves/scales the alpha triangle's transform.
- `S` toggles full-framebuffer scissor / fixed scissor at `[250, 150]` with extent `[100, 100]` pixels. With the fixed viewport selected (`V`), it visibly cuts the small triangle to the portion inside this rectangle, without changing its transform. Test all four combinations.
- The yellow outline marks the fixed scissor even when it is inactive. With `S` active, only triangle pixels inside it should remain; with `S` inactive, the triangle can extend beyond it. The outline is drawn by a following 2D pass and must not itself be clipped or moved by the raw pipeline.
- Resize the window with each combination selected: full policies follow the acquired image; fixed rectangles keep their pixel origin/extent and may extend outside the image. The title and console report the GLFW framebuffer dimensions in pixels on startup, resize, and policy changes. These are pixel dimensions, not logical window dimensions; HiDPI scaling can make them differ. They are not a last-submitted swapchain snapshot.
- With the full viewport, a large framebuffer can place the entire triangle outside the small fixed scissor; disappearance is then expected. With the fixed viewport, partial clipping must remain visible after resize while the rectangle stays within the image.
- The cyan square at `[20, 20]` is drawn by a 2D pass after the custom raw draw. It must remain visible and correctly positioned even with the fixed viewport/scissor selected. The green triangle uses a subsequent full-framebuffer raw pipeline and must remain in the upper-right area.

An operator should verify those observations, then run `raw_triangle`, `raw_pipeline` (`C` / `F` / `P` / `W`) and `raw_uniform` for visible regressions. API tests verify policy resolution/ranges; GPU tests exercise mixed draws and resize without asserting displayed pixels.

Policies are independent and immutable on a built pipeline. Inspect resolution without a GPU:

```rust
# extern crate vmnl;
use vmnl::raw::{PipelineSpec, Scissor, ScissorPolicy, Viewport, ViewportPolicy};
let spec = PipelineSpec::<()>::default()
    .viewport(ViewportPolicy::Fixed(Viewport {
        offset: [100.0, 50.0], extent: [450.0, 300.0], depth_range: [0.0, 1.0],
    }))
    .scissor(ScissorPolicy::Fixed(Scissor { offset: [250, 150], extent: [100, 100] }));
assert_eq!(spec.viewport_value().resolve([900, 600])?.extent, [450.0, 300.0]);
assert_eq!(spec.scissor_value().resolve([900, 600])?.offset, [250, 150]);
# Ok::<(), vmnl::VMNLError>(())
```

## Optional depth attachment regression

Run the same scene with an explicit depth attachment:

```bash
VMNL_DEPTH_FORMAT=d16 just run raw_d2_composition
VMNL_DEPTH_FORMAT=d32 just run raw_d2_composition
```

`VMNL_DEPTH_FORMAT` accepts `none` (also the default), `d16`, or `d32`; invalid values fail explicitly. Unsupported formats fail at window build with `VulkanFormatNotSupported`. Startup prints the validated target configuration. No format or device fallback occurs.

With `none`, the original alpha triangle and `V` / `S` workflow above are unchanged. With a depth format, two opaque, coincident triangles replace that triangle: yellow at `z=0.25`, purple at `z=0.75`, with clear depth `1`. Twelve pipelines are built before the loop; keys select existing state.

- Initially depth is disabled and yellow is drawn before purple: purple wins because it is drawn last.
- Press `D`: `Less` with writes enabled. Yellow wins because it is nearer. Press `O` to reverse draw order: yellow must still win.
- Press `D` again: `Less` with writes disabled. Both triangles pass against clear depth `1`; the last drawn triangle wins. `O` must alternate yellow/purple.
- Press `D` again: depth is disabled. The last drawn triangle wins again. The title and console print depth state and draw order.
- Repeat with `V`, `S`, and resize. The yellow scissor outline, cyan 2D square and subsequent green depth-disabled raw marker must remain positioned and visible as before. The 2D background must not populate depth or occlude the raw triangles. Depth clears every frame; previous frames must not cause flicker/disappearance.

Closing/reopening should work for both formats when supported. Run `raw_triangle`, `raw_pipeline` (`C` / `F` / `P` / `W`) and `raw_uniform` afterwards for visible regressions. GPU tests exercise pipeline creation, all comparison/write combinations, mixed passes, image reuse and resize; unit tests check backend state translation. They do not assert displayed pixels. These observations require an explicit operator report for manual graphical validation.

## Inspect and preflight the target

The example prints `resolved attachments` at startup and checks each built pipeline against its window. `RenderTargetInfo` reports color format, samples per pixel and optional depth format; it excludes extent and clear values. Current windows use one sample per pixel. Automatic surface format selection and the displayed scene retain their existing policies.

```rust,no_run
# extern crate vmnl;
use vmnl::{raw::Pipeline, Window};
fn inspect<V>(pipeline: &Pipeline<V>, window: &Window) -> vmnl::VMNLResult<()> {
    let info = window.render_target_info();
    println!("color: {}, samples: {}, depth: {:?}",
        info.color_format(), info.sample_count(), info.depth_format());
    pipeline.validate_target(window)?;
    assert_eq!(pipeline.render_target_info(), info);
    Ok(())
}
```

Preflight checks device/render-pass identity without GPU work or submission. Another window is rejected even if its description is equal. Resize preserves the originating window's render pass and description. This check does not replace submission validation of geometry/resources, viewport/scissor bounds or framebuffer readiness. See [`RenderTargetInfo`](../reference/window/render_target_info.md).
