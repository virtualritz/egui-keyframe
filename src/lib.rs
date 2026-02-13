//! # egui-keyframe
//!
//! Keyframe animation and curve editing widgets for egui.
//!
//! This crate provides:
//! - [`Keyframe`] and [`Track`] data structures for storing animation data
//! - [`CurveEditor`] widget for editing bezier animation curves
//! - [`DopeSheet`] widget for timeline-style keyframe editing
//! - Traits for integrating with your own data model
//!
//! ## Quick Start
//!
//! ```ignore
//! use egui_keyframe::{Track, Keyframe, CurveEditor, SpaceTransform, TimeTick};
//!
//! // Create a track with some keyframes
//! let mut track = Track::<f32>::new();
//! track.add_keyframe(Keyframe::new(TimeTick::new(0.0), 0.0));
//! track.add_keyframe(Keyframe::new(TimeTick::new(1.0), 100.0));
//!
//! // In your egui code:
//! let space = SpaceTransform::new(100.0, TimeTick::default(), 400.0);
//! CurveEditor::new(&track, &selected, &space, (0.0, 100.0)).show(ui);
//! ```

// Type aliases for consistent usage (ahash for faster hashing).
pub type HashSet<T> = ahash::AHashSet<T>;
pub type HashMap<K, V> = ahash::AHashMap<K, V>;

pub mod core;
pub mod dopesheet;
pub mod spaces;
pub mod traits;
pub mod widgets;

// Re-exports for convenience
pub use core::{
    easing,
    interpolation::{CubicBezier, InterpolationTriple, interpolate_at_position},
    keyframe::{BezierHandles, Keyframe, KeyframeId, KeyframeType},
    time::TimeTick,
    track::{Track, TrackId},
};
pub use dopesheet::DopeSheet;
pub use spaces::SpaceTransform;
pub use traits::{
    Animatable, AnimationCommand, AnimationDataMutator, AnimationDataProvider, KeyframeSource,
    KeyframeView, PropertyRow,
};

// Re-export uuid for KeyframeId construction in downstream crates
pub use uuid;
pub use widgets::{
    AnchorMode, BoundingBox, BoundingBoxConfig, BoundingBoxHandle, CurveEditor, HandleSide,
};
