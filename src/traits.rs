//! Integration traits for connecting egui-keyframe to your data model.
//!
//! These traits allow the widgets to work with any data source without
//! coupling to a specific implementation.

use crate::core::keyframe::{BezierHandles, Keyframe, KeyframeId, KeyframeType};
use crate::core::time::TimeTick;
use crate::core::track::{Track, TrackId};

/// Trait for types that can be animated (interpolated).
pub trait Animatable: Clone + Send + Sync + 'static {
    /// Linearly interpolate between two values.
    fn lerp(&self, other: &Self, t: f32) -> Self;

    /// Get the "distance" between two values (for auto-scaling curves).
    fn distance(&self, other: &Self) -> f32;

    /// Get the default/zero value.
    fn default_value() -> Self;
}

impl Animatable for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }

    fn distance(&self, other: &Self) -> f32 {
        (self - other).abs()
    }

    fn default_value() -> Self {
        0.0
    }
}

impl Animatable for f64 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }

    fn distance(&self, other: &Self) -> f32 {
        (self - other).abs() as f32
    }

    fn default_value() -> Self {
        0.0
    }
}

impl Animatable for [f32; 2] {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        [
            self[0] + (other[0] - self[0]) * t,
            self[1] + (other[1] - self[1]) * t,
        ]
    }

    fn distance(&self, other: &Self) -> f32 {
        let dx = self[0] - other[0];
        let dy = self[1] - other[1];
        (dx * dx + dy * dy).sqrt()
    }

    fn default_value() -> Self {
        [0.0, 0.0]
    }
}

impl Animatable for [f32; 3] {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        [
            self[0] + (other[0] - self[0]) * t,
            self[1] + (other[1] - self[1]) * t,
            self[2] + (other[2] - self[2]) * t,
        ]
    }

    fn distance(&self, other: &Self) -> f32 {
        let dx = self[0] - other[0];
        let dy = self[1] - other[1];
        let dz = self[2] - other[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn default_value() -> Self {
        [0.0, 0.0, 0.0]
    }
}

impl Animatable for [f32; 4] {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        [
            self[0] + (other[0] - self[0]) * t,
            self[1] + (other[1] - self[1]) * t,
            self[2] + (other[2] - self[2]) * t,
            self[3] + (other[3] - self[3]) * t,
        ]
    }

    fn distance(&self, other: &Self) -> f32 {
        let dx = self[0] - other[0];
        let dy = self[1] - other[1];
        let dz = self[2] - other[2];
        let dw = self[3] - other[3];
        (dx * dx + dy * dy + dz * dz + dw * dw).sqrt()
    }

    fn default_value() -> Self {
        [0.0, 0.0, 0.0, 0.0]
    }
}

/// A row in the property tree (for DopeSheet).
#[derive(Debug, Clone)]
pub struct PropertyRow {
    /// Unique ID for this row.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Nesting depth (0 = root level).
    pub depth: usize,
    /// Whether this row can be collapsed (has children).
    pub can_collapse: bool,
    /// Whether this row is currently collapsed.
    pub is_collapsed: bool,
    /// Associated track ID (None for parent/group rows).
    pub track_id: Option<TrackId>,
    /// Optional color for this row's keyframes.
    pub color: Option<egui::Color32>,
}

/// Trait for providing animation data to widgets (read-only).
///
/// Implement this to connect your animation data to the DopeSheet and CurveEditor.
pub trait AnimationDataProvider {
    /// Get the list of property rows for the DopeSheet tree.
    fn property_rows(&self) -> Vec<PropertyRow>;

    /// Get keyframe positions for a track.
    ///
    /// Returns a list of (KeyframeId, position) tuples.
    fn keyframe_positions(&self, track_id: TrackId) -> Option<Vec<(KeyframeId, TimeTick)>>;

    /// Get the value at a specific keyframe (as f64 for display).
    fn keyframe_value(&self, track_id: TrackId, keyframe_id: KeyframeId) -> Option<f64>;

    /// Get the bezier handles for a keyframe.
    fn keyframe_handles(&self, track_id: TrackId, keyframe_id: KeyframeId)
        -> Option<BezierHandles>;

    /// Get the current time position.
    fn current_time(&self) -> TimeTick;

    /// Get the animation time range (start, end).
    fn time_range(&self) -> (TimeTick, TimeTick);

    /// Get the value range for a track (for curve editor scaling).
    fn value_range(&self, track_id: TrackId) -> Option<(f32, f32)>;
}

