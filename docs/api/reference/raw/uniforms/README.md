# Raw uniforms

[`Uniform<TData>`](uniform.md) owns one typed uniform buffer. [`UniformBuilder<TData>`](uniform_builder.md) uploads its initial value. `Uniform::write` updates the existing buffer directly and can fail on active CPU or GPU access conflicts.
