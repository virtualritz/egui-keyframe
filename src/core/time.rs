//! Time position type for keyframe animation.
//!
//! `TimeTick` is a unit-agnostic time position that can represent seconds,
//! frames, or any other time unit depending on the application's needs.
//!
//! # Feature Flags
//!
//! - `serde`: Enables serialization/deserialization via serde
//! - `facet`: Enables reflection via the facet crate
//! - `frame-tick`: Uses `frame_tick::Tick` as the underlying storage instead of `f64`

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "facet")]
use facet::Facet;

use std::ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// =============================================================================
// Inner type alias
// =============================================================================

#[cfg(not(feature = "frame-tick"))]
type Inner = f64;

#[cfg(feature = "frame-tick")]
type Inner = frame_tick::Tick;

// =============================================================================
// TimeTick struct definition
// =============================================================================

/// A position in time, agnostic to the underlying unit (seconds, frames, etc.).
///
/// `TimeTick` wraps a time value and provides type safety for time-related
/// operations. The interpretation of the value (seconds, frames, beats, etc.)
/// is left to the application.
///
/// When the `frame-tick` feature is enabled, this wraps `frame_tick::Tick`
/// for enhanced precision and frame-rate handling. Otherwise, it wraps `f64`.
///
/// Use [`Deref`] to access the inner type directly for operations not
/// exposed by `TimeTick`.
///
/// # Examples
///
/// ```
/// use egui_keyframe::TimeTick;
///
/// // Create time positions
/// let t1 = TimeTick::new(1.5);
/// let t2 = TimeTick::new(2.0);
///
/// // Arithmetic operations
/// let sum = t1 + t2;
/// let diff = t2 - t1;
/// let scaled = t1 * 2.0;
///
/// // Access raw value
/// assert_eq!(t1.value(), 1.5);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "facet", derive(Facet))]
#[repr(transparent)]
pub struct TimeTick(Inner);

// =============================================================================
// Deref/DerefMut - access inner type
// =============================================================================

impl Deref for TimeTick {
    type Target = Inner;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TimeTick {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Inner> for TimeTick {
    #[inline]
    fn as_ref(&self) -> &Inner {
        &self.0
    }
}

impl AsMut<Inner> for TimeTick {
    #[inline]
    fn as_mut(&mut self) -> &mut Inner {
        &mut self.0
    }
}

// =============================================================================
// TimeTick core implementation - f64 backend
// =============================================================================

#[cfg(not(feature = "frame-tick"))]
impl TimeTick {
    /// Zero time position.
    pub const ZERO: Self = Self(0.0);

    /// Zero time position (function form, works with both backends).
    #[inline]
    pub fn zero() -> Self {
        Self::ZERO
    }

    /// Create a new time tick from a raw value.
    #[inline]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Wrap an inner value.
    #[inline]
    pub const fn from_inner(inner: f64) -> Self {
        Self(inner)
    }

    /// Get the raw value as f64.
    #[inline]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Create from seconds.
    #[inline]
    pub fn from_seconds<T: Into<f64>>(secs: T) -> Self {
        Self(secs.into())
    }

    /// Create from frames at a given frame rate.
    #[inline]
    pub fn from_frames<T: Into<f64>, F: Into<f64>>(frames: T, fps: F) -> Self {
        Self(frames.into() / fps.into())
    }

    /// Convert to frames at a given frame rate.
    #[inline]
    pub fn to_frames<F: Into<f64>>(self, fps: F) -> f64 {
        self.0 * fps.into()
    }

    /// Linear interpolation between two time ticks.
    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self(self.0 + (other.0 - self.0) * t)
    }

    /// Absolute value.
    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Minimum of two time ticks.
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Maximum of two time ticks.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    /// Clamp to range.
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }

    /// Check if finite.
    #[inline]
    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    /// Round to nearest integer.
    #[inline]
    pub fn round(self) -> Self {
        Self(self.0.round())
    }

    /// Floor to integer.
    #[inline]
    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }

    /// Ceiling to integer.
    #[inline]
    pub fn ceil(self) -> Self {
        Self(self.0.ceil())
    }
}

// =============================================================================
// TimeTick core implementation - frame_tick::Tick backend
// =============================================================================

