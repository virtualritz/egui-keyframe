//! Coordinate space transformations for timeline UI.
//!
//! This module provides the [`SpaceTransform`] struct for converting between:
//! - **Unit space**: Animation time ([`TimeTick`])
//! - **Scaled space**: Unit space with zoom applied (pixels)
//! - **Clipped space**: Scaled space with scroll offset (screen pixels)
//!
//! This follows the Theatre.js coordinate space pattern.

use crate::TimeTick;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "facet")]
use facet::Facet;

/// Coordinate space transformation for timeline UI.
///
/// Converts between animation time (unit space) and screen coordinates (clipped space).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "facet", derive(Facet))]
pub struct SpaceTransform {
    /// Pixels per time unit (zoom level).
    pub pixels_per_unit: f64,
    /// Scroll offset in unit space (time).
    pub scroll_offset: TimeTick,
    /// Left padding in pixels.
    pub left_padding: f32,
    /// Visible width in pixels.
    pub visible_width: f32,
}

impl Default for SpaceTransform {
    fn default() -> Self {
        Self {
            pixels_per_unit: 100.0,
            scroll_offset: TimeTick::default(),
            left_padding: 0.0,
            visible_width: 400.0,
        }
    }
}

impl SpaceTransform {
    /// Create a new space transform.
    ///
    /// # Arguments
    /// * `pixels_per_unit` - Zoom level (pixels per time unit)
    /// * `scroll_offset` - Pan offset in time units
    /// * `visible_width` - Width of the visible area in pixels
    pub fn new(
        pixels_per_unit: f64,
        scroll_offset: impl Into<TimeTick>,
        visible_width: f32,
    ) -> Self {
        Self {
            pixels_per_unit,
            scroll_offset: scroll_offset.into(),
            left_padding: 0.0,
            visible_width,
        }
    }

    /// Set the left padding.
    pub fn with_left_padding(mut self, padding: f32) -> Self {
        self.left_padding = padding;
        self
    }

    // -------------------------------------------------------------------------
    // Unit Space <-> Scaled Space
    // -------------------------------------------------------------------------

    /// Convert from unit space (time) to scaled space (pixels, no scroll).
    #[inline]
    pub fn unit_to_scaled(&self, unit: TimeTick) -> f64 {
        unit.value() * self.pixels_per_unit
    }

    /// Convert from scaled space (pixels) to unit space (time).
    #[inline]
    pub fn scaled_to_unit(&self, scaled: f64) -> TimeTick {
        TimeTick::new(scaled / self.pixels_per_unit)
    }

    // -------------------------------------------------------------------------
    // Unit Space <-> Clipped Space (Screen)
    // -------------------------------------------------------------------------

    /// Convert from unit space (time) to clipped space (screen x coordinate).
    #[inline]
    pub fn unit_to_clipped(&self, unit: TimeTick) -> f32 {
        let scaled = self.unit_to_scaled(unit - self.scroll_offset);
        (scaled as f32) + self.left_padding
    }

