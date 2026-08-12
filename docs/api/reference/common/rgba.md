# `Rgba`

## Public path and maturity

Import path: `vmnl::common::Rgba`. Status: experimental, operational value type.

## Purpose and use cases

Stores an RGBA color for shape vertices, clear colors, and client-defined buffer layouts.

## Public API

Public `u8` fields: `r`, `g`, `b`, `a`. Constructors: `rgb(r, g, b)` sets alpha to `255`; `rgba(r, g, b, a)` and its alias `new` set all channels. The type is `#[repr(C)]` and derives `Clone`, `Copy`, `Debug`, `Default`, `Pod`, `Zeroable`, and `PartialEq`. It explicitly implements `Eq` and component-wise total `Ord`/`PartialOrd`, conversions from `[u8; 3]` and `[u8; 4]`, component-wise `AddAssign`, `Sub`, `SubAssign`, and multiplication by `u8` or `f32`; arithmetic is clamped/saturating as documented by Rustdoc.

`TRANSPARENT` is `(0, 0, 0, 0)`. Opaque named constants are: `BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE`, `CYAN`, `MAGENTA`, `YELLOW`, `ALICE_BLUE`, `ANTIQUE_WHITE`, `AQUA`, `AQUAMARINE`, `AZURE`, `BEIGE`, `BISQUE`, `BLANCHED_ALMOND`, `BLUE_VIOLET`, `BROWN`, `BURLY_WOOD`, `CADET_BLUE`, `CHARTREUSE`, `CHOCOLATE`, `CORAL`, `CORNFLOWER_BLUE`, `CORNSILK`, `CRIMSON`, `DARK_BLUE`, `DARK_CYAN`, `DARK_GOLDENROD`, `DARK_GRAY`, `DARK_GREEN`, `DARK_GREY`, `DARK_KHAKI`, `DARK_MAGENTA`, `DARK_OLIVE_GREEN`, `DARK_ORANGE`, `DARK_ORCHID`, `DARK_RED`, `DARK_SALMON`, `DARK_SEA_GREEN`, `DARK_SLATE_BLUE`, `DARK_SLATE_GRAY`, `DARK_SLATE_GREY`, `DARK_TURQUOISE`, `DARK_VIOLET`, `DEEP_PINK`, `DEEP_SKY_BLUE`, `DIM_GRAY`, `DIM_GREY`, `DODGER_BLUE`, `FIREBRICK`, `FLORAL_WHITE`, `FOREST_GREEN`, `FUCHSIA`, `GAINSBORO`, `GHOST_WHITE`, `GOLD`, `GOLDENROD`, `GRAY`, `GREY`, `GREEN_YELLOW`, `HONEYDEW`, `HOT_PINK`, `INDIAN_RED`, `INDIGO`, `IVORY`, `KHAKI`, `LAVENDER`, `LAVENDER_BLUSH`, `LAWN_GREEN`, `LEMON_CHIFFON`, `LIGHT_BLUE`, `LIGHT_CORAL`, `LIGHT_CYAN`, `LIGHT_GOLDENROD_YELLOW`, `LIGHT_GRAY`, `LIGHT_GREEN`, `LIGHT_GREY`, `LIGHT_PINK`, `LIGHT_SALMON`, `LIGHT_SEA_GREEN`, `LIGHT_SKY_BLUE`, `LIGHT_SLATE_GRAY`, `LIGHT_SLATE_GREY`, `LIGHT_STEEL_BLUE`, `LIGHT_YELLOW`, `LIME`, `LIME_GREEN`, `LINEN`, `MAROON`, `MEDIUM_AQUAMARINE`, `MEDIUM_BLUE`, `MEDIUM_ORCHID`, `MEDIUM_PURPLE`, `MEDIUM_SEA_GREEN`, `MEDIUM_SLATE_BLUE`, `MEDIUM_SPRING_GREEN`, `MEDIUM_TURQUOISE`, `MEDIUM_VIOLET_RED`, `MIDNIGHT_BLUE`, `MINT_CREAM`, `MISTY_ROSE`, `MOCCASIN`, `NAVAJO_WHITE`, `NAVY`, `OLD_LACE`, `OLIVE`, `OLIVE_DRAB`, `ORANGE`, `ORANGE_RED`, `ORCHID`, `PALE_GOLDENROD`, `PALE_GREEN`, `PALE_TURQUOISE`, `PALE_VIOLET_RED`, `PAPAYA_WHIP`, `PEACH_PUFF`, `PERU`, `PINK`, `PLUM`, `POWDER_BLUE`, `PURPLE`, `REBECCA_PURPLE`, `ROSY_BROWN`, `ROYAL_BLUE`, `SADDLE_BROWN`, `SALMON`, `SANDY_BROWN`, `SEA_GREEN`, `SEA_SHELL`, `SIENNA`, `SILVER`, `SKY_BLUE`, `SLATE_BLUE`, `SLATE_GRAY`, `SLATE_GREY`, `SNOW`, `SPRING_GREEN`, `STEEL_BLUE`, `TAN`, `TEAL`, `THISTLE`, `TOMATO`, `TURQUOISE`, `VIOLET`, `WEB_GREEN`, `WHEAT`, `WHITE_SMOKE`, and `YELLOW_GREEN`.

## Construction, defaults, and validation

`Default` is transparent black (`0, 0, 0, 0`). Every `u8` bit pattern is valid; no runtime validation occurs.

## Units, coordinates, and valid ranges

Every channel is in `0..=255`; alpha `0` is transparent and `255` opaque. VMNL normalizes channels to `0.0..=1.0` when transferring its built-in color format to shaders.

## Ownership, lifecycle, and threading

Plain copied data with no borrowed or external resource.

## Errors, panics, and failure conditions

Constructors are infallible. Public arithmetic does not intentionally panic on component overflow; exact operator semantics are canonical in Rustdoc.

## Allocation, transfers, synchronization, and GPU cost

No heap allocation or synchronization. The `repr(C)`/`Pod` layout permits byte transfer as part of compatible GPU data structures.

## Platform, Vulkan, and display constraints

None. Color-space conversion, blending, framebuffer format, and display calibration are not specified by this type.

## Example and related types

```rust
# extern crate vmnl;
use vmnl::common::Rgba;

let color = Rgba::rgb(20, 24, 32);
assert_eq!(color, Rgba::new(20, 24, 32, 255));
```

Related: [`Vertex2D`](../d2/vertex_2d.md), [`Vertex3D`](../d3/vertex_3d.md), and [`Window` configuration](../window/configuration.md).
