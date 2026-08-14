# Draw in 2D

1. Build shapes once from the same `Context` used by the window.
2. Start a frame with `window.render()`.
3. Append one or more `draw2d` passes; call order is preserved.
4. Call `submit` and handle resize/out-of-date errors according to the application loop.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, Window};
use vmnl::common::Rgba;
use vmnl::d2::Shape;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::new(&context)?;
    let rect = Shape::rect(200.0, 100.0).color(Rgba::RED).build(&context)?;
    window.render().draw2d([&rect]).submit()?;
    Ok(())
}
```

Full programs: [`shapes`](../../../examples/d2/shapes/src/main.rs) and [`advanced_geometry`](../../../examples/d2/advanced_geometry/src/main.rs). See [`Shape`](../reference/d2/shapes/shape.md) and [`FrameRenderer`](../reference/window/rendering/frame_renderer.md).
