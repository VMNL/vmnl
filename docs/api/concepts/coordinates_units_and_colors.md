# Coordinates, units, and colors

- 2D positions and dimensions use `f32`. Shape coordinates are interpreted by the active 2D shader contract; the built-in shape path uses window-space geometry transformed for rendering.
- Rectangle rotation is expressed in degrees.
- Window sizes and framebuffer sizes are pixels. Window positions and work areas use platform screen coordinates.
- Monitor physical sizes are millimetres; refresh rates are hertz; content scale is a dimensionless pair.
- `Rgba` channels are `u8` in `0..=255`; alpha `0` is transparent and `255` opaque.
- Raw shader coordinates, units, ranges, and matrix conventions are application-defined by the shaders.
- 3D coordinate handedness, clip convention, camera transforms, and projection are not specified because 3D rendering is scaffolded.
