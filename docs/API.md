# API

## Introduction

VMNL does not divide applications into a single high-level or low-level mode. Control is selected per responsibility: an application can use an ergonomic window or 2D helper while retaining explicit control over another concern such as resource placement, pipeline topology, shader sources, or blending.

An abstraction is local and opt-in. Choosing a helper delegates only the responsibility named by that helper; it must not silently take ownership of unrelated GPU decisions.

## Control Spectrum Through Builders

The following snippets are illustrative examples of the control model. They omit surrounding setup and error handling unrelated to the selected options.

Builders are the primary control surface for optional configuration. Each method exposes one decision while retaining defaults for every decision that the caller does not need to control. This is how higher-level ergonomics and lower-level precision coexist in the same API.

## Constructor Inputs and Builder Options

Constructor parameters establish the minimum valid and meaningful object. Builder methods refine an object that is already meaningful with those required inputs.

- Put a value in the constructor when the object cannot be created meaningfully without it.
- Put a value in the builder when a documented default is valid or the caller may omit the decision.
- Do not move a structural value into a builder merely for fluent syntax.

For example, position belongs in a constructor when every instance requires a deliberate position and no meaningful default exists. A builder setter for position is appropriate only when a default position is a valid part of the API contract.

This is a design rule for new or intentionally breaking APIs. Existing public signatures remain their current contract until changed through a separate API decision.

For a rectangle, a caller can keep shape creation ergonomic while selecting only the placement, pivot, rotation, and buffer preference that matter:

```rust
let rectangle = Shape::rect(160.0, 160.0)
    .position(420.0, 160.0)
    .color(Rgba::rgba(255, 255, 255, 120))
    .anchor(Anchor::BottomRight)
    .rotation(35.0)
    .buffer_memory_preference(BufferMemoryPreference::Device)
    .build(&context)?;
```

`anchor` and `origin` configure the same rotation pivot, with different control levels:

```rust
// Finite, semantic choice.
.anchor(Anchor::Center)

// Exact local coordinate when no predefined anchor fits.
.origin(20.0, 90.0)
```

The last call replaces the previous pivot choice. Use `Anchor` when the meaningful choices are finite; use `origin` when the caller needs a precise value.

The same builder pattern exposes raw pipeline decisions without introducing a global low-level mode:

```rust
let pipeline = raw::Pipeline::<RawVertex>::builder()
    .vertex_shader(vertex_shader)
    .fragment_shader(fragment_shader)
    .topology(raw::PrimitiveTopology::TriangleList)
    .blend_mode(raw::BlendMode::Alpha)
    .build(&window)?;
```

The lower-level path composes with the same `Context` and `Window` lifecycle used by higher-level APIs.

## Decision Rules

- Start with the narrowest API that exposes every decision the feature currently needs.
- Move to a lower-level API only for a concrete missing control point, not for anticipated flexibility.
- Decide constructor parameters before adding builder methods: required values define the object; optional values configure it.
- Expose each optional low-level decision through an independent builder method; document defaults and required fields.
- Use an enum when the valid choices are finite and semantic, such as `Anchor`, `PrimitiveTopology`, or `BlendMode`.
- Use a scalar, vector, or dedicated type when the caller needs a continuous or exact value, such as `origin(x, y)` or rotation.
- Keep control explicit for resource ownership, synchronization, allocation, pipeline creation, and other expensive GPU operations.
- A high-level helper must expose an explicit configuration point before it needs to become more generic.
- The high-level and low-level APIs must be clearly documented, with precise examples, and discoverable, so that users can find the right level of control for their needs.
