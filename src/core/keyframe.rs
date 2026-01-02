//! Keyframe data structures.
//!
//! A keyframe stores a value at a specific time position, along with
//! bezier handles for smooth interpolation.

use super::time::TimeTick;
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Unique identifier for a keyframe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KeyframeId(pub Uuid);

impl KeyframeId {
    /// Create a new random keyframe ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for KeyframeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Bezier control handles for a keyframe.
///
/// The handles control the shape of the interpolation curve between keyframes.
/// Values are normalized to the segment between this keyframe and the next:
/// - X values are in [0, 1] representing time within the segment
/// - Y values are unbounded, representing the value curve shape
///
/// This follows the Theatre.js convention: `[leftX, leftY, rightX, rightY]`
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BezierHandles {
    /// Left handle X (0.0 to 1.0), controls incoming curve tangent position.
    pub left_x: f32,
    /// Left handle Y (unbounded), controls incoming curve tangent height.
    pub left_y: f32,
    /// Right handle X (0.0 to 1.0), controls outgoing curve tangent position.
    pub right_x: f32,
    /// Right handle Y (unbounded), controls outgoing curve tangent height.
    pub right_y: f32,
}

impl Default for BezierHandles {
    fn default() -> Self {
        Self::linear()
    }
}

impl BezierHandles {
    /// Create handles from array `[left_x, left_y, right_x, right_y]`.
    pub fn from_array(arr: [f32; 4]) -> Self {
        Self {
            left_x: arr[0],
            left_y: arr[1],
            right_x: arr[2],
            right_y: arr[3],
        }
    }

    /// Convert handles to array `[left_x, left_y, right_x, right_y]`.
    pub fn to_array(&self) -> [f32; 4] {
        [self.left_x, self.left_y, self.right_x, self.right_y]
    }

    /// Linear interpolation (straight line between keyframes).
    pub fn linear() -> Self {
        Self {
            left_x: 0.0,
            left_y: 0.0,
            right_x: 1.0,
            right_y: 1.0,
        }
    }

    /// Ease in (slow start, fast end).
    pub fn ease_in() -> Self {
        Self {
            left_x: 0.0,
            left_y: 0.0,
            right_x: 0.42,
            right_y: 0.0,
        }
    }

    /// Ease out (fast start, slow end).
    pub fn ease_out() -> Self {
        Self {
            left_x: 0.58,
            left_y: 1.0,
            right_x: 1.0,
            right_y: 1.0,
        }
    }

    /// Ease in-out (slow start and end).
    pub fn ease_in_out() -> Self {
        Self {
            left_x: 0.42,
            left_y: 0.0,
            right_x: 0.58,
            right_y: 1.0,
        }
    }

    /// CSS cubic-bezier format: `cubic-bezier(x1, y1, x2, y2)`.
    ///
    /// Note: CSS format uses right handle of start point and left handle of end point.
    pub fn from_css(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            left_x: 1.0 - x2,
            left_y: 1.0 - y2,
            right_x: x1,
            right_y: y1,
        }
    }
}

/// The interpolation type between keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum KeyframeType {
    /// Bezier curve interpolation using the control handles.
    #[default]
    Bezier,
    /// Hold the value until the next keyframe (step function).
    Hold,
    /// Linear interpolation (ignore bezier handles).
    Linear,
}

/// A keyframe storing a value at a specific time position.
///
/// The generic type `T` is the value type being animated (e.g., `f32`, `[f32; 3]`).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Keyframe<T> {
    /// Unique identifier for this keyframe.
    pub id: KeyframeId,
    /// The value at this keyframe.
    pub value: T,
    /// Time position.
    pub position: TimeTick,
    /// Bezier control handles for interpolation.
    pub handles: BezierHandles,
    /// Whether this keyframe is connected to the next one.
    ///
    /// If `false`, there's a gap and the value holds until the next keyframe.
    pub connected_right: bool,
    /// The interpolation type for the curve leaving this keyframe.
    pub keyframe_type: KeyframeType,
}

impl<T: Default> Keyframe<T> {
    /// Create a new keyframe with the given position and value.
    pub fn new(position: impl Into<TimeTick>, value: T) -> Self {
        Self {
            id: KeyframeId::new(),
            value,
            position: position.into(),
            handles: BezierHandles::default(),
            connected_right: true,
            keyframe_type: KeyframeType::default(),
        }
    }
}

impl<T> Keyframe<T> {
    /// Create a keyframe with a specific ID.
    pub fn with_id(id: KeyframeId, position: impl Into<TimeTick>, value: T) -> Self {
        Self {
            id,
            value,
            position: position.into(),
            handles: BezierHandles::default(),
            connected_right: true,
            keyframe_type: KeyframeType::default(),
        }
    }

    /// Set the bezier handles.
    pub fn with_handles(mut self, handles: BezierHandles) -> Self {
        self.handles = handles;
        self
    }

    /// Set the keyframe type.
    pub fn with_type(mut self, keyframe_type: KeyframeType) -> Self {
        self.keyframe_type = keyframe_type;
        self
    }

    /// Set whether this keyframe is connected to the next.
    pub fn with_connected(mut self, connected: bool) -> Self {
        self.connected_right = connected;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyframe_creation() {
        let kf = Keyframe::new(1.5, 42.0_f32);
        assert_eq!(kf.position, TimeTick::new(1.5));
        assert_eq!(kf.value, 42.0);
        assert!(kf.connected_right);
        assert_eq!(kf.keyframe_type, KeyframeType::Bezier);
    }

    #[test]
    fn handles_presets() {
        let linear = BezierHandles::linear();
        assert_eq!(linear.right_x, 1.0);
        assert_eq!(linear.right_y, 1.0);

        let ease_in = BezierHandles::ease_in();
        assert_eq!(ease_in.right_x, 0.42);
        assert_eq!(ease_in.right_y, 0.0);
    }

    #[test]
    fn handles_array_conversion() {
        let handles = BezierHandles::ease_in_out();
        let arr = handles.to_array();
        let restored = BezierHandles::from_array(arr);
        assert_eq!(handles, restored);
    }
}
