# Create a context and window

1. Create one `Context` for the related graphics resources.
2. Configure a `WindowBuilder`; keep dimensions at least 64 pixels.
3. Build the window from that context.
4. Treat creation as GPU/display-dependent and propagate `VMNLResult`.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, PresentMode, Window};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::new()?;
    let window = Window::builder()
        .title("VMNL")
        .size(1280, 720)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(&context)?;
    drop(window);
    Ok(())
}
```

`Context` chooses the device automatically; equal-ranked selection is not deterministic. See [`Context`](../reference/context.md), [`WindowBuilder`](../reference/window/window_builder.md), and [`PresentMode`](../reference/window/present_mode.md).

## Require optional device features

Pass a CPU-side `DeviceConfig` to `Context::builder().device(config)`. Configuration can be cloned for reuse; passing another config replaces the previous one. Without a config, `Context::builder().build()` preserves `Context::new()` defaults.

```rust,no_run
# extern crate vmnl;
use vmnl::{Context, DeviceConfig, DeviceFeature};

fn main() -> vmnl::VMNLResult<()> {
    let context = Context::builder()
        .device(DeviceConfig::default().require_features([
            DeviceFeature::FillModeNonSolid,
            DeviceFeature::WideLines,
        ]))
        .build()?;
    println!("GPU: {}", context.device_name());
    assert!(context.is_device_feature_enabled(DeviceFeature::WideLines));
    Ok(())
}
```

Requirements filter GPUs before ranking and are mandatory together on one device. A lower-ranked compatible GPU can replace a higher-ranked incompatible candidate. No compatible device yields `DeviceRequirementsNotMet`; the caller must explicitly change requirements or the environment. Support reported by `is_device_feature_supported` is distinct from activation reported by `is_device_feature_enabled`; unrequested capabilities can be supported but disabled. Device limits still apply.

`raw_pipeline` requires `LargePoints` for its point shader and prints the selected GPU and enabled state before creating the window. An operator should check the example's points and existing topology/culling workflows, then run `raw_triangle`, `raw_uniform`, and `raw_d2_composition` for visible regressions. Context/feature GPU tests do not assert displayed pixels.