    /// Convert from clipped space (screen x) to unit space (time).
    #[inline]
    pub fn clipped_to_unit(&self, clipped: f32) -> TimeTick {
        let scaled = (clipped - self.left_padding) as f64;
        self.scaled_to_unit(scaled) + self.scroll_offset
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Get the visible time range in unit space.
    pub fn visible_range(&self) -> (TimeTick, TimeTick) {
        let start = self.scroll_offset;
        let end = start + self.scaled_to_unit(self.visible_width as f64);
        (start, end)
    }

    /// Check if a time value is visible.
    pub fn is_visible(&self, unit: TimeTick) -> bool {
        let (start, end) = self.visible_range();
        unit >= start && unit <= end
    }

    /// Get the width of one time unit in pixels.
    #[inline]
    pub fn unit_width(&self) -> f32 {
        self.pixels_per_unit as f32
    }

    // -------------------------------------------------------------------------
    // Modifications
    // -------------------------------------------------------------------------

    /// Zoom around a specific screen position.
    ///
    /// # Arguments
    /// * `clipped_x` - Screen x coordinate to zoom around
    /// * `zoom_factor` - Factor to multiply zoom by (>1 = zoom in, <1 = zoom out)
    pub fn zoom_at(&self, clipped_x: f32, zoom_factor: f64) -> Self {
        let unit_at_mouse = self.clipped_to_unit(clipped_x);
        let new_pixels_per_unit = (self.pixels_per_unit * zoom_factor).clamp(1.0, 10000.0);

        // Calculate new scroll offset to keep unit_at_mouse at the same screen position
        let screen_offset = clipped_x - self.left_padding;
        let new_scroll = unit_at_mouse - TimeTick::new(screen_offset as f64 / new_pixels_per_unit);

        Self {
            pixels_per_unit: new_pixels_per_unit,
            scroll_offset: new_scroll,
            left_padding: self.left_padding,
            visible_width: self.visible_width,
        }
    }

    /// Pan by a screen delta (pixels).
    pub fn pan(&self, delta_x: f32) -> Self {
        let delta_unit = self.scaled_to_unit(-delta_x as f64);
        Self {
            pixels_per_unit: self.pixels_per_unit,
            scroll_offset: self.scroll_offset + delta_unit,
            left_padding: self.left_padding,
            visible_width: self.visible_width,
        }
    }

    /// Set the visible width (call when the widget resizes).
    pub fn with_visible_width(mut self, width: f32) -> Self {
        self.visible_width = width;
        self
    }

    /// Fit a time range to the visible area with some padding.
    pub fn fit_range(
        &self,
        start: impl Into<TimeTick>,
        end: impl Into<TimeTick>,
        padding_fraction: f64,
    ) -> Self {
        let start = start.into();
        let end = end.into();
        let range = (end - start).value();
        let padded_range = range * (1.0 + 2.0 * padding_fraction);
        let new_pixels_per_unit = self.visible_width as f64 / padded_range;
        let new_scroll = start - TimeTick::new(range * padding_fraction);

        Self {
            pixels_per_unit: new_pixels_per_unit.clamp(1.0, 10000.0),
            scroll_offset: new_scroll,
            left_padding: self.left_padding,
            visible_width: self.visible_width,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_to_clipped_roundtrip() {
        let transform = SpaceTransform::new(100.0, 0.0, 400.0);

        let unit = TimeTick::new(2.5);
        let clipped = transform.unit_to_clipped(unit);
        let back = transform.clipped_to_unit(clipped);

        assert!((back - unit).value().abs() < 1e-10);
    }

    #[test]
    fn visible_range() {
        let transform = SpaceTransform::new(100.0, 1.0, 200.0);
        let (start, end) = transform.visible_range();

        assert_eq!(start, TimeTick::new(1.0));
        // 1.0 + 200/100 = 3.0.
        assert!((end.value() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn zoom_at_center() {
        let transform = SpaceTransform::new(100.0, 0.0, 400.0);
        // Zoom in 2x at center.
        let zoomed = transform.zoom_at(200.0, 2.0);

        // After zoom, pixels_per_unit should double.
        assert!((zoomed.pixels_per_unit - 200.0).abs() < 1e-10);

        // The time at the zoom point should remain the same.
        let time_before = transform.clipped_to_unit(200.0);
        let time_after = zoomed.clipped_to_unit(200.0);
        assert!((time_before - time_after).value().abs() < 1e-10);
    }

    #[test]
    fn pan() {
        let transform = SpaceTransform::new(100.0, 0.0, 400.0);
        // Pan right by 100 pixels.
        let panned = transform.pan(-100.0);

        // Scroll should increase by 1 unit (100 pixels / 100 ppu).
        assert!((panned.scroll_offset.value() - 1.0).abs() < 1e-10);
    }
}
