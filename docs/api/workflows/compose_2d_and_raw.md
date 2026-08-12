# Compose 2D and raw passes

Append `draw2d`, `draw_raw`, and `draw_raw_with` calls in the required logical order on one `FrameRenderer`; `RenderMode` does not reorder passes. Submit once.

```rust,no_run
# extern crate vmnl;
# use vmnl::{Context, Window};
# use vmnl::d2::Shape;
# use vmnl::raw::{Geometry, Pipeline};
# fn render<T>(window: &mut Window, shape: &Shape, pipeline: &Pipeline<T>, geometry: &Geometry<T>) -> vmnl::VMNLResult<()> {
window.render()
    .draw2d([shape])
    .draw_raw(pipeline, [geometry])
    .draw2d([shape])
    .submit()?;
# Ok(())
# }
```

The canonical runnable composition is [`examples/raw/d2_composition`](../../../examples/raw/d2_composition/src/main.rs). A 3D pass must not be inserted: any recorded 3D pass makes submission fail.

