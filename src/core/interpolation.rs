//! Bezier curve interpolation for keyframe animation.
//!
//! This module provides cubic bezier solving for smooth animation curves.

use super::keyframe::{Keyframe, KeyframeType};
use super::time::TimeTick;

/// Cubic bezier curve solver.
///
/// Uses Newton-Raphson iteration with bisection fallback for solving
/// the curve at a given x position.
#[derive(Debug, Clone, Copy)]
pub struct CubicBezier {
    cx: f32,
    bx: f32,
    ax: f32,
    cy: f32,
    by: f32,
    ay: f32,
}

impl CubicBezier {
    /// Create a bezier from control points.
    ///
    /// The curve goes from (0, 0) to (1, 1) with control points at
    /// (x1, y1) and (x2, y2).
    ///
    /// This is the same format as CSS `cubic-bezier(x1, y1, x2, y2)`.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let cx = 3.0 * x1;
        let bx = 3.0 * (x2 - x1) - cx;
        let ax = 1.0 - cx - bx;

        let cy = 3.0 * y1;
        let by = 3.0 * (y2 - y1) - cy;
        let ay = 1.0 - cy - by;

        Self {
            cx,
            bx,
            ax,
            cy,
            by,
            ay,
        }
    }

    /// Create a bezier from keyframe handles.
    ///
    /// # Arguments
    /// * `left_right_x`, `left_right_y` - Right handle of the left keyframe
    /// * `right_left_x`, `right_left_y` - Left handle of the right keyframe
    pub fn from_handles(
        left_right_x: f32,
        left_right_y: f32,
        right_left_x: f32,
        right_left_y: f32,
    ) -> Self {
        Self::new(left_right_x, left_right_y, right_left_x, right_left_y)
    }

    /// Linear bezier (straight line).
    pub fn linear() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    /// Ease-in bezier.
    pub fn ease_in() -> Self {
        Self::new(0.42, 0.0, 1.0, 1.0)
    }

    /// Ease-out bezier.
    pub fn ease_out() -> Self {
        Self::new(0.0, 0.0, 0.58, 1.0)
    }

    /// Ease-in-out bezier.
    pub fn ease_in_out() -> Self {
        Self::new(0.42, 0.0, 0.58, 1.0)
    }

    #[inline]
    fn sample_curve_x(&self, t: f32) -> f32 {
        ((self.ax * t + self.bx) * t + self.cx) * t
    }

    #[inline]
    fn sample_curve_y(&self, t: f32) -> f32 {
        ((self.ay * t + self.by) * t + self.cy) * t
    }

    #[inline]
    fn sample_curve_derivative_x(&self, t: f32) -> f32 {
        (3.0 * self.ax * t + 2.0 * self.bx) * t + self.cx
    }

    /// Solve for t given x using Newton-Raphson iteration.
    fn solve_curve_x(&self, x: f32) -> f32 {
        let mut t = x;

        // Newton-Raphson iteration
        for _ in 0..8 {
            let x_est = self.sample_curve_x(t) - x;
            if x_est.abs() < 1e-6 {
                return t;
            }
            let d = self.sample_curve_derivative_x(t);
            if d.abs() < 1e-6 {
                break;
            }
            t -= x_est / d;
        }

        // Fall back to bisection
        let mut lo = 0.0_f32;
        let mut hi = 1.0_f32;
        t = x;

        while lo < hi {
            let x_est = self.sample_curve_x(t);
            if (x_est - x).abs() < 1e-6 {
                return t;
            }
            if x > x_est {
                lo = t;
            } else {
                hi = t;
            }
            t = (lo + hi) / 2.0;

            // Prevent infinite loop
            if (hi - lo) < 1e-7 {
                break;
            }
        }

        t
    }

    /// Solve for y given x.
    ///
    /// This is the main entry point for evaluating the bezier curve.
    /// Given an x value in [0, 1], returns the corresponding y value.
    pub fn solve(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        let t = self.solve_curve_x(x);
        self.sample_curve_y(t)
    }
}

/// Result of interpolating between keyframes.
#[derive(Debug, Clone)]
pub struct InterpolationTriple<T> {
    /// Value at the left keyframe.
    pub left: T,
    /// Value at the right keyframe (if interpolating between two keyframes).
    pub right: Option<T>,
    /// Interpolation progression (0.0 to 1.0).
    ///
    /// This is the bezier-eased progression, not linear time.
    pub progression: f32,
}

impl<T: Clone> InterpolationTriple<T> {
    /// Get the interpolated value using linear interpolation.
    pub fn lerp(&self) -> T
    where
        T: Lerp,
    {
        match &self.right {
            Some(right) => self.left.lerp(right, self.progression),
            None => self.left.clone(),
        }
    }
}

/// Trait for linear interpolation.
pub trait Lerp {
    /// Linearly interpolate between self and other.
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Lerp for f64 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t as f64
    }
}

impl<const N: usize> Lerp for [f32; N] {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        let mut result = *self;
        for i in 0..N {
            result[i] = self[i] + (other[i] - self[i]) * t;
        }
        result
    }
}

