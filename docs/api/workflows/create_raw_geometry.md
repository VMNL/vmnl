# Create raw geometry

After defining a `BufferContents` vertex type, pass a non-empty vertex collection to `Geometry::builder`. Add optional non-empty in-bounds `u32` indices and choose a memory preference before `build(&Context)`.

```rust,no_run
# extern crate vmnl;
use vmnl::Context;
use vmnl::raw::Geometry;

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let geometry = Geometry::<[f32; 2]>::builder([
        [0.0, -0.5], [0.5, 0.5], [-0.5, 0.5],
    ]).indices([0, 1, 2]).build(&context)?;
    drop(geometry);
    Ok(())
}
```

Topology-specific primitive counts are not validated by `GeometryBuilder`; align the data with `PipelineSpec::topology`. Use [`examples/raw/triangle`](../../../examples/raw/triangle/src/main.rs) and [`GeometryBuilder`](../reference/raw/geometry/geometry_builder.md).
