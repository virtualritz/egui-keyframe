//! egui widgets for keyframe editing.

pub mod bounding_box;
mod curve_editor;
pub mod keyframe_dot;
pub mod time_ruler;

pub use bounding_box::{AnchorMode, BoundingBox, BoundingBoxConfig, BoundingBoxHandle};
pub use curve_editor::{
    CurveEditor, CurveEditorConfig, CurveEditorResponse, HandleDrag, HandleSide, KeyframeMove,
};
pub use keyframe_dot::KeyframeDot;
pub use time_ruler::TimeRuler;