/// Compute the interpolation triple at a given position.
///
/// # Arguments
/// * `keyframes` - Slice of keyframes, must be sorted by position
/// * `position` - Time position to evaluate at
///
/// # Returns
/// `None` if there are no keyframes, otherwise the interpolation triple.
pub fn interpolate_at_position<T: Clone>(
    keyframes: &[&Keyframe<T>],
    position: impl Into<TimeTick>,
) -> Option<InterpolationTriple<T>> {
    let position = position.into();

    if keyframes.is_empty() {
        return None;
    }

    // Find keyframes around position
    let mut left_idx = None;
    let mut right_idx = None;

    for (i, kf) in keyframes.iter().enumerate() {
        if kf.position <= position {
            left_idx = Some(i);
        } else if right_idx.is_none() {
            right_idx = Some(i);
            break;
        }
    }

    match (left_idx, right_idx) {
        // Before first keyframe - hold first value
        (None, Some(r)) => Some(InterpolationTriple {
            left: keyframes[r].value.clone(),
            right: None,
            progression: 0.0,
        }),

        // After last keyframe - hold last value
        (Some(l), None) => Some(InterpolationTriple {
            left: keyframes[l].value.clone(),
            right: None,
            progression: 0.0,
        }),

        // Between two keyframes
        (Some(l), Some(r)) => {
            let left_kf = keyframes[l];
            let right_kf = keyframes[r];

            // Check if connected
            if !left_kf.connected_right {
                return Some(InterpolationTriple {
                    left: left_kf.value.clone(),
                    right: None,
                    progression: 0.0,
                });
            }

            // Calculate local progression (0-1 between the two keyframes)
            let time_range = right_kf.position - left_kf.position;
            if time_range.value() <= 0.0 {
                return Some(InterpolationTriple {
                    left: left_kf.value.clone(),
                    right: None,
                    progression: 0.0,
                });
            }

            let local_pos = ((position - left_kf.position) / time_range) as f32;

            // Calculate value progression based on keyframe type
            let value_progression = match left_kf.keyframe_type {
                KeyframeType::Hold => 0.0,
                KeyframeType::Linear => local_pos,
                KeyframeType::Bezier => {
                    let bezier = CubicBezier::from_handles(
                        left_kf.handles.right_x,
                        left_kf.handles.right_y,
                        right_kf.handles.left_x,
                        right_kf.handles.left_y,
                    );
                    bezier.solve(local_pos)
                }
            };

            Some(InterpolationTriple {
                left: left_kf.value.clone(),
                right: Some(right_kf.value.clone()),
                progression: value_progression,
            })
        }

        // No keyframes (shouldn't happen if len > 0)
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::keyframe::BezierHandles;

    #[test]
    fn bezier_linear() {
        let bezier = CubicBezier::linear();
        assert!((bezier.solve(0.0) - 0.0).abs() < 1e-5);
        assert!((bezier.solve(0.5) - 0.5).abs() < 1e-5);
        assert!((bezier.solve(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn bezier_ease_in_out() {
        let bezier = CubicBezier::ease_in_out();
        // At midpoint, should be close to 0.5
        assert!((bezier.solve(0.5) - 0.5).abs() < 0.1);
        // At 0 and 1, should be exact
        assert!((bezier.solve(0.0) - 0.0).abs() < 1e-5);
        assert!((bezier.solve(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn interpolate_single_keyframe() {
        let kf = Keyframe::new(1.0, 42.0_f32);
        let keyframes: Vec<&Keyframe<f32>> = vec![&kf];

        // Before keyframe
        let result = interpolate_at_position(&keyframes, 0.0).unwrap();
        assert_eq!(result.left, 42.0);
        assert!(result.right.is_none());

        // After keyframe
        let result = interpolate_at_position(&keyframes, 2.0).unwrap();
        assert_eq!(result.left, 42.0);
        assert!(result.right.is_none());
    }

    #[test]
    fn interpolate_two_keyframes_linear() {
        let kf1 = Keyframe::new(0.0, 0.0_f32).with_type(KeyframeType::Linear);
        let kf2 = Keyframe::new(1.0, 100.0_f32);
        let keyframes: Vec<&Keyframe<f32>> = vec![&kf1, &kf2];

        let result = interpolate_at_position(&keyframes, 0.5).unwrap();
        assert_eq!(result.left, 0.0);
        assert_eq!(result.right.unwrap(), 100.0);
        assert!((result.progression - 0.5).abs() < 1e-5);

        let lerped = result.lerp();
        assert!((lerped - 50.0).abs() < 1e-5);
    }

    #[test]
    fn interpolate_hold_keyframe() {
        let kf1 = Keyframe::new(0.0, 10.0_f32).with_type(KeyframeType::Hold);
        let kf2 = Keyframe::new(1.0, 100.0_f32);
        let keyframes: Vec<&Keyframe<f32>> = vec![&kf1, &kf2];

        let result = interpolate_at_position(&keyframes, 0.5).unwrap();
        // Hold should have progression 0.
        assert_eq!(result.progression, 0.0);

        let lerped = result.lerp();
        // Should hold at left value.
        assert_eq!(lerped, 10.0);
    }

    #[test]
    fn interpolate_bezier_keyframe() {
        let kf1 = Keyframe::new(0.0, 0.0_f32)
            .with_handles(BezierHandles::ease_in_out())
            .with_type(KeyframeType::Bezier);
        let kf2 = Keyframe::new(1.0, 100.0_f32).with_handles(BezierHandles::ease_in_out());
        let keyframes: Vec<&Keyframe<f32>> = vec![&kf1, &kf2];

        let result = interpolate_at_position(&keyframes, 0.5).unwrap();
        // Ease-in-out at midpoint should be close to 0.5 but eased
        assert!(result.progression >= 0.0 && result.progression <= 1.0);
    }
}