/// Commands for mutating animation data.
///
/// The host application receives these commands and applies them to the data model.
#[derive(Debug, Clone)]
pub enum AnimationCommand {
    /// Add a keyframe to a track.
    AddKeyframe {
        track_id: TrackId,
        position: TimeTick,
        value: f64,
    },
    /// Remove keyframes.
    RemoveKeyframes { keyframe_ids: Vec<KeyframeId> },
    /// Move a keyframe to a new position.
    MoveKeyframe {
        keyframe_id: KeyframeId,
        new_position: TimeTick,
    },
    /// Set a keyframe's value.
    SetKeyframeValue { keyframe_id: KeyframeId, value: f64 },
    /// Update bezier handles.
    SetKeyframeHandles {
        keyframe_id: KeyframeId,
        handles: BezierHandles,
    },
    /// Set the current time.
    SetCurrentTime(TimeTick),
    /// Toggle row collapse state.
    ToggleRowCollapse(String),

    /// Offset multiple keyframes by delta time and value.
    OffsetKeyframes {
        keyframe_ids: Vec<KeyframeId>,
        delta_time: TimeTick,
        delta_value: f64,
    },

    /// Scale multiple keyframes around an anchor point.
    ScaleKeyframes {
        keyframe_ids: Vec<KeyframeId>,
        anchor_time: TimeTick,
        anchor_value: f64,
        time_scale: f64,
        value_scale: f64,
    },

    /// Set the interpolation type for a keyframe.
    SetKeyframeType {
        keyframe_id: KeyframeId,
        keyframe_type: KeyframeType,
    },
}

/// Trait for mutating animation data.
///
/// Implement this to receive edit commands from the widgets.
pub trait AnimationDataMutator {
    /// Execute an animation command.
    fn execute(&mut self, command: AnimationCommand);

    /// Begin a scrub operation (for undo grouping).
    ///
    /// Multiple commands during a scrub are grouped as one undo action.
    fn begin_scrub(&mut self);

    /// End a scrub operation.
    ///
    /// If `commit` is true, the changes are committed. Otherwise they're discarded.
    fn end_scrub(&mut self, commit: bool);
}

// ===========================================================================
// KeyframeSource trait for CurveEditor (zero-copy keyframe access)
// ===========================================================================

/// A view into keyframe data for the CurveEditor.
///
/// This is an owned copy of keyframe properties, suitable for iteration
/// and rendering without holding references to the source data.
#[derive(Debug, Clone)]
pub struct KeyframeView {
    /// Unique identifier for this keyframe.
    pub id: KeyframeId,
    /// Time position.
    pub position: TimeTick,
    /// The value (as f32 for curve display).
    pub value: f32,
    /// Bezier control handles.
    pub handles: BezierHandles,
    /// Whether connected to the next keyframe.
    pub connected_right: bool,
    /// Interpolation type.
    pub keyframe_type: KeyframeType,
}

impl KeyframeView {
    /// Create a new KeyframeView.
    pub fn new(
        id: KeyframeId,
        position: TimeTick,
        value: f32,
        handles: BezierHandles,
        connected_right: bool,
        keyframe_type: KeyframeType,
    ) -> Self {
        Self {
            id,
            position,
            value,
            handles,
            connected_right,
            keyframe_type,
        }
    }
}

impl From<&Keyframe<f32>> for KeyframeView {
    fn from(kf: &Keyframe<f32>) -> Self {
        Self {
            id: kf.id,
            position: kf.position,
            value: kf.value,
            handles: kf.handles,
            connected_right: kf.connected_right,
            keyframe_type: kf.keyframe_type,
        }
    }
}

/// Trait for providing keyframe data to the CurveEditor.
///
/// This allows the CurveEditor to work with any keyframe source,
/// not just `Track<f32>`. Implement this for your animation data
/// to enable zero-copy curve editing.
pub trait KeyframeSource {
    /// Get keyframes in sorted order by position.
    ///
    /// Returns owned KeyframeView values so the source can be released
    /// after this call.
    fn keyframes_sorted(&self) -> Vec<KeyframeView>;

    /// Get the value range (min, max) for scaling the curve display.
    ///
    /// Returns None if there are no keyframes.
    fn value_range(&self) -> Option<(f32, f32)>;

    /// Number of keyframes.
    fn len(&self) -> usize;

    /// Check if empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Blanket implementation of KeyframeSource for Track<f32>.
impl KeyframeSource for Track<f32> {
    fn keyframes_sorted(&self) -> Vec<KeyframeView> {
        self.keyframes_sorted()
            .into_iter()
            .map(KeyframeView::from)
            .collect()
    }

    fn value_range(&self) -> Option<(f32, f32)> {
        Track::value_range(self)
    }

    fn len(&self) -> usize {
        Track::len(self)
    }
}