#[cfg(feature = "frame-tick")]
impl TimeTick {
    // Note: Can't use const ZERO with frame_tick::Tick as Tick::new isn't const.
    // Use TimeTick::default() or TimeTick::zero() instead.

    /// Zero time position.
    #[inline]
    pub fn zero() -> Self {
        Self::default()
    }

    /// Create a new time tick from a raw value (interpreted as seconds).
    #[inline]
    pub fn new(value: f64) -> Self {
        Self(frame_tick::Tick::from_secs(value))
    }

    /// Wrap an inner value.
    #[inline]
    pub const fn from_inner(inner: frame_tick::Tick) -> Self {
        Self(inner)
    }

    /// Get the raw value as f64 (in seconds).
    #[inline]
    pub fn value(self) -> f64 {
        self.0.to_secs()
    }

    /// Create from seconds.
    #[inline]
    pub fn from_seconds<T: Into<f64>>(secs: T) -> Self {
        Self(frame_tick::Tick::from_secs(secs.into()))
    }

    /// Create from frames at a given frame rate.
    #[inline]
    pub fn from_frames<T: Into<f64>, F: Into<f64>>(frames: T, fps: F) -> Self {
        Self(frame_tick::Tick::from_secs(frames.into() / fps.into()))
    }

    /// Convert to frames at a given frame rate.
    #[inline]
    pub fn to_frames<F: Into<f64>>(self, fps: F) -> f64 {
        self.0.to_secs() * fps.into()
    }

    /// Linear interpolation between two time ticks.
    #[inline]
    pub fn lerp(self, other: Self, t: f64) -> Self {
        let a = self.0.to_secs();
        let b = other.0.to_secs();
        Self(frame_tick::Tick::from_secs(a + (b - a) * t))
    }

    /// Absolute value.
    #[inline]
    pub fn abs(self) -> Self {
        Self(frame_tick::Tick::from_secs(self.0.to_secs().abs()))
    }

    /// Minimum of two time ticks.
    #[inline]
    pub fn min(self, other: Self) -> Self {
        if self.0 < other.0 { self } else { other }
    }

    /// Maximum of two time ticks.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        if self.0 > other.0 { self } else { other }
    }

    /// Clamp to range.
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }

    /// Check if finite (always true for frame_tick::Tick).
    #[inline]
    pub fn is_finite(self) -> bool {
        true
    }

    /// Round to nearest integer.
    #[inline]
    pub fn round(self) -> Self {
        Self(frame_tick::Tick::from_secs(self.0.to_secs().round()))
    }

    /// Floor to integer.
    #[inline]
    pub fn floor(self) -> Self {
        Self(frame_tick::Tick::from_secs(self.0.to_secs().floor()))
    }

    /// Ceiling to integer.
    #[inline]
    pub fn ceil(self) -> Self {
        Self(frame_tick::Tick::from_secs(self.0.to_secs().ceil()))
    }

    /// Get the underlying `frame_tick::Tick`.
    #[inline]
    pub const fn as_tick(self) -> frame_tick::Tick {
        self.0
    }
}

// =============================================================================
// From implementations
// =============================================================================

impl From<Inner> for TimeTick {
    #[inline]
    fn from(inner: Inner) -> Self {
        Self(inner)
    }
}

impl From<TimeTick> for Inner {
    #[inline]
    fn from(tick: TimeTick) -> Self {
        tick.0
    }
}

#[cfg(not(feature = "frame-tick"))]
impl From<f32> for TimeTick {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}

#[cfg(not(feature = "frame-tick"))]
impl From<i32> for TimeTick {
    #[inline]
    fn from(value: i32) -> Self {
        Self(value as f64)
    }
}

#[cfg(not(feature = "frame-tick"))]
impl From<i64> for TimeTick {
    #[inline]
    fn from(value: i64) -> Self {
        Self(value as f64)
    }
}

