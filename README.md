# `egui-keyframe`

Keyframe animation and curve editing widgets for [egui](https://github.com/emilk/egui).

Provides `Keyframe<T>` and `Track<T>` data structures alongside `CurveEditor` and `DopeSheet` widgets for building animation tools.

## Widgets

**CurveEditor** -- Bezier curve editor with handle editing, multi-keyframe bounding box transforms, keyboard shortcuts, zoom/pan, and context menus for interpolation type switching.

**DopeSheet** -- Timeline widget with a hierarchical property tree and keyframe tracks. Supports aggregate keyframes for parent rows, box selection, and playhead display.

## Quick start

```rust
use egui_keyframe::{Track, Keyframe, CurveEditor, SpaceTransform, TimeTick};

// Create a track with keyframes.
let mut track = Track::<f32>::new();
track.add_keyframe(Keyframe::new(TimeTick::new(0.0), 0.0));
track.add_keyframe(Keyframe::new(TimeTick::new(1.0), 100.0));

// In your egui code:
let space = SpaceTransform::new(100.0, TimeTick::default(), 400.0);
let selected = egui_keyframe::HashSet::default();
let response = CurveEditor::new(&track, &selected, &space, (0.0, 100.0)).show(ui);
```

## Integration

Widgets are decoupled from your data model via traits:

- **`KeyframeSource`** -- Read-only keyframe access for `CurveEditor`. Blanket-implemented for `Track<f32>`.
- **`AnimationDataProvider`** -- Read interface for `DopeSheet` (property tree, keyframe positions, values, handles).
- **`AnimationDataMutator`** -- Write interface via `AnimationCommand` enum, designed for undo/redo.
- **`Animatable`** -- Trait for types that can be interpolated. Implemented for `f32`, `f64`, `[f32; N]`.

## Feature flags

| Flag         | Default | Description                                                                   |
| ------------ | ------- | ----------------------------------------------------------------------------- |
| `serde`      | Yes     | Serialization for all public types                                            |
| `facet`      | No      | [Facet](https://crates.io/crates/facet) derive support                        |
| `frame-tick` | No      | Use [`frame-tick`](https://crates.io/crates/frame-tick) as `TimeTick` backend |

## Architecture

### Core types

- `TimeTick` -- Unit-agnostic time position (wraps `f64` or `frame_tick::Tick`)
- `Keyframe<T>` -- Value at a time point with bezier handles and interpolation type
- `Track<T>` -- Ordered collection of keyframes (backed by `IndexMap`)
- `BezierHandles` -- Cubic bezier control points (Theatre.js convention: X in [0,1], Y unbounded)
- `SpaceTransform` -- Coordinate conversion between time, zoomed, and screen space

### Coordinate spaces

`SpaceTransform` manages three coordinate spaces:

1. **Unit** -- Animation time (`TimeTick`)
2. **Scaled** -- Time x zoom (pixels, no scroll offset)
3. **Clipped** -- Screen coordinates (with scroll + padding)

### Easing presets

29 built-in easing presets from [easings.net](https://easings.net), with CSS cubic-bezier conversion and fuzzy preset matching.

## License

MIT OR Apache-2.0
