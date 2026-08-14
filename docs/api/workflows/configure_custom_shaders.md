# Configure custom 2D shaders

Supply `ShaderSource::Src` or `ShaderSource::Path` to `WindowBuilder::vertex_shader` and/or `fragment_shader` before `build`. Window creation reads/compiles them and creates the built-in 2D pipeline contract against those stages.

The custom shaders must preserve VMNL's expected 2D vertex interface, push-constant/window-size contract, entry point, and fragment output. Treat a path as runtime data relative to the process working directory.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, ShaderSource, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let window = Window::builder()
        .vertex_shader(ShaderSource::Path("shaders/color.vert".into()))
        .fragment_shader(ShaderSource::Path("shaders/color.frag".into()))
        .build(&context)?;
    drop(window);
    Ok(())
}
```

Use the canonical [`custom_shaders`](../../../examples/window/custom_shaders/src/main.rs) program and its shader files. See [`ShaderSource`](../reference/common/shader_source.md).
