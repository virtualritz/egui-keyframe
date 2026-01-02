//! Easing presets for animation curves.
//!
//! This module provides common easing functions as bezier control points.

use super::keyframe::BezierHandles;

/// Named easing preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EasingPreset {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
}

impl EasingPreset {
    /// Get the display name for this preset.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::EaseIn => "Ease In",
            Self::EaseOut => "Ease Out",
            Self::EaseInOut => "Ease In Out",
            Self::EaseInQuad => "Ease In Quad",
            Self::EaseOutQuad => "Ease Out Quad",
            Self::EaseInOutQuad => "Ease In Out Quad",
            Self::EaseInCubic => "Ease In Cubic",
            Self::EaseOutCubic => "Ease Out Cubic",
            Self::EaseInOutCubic => "Ease In Out Cubic",
            Self::EaseInQuart => "Ease In Quart",
            Self::EaseOutQuart => "Ease Out Quart",
            Self::EaseInOutQuart => "Ease In Out Quart",
            Self::EaseInQuint => "Ease In Quint",
            Self::EaseOutQuint => "Ease Out Quint",
            Self::EaseInOutQuint => "Ease In Out Quint",
            Self::EaseInSine => "Ease In Sine",
            Self::EaseOutSine => "Ease Out Sine",
            Self::EaseInOutSine => "Ease In Out Sine",
            Self::EaseInExpo => "Ease In Expo",
            Self::EaseOutExpo => "Ease Out Expo",
            Self::EaseInOutExpo => "Ease In Out Expo",
            Self::EaseInCirc => "Ease In Circ",
            Self::EaseOutCirc => "Ease Out Circ",
            Self::EaseInOutCirc => "Ease In Out Circ",
            Self::EaseInBack => "Ease In Back",
            Self::EaseOutBack => "Ease Out Back",
            Self::EaseInOutBack => "Ease In Out Back",
        }
    }

    /// Get the bezier handles for this preset.
    pub fn handles(&self) -> BezierHandles {
        // CSS cubic-bezier values from easings.net
        let (x1, y1, x2, y2) = match self {
            Self::Linear => (0.0, 0.0, 1.0, 1.0),
            Self::EaseIn => (0.42, 0.0, 1.0, 1.0),
            Self::EaseOut => (0.0, 0.0, 0.58, 1.0),
            Self::EaseInOut => (0.42, 0.0, 0.58, 1.0),
            Self::EaseInQuad => (0.55, 0.085, 0.68, 0.53),
            Self::EaseOutQuad => (0.25, 0.46, 0.45, 0.94),
            Self::EaseInOutQuad => (0.455, 0.03, 0.515, 0.955),
            Self::EaseInCubic => (0.55, 0.055, 0.675, 0.19),
            Self::EaseOutCubic => (0.215, 0.61, 0.355, 1.0),
            Self::EaseInOutCubic => (0.645, 0.045, 0.355, 1.0),
            Self::EaseInQuart => (0.895, 0.03, 0.685, 0.22),
            Self::EaseOutQuart => (0.165, 0.84, 0.44, 1.0),
            Self::EaseInOutQuart => (0.77, 0.0, 0.175, 1.0),
            Self::EaseInQuint => (0.755, 0.05, 0.855, 0.06),
            Self::EaseOutQuint => (0.23, 1.0, 0.32, 1.0),
            Self::EaseInOutQuint => (0.86, 0.0, 0.07, 1.0),
            Self::EaseInSine => (0.47, 0.0, 0.745, 0.715),
            Self::EaseOutSine => (0.39, 0.575, 0.565, 1.0),
            Self::EaseInOutSine => (0.445, 0.05, 0.55, 0.95),
            Self::EaseInExpo => (0.95, 0.05, 0.795, 0.035),
            Self::EaseOutExpo => (0.19, 1.0, 0.22, 1.0),
            Self::EaseInOutExpo => (1.0, 0.0, 0.0, 1.0),
            Self::EaseInCirc => (0.6, 0.04, 0.98, 0.335),
            Self::EaseOutCirc => (0.075, 0.82, 0.165, 1.0),
            Self::EaseInOutCirc => (0.785, 0.135, 0.15, 0.86),
            Self::EaseInBack => (0.6, -0.28, 0.735, 0.045),
            Self::EaseOutBack => (0.175, 0.885, 0.32, 1.275),
            Self::EaseInOutBack => (0.68, -0.55, 0.265, 1.55),
        };

        // Convert CSS cubic-bezier to our handle format
        BezierHandles {
            left_x: 1.0 - x2,
            left_y: 1.0 - y2,
            right_x: x1,
            right_y: y1,
        }
    }

    /// Get all presets.
    pub fn all() -> &'static [Self] {
        &[
            Self::Linear,
            Self::EaseIn,
            Self::EaseOut,
            Self::EaseInOut,
            Self::EaseInQuad,
            Self::EaseOutQuad,
            Self::EaseInOutQuad,
            Self::EaseInCubic,
            Self::EaseOutCubic,
            Self::EaseInOutCubic,
            Self::EaseInQuart,
            Self::EaseOutQuart,
            Self::EaseInOutQuart,
            Self::EaseInQuint,
            Self::EaseOutQuint,
            Self::EaseInOutQuint,
            Self::EaseInSine,
            Self::EaseOutSine,
            Self::EaseInOutSine,
            Self::EaseInExpo,
            Self::EaseOutExpo,
            Self::EaseInOutExpo,
            Self::EaseInCirc,
            Self::EaseOutCirc,
            Self::EaseInOutCirc,
            Self::EaseInBack,
            Self::EaseOutBack,
            Self::EaseInOutBack,
        ]
    }

    /// Get common presets (subset for UI).
    pub fn common() -> &'static [Self] {
        &[
            Self::Linear,
            Self::EaseIn,
            Self::EaseOut,
            Self::EaseInOut,
            Self::EaseInCubic,
            Self::EaseOutCubic,
            Self::EaseInOutCubic,
            Self::EaseInBack,
            Self::EaseOutBack,
        ]
    }
}

/// Check if two handle sets are approximately equal.
pub fn handles_similar(a: &BezierHandles, b: &BezierHandles, tolerance: f32) -> bool {
    (a.left_x - b.left_x).abs() < tolerance
        && (a.left_y - b.left_y).abs() < tolerance
        && (a.right_x - b.right_x).abs() < tolerance
        && (a.right_y - b.right_y).abs() < tolerance
}

/// Try to match handles to a known preset.
pub fn match_preset(handles: &BezierHandles, tolerance: f32) -> Option<EasingPreset> {
    for preset in EasingPreset::all() {
        if handles_similar(handles, &preset.handles(), tolerance) {
            return Some(*preset);
        }
    }
    None
}