#[cfg(feature = "frame-tick")]
impl From<f64> for TimeTick {
    #[inline]
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

#[cfg(feature = "frame-tick")]
impl From<f32> for TimeTick {
    #[inline]
    fn from(value: f32) -> Self {
        Self::new(value as f64)
    }
}

#[cfg(feature = "frame-tick")]
impl From<i32> for TimeTick {
    #[inline]
    fn from(value: i32) -> Self {
        Self::new(value as f64)
    }
}

#[cfg(feature = "frame-tick")]
impl From<i64> for TimeTick {
    #[inline]
    fn from(value: i64) -> Self {
        Self::new(value as f64)
    }
}

#[cfg(feature = "frame-tick")]
impl From<TimeTick> for f64 {
    #[inline]
    fn from(tick: TimeTick) -> Self {
        tick.value()
    }
}

// =============================================================================
// Arithmetic operations - delegate to inner, wrap result
// =============================================================================

impl Add for TimeTick {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for TimeTick {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl Sub for TimeTick {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for TimeTick {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 - rhs.0;
    }
}

// Mul/Div by f64 - need backend-specific impl

#[cfg(not(feature = "frame-tick"))]
impl Mul<f64> for TimeTick {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[cfg(not(feature = "frame-tick"))]
impl Mul<TimeTick> for f64 {
    type Output = TimeTick;

    #[inline]
    fn mul(self, rhs: TimeTick) -> Self::Output {
        TimeTick(self * rhs.0)
    }
}

#[cfg(not(feature = "frame-tick"))]
impl MulAssign<f64> for TimeTick {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
    }
}

#[cfg(not(feature = "frame-tick"))]
impl Div<f64> for TimeTick {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

#[cfg(not(feature = "frame-tick"))]
impl DivAssign<f64> for TimeTick {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.0 /= rhs;
    }
}

#[cfg(not(feature = "frame-tick"))]
impl Div for TimeTick {
    type Output = f64;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
    }
}

#[cfg(not(feature = "frame-tick"))]
impl Neg for TimeTick {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

#[cfg(feature = "frame-tick")]
impl Mul<f64> for TimeTick {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.value() * rhs)
    }
}

#[cfg(feature = "frame-tick")]
impl Mul<TimeTick> for f64 {
    type Output = TimeTick;

    #[inline]
    fn mul(self, rhs: TimeTick) -> Self::Output {
        TimeTick::new(self * rhs.value())
    }
}

#[cfg(feature = "frame-tick")]
impl MulAssign<f64> for TimeTick {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        *self = Self::new(self.value() * rhs);
    }
}

#[cfg(feature = "frame-tick")]
impl Div<f64> for TimeTick {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self::new(self.value() / rhs)
    }
}

#[cfg(feature = "frame-tick")]
impl DivAssign<f64> for TimeTick {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        *self = Self::new(self.value() / rhs);
    }
}

#[cfg(feature = "frame-tick")]
impl Div for TimeTick {
    type Output = f64;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.value() / rhs.value()
    }
}

#[cfg(feature = "frame-tick")]
impl Neg for TimeTick {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.value())
    }
}

// =============================================================================
// Display
// =============================================================================

impl std::fmt::Display for TimeTick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_operations() {
        let t1 = TimeTick::new(1.0);
        let t2 = TimeTick::new(2.0);

        assert_eq!((t1 + t2).value(), 3.0);
        assert_eq!((t2 - t1).value(), 1.0);
        assert_eq!((t1 * 3.0).value(), 3.0);
        assert_eq!((t2 / 2.0).value(), 1.0);
        assert_eq!(t2 / t1, 2.0);
    }

    #[test]
    fn frame_conversion() {
        let t = TimeTick::from_frames(60.0, 30.0);
        assert_eq!(t.value(), 2.0); // 60 frames at 30fps = 2 seconds

        let frames = t.to_frames(30.0);
        assert_eq!(frames, 60.0);
    }

    #[test]
    fn from_seconds_generic() {
        let t1 = TimeTick::from_seconds(1.5_f64);
        let t2 = TimeTick::from_seconds(1.5_f32);
        let t3 = TimeTick::from_seconds(2_i32);

        assert_eq!(t1.value(), 1.5);
        assert_eq!(t2.value(), 1.5);
        assert_eq!(t3.value(), 2.0);
    }

    #[test]
    fn deref_access() {
        let t = TimeTick::new(1.5);
        // Access inner type via deref
        let _inner: &Inner = &*t;
    }

    #[test]
    fn from_inner() {
        #[cfg(not(feature = "frame-tick"))]
        let t = TimeTick::from_inner(1.5);
        #[cfg(feature = "frame-tick")]
        let t = TimeTick::from_inner(frame_tick::Tick::from_secs(1.5));

        assert_eq!(t.value(), 1.5);
    }
}
