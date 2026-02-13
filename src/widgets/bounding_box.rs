//! Bounding box widget for multi-keyframe selection transforms.
//!
//! Provides a visual bounding box around selected keyframes with handles
//! for offset (translate) and scale operations.

use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Which handle of the bounding box is being interacted with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundingBoxHandle {
    /// Top-left corner (scales both axes).
    TopLeft,
    /// Top edge center (scales Y only).
    Top,
    /// Top-right corner (scales both axes).
    TopRight,
    /// Left edge center (scales X only).
    Left,
    /// Right edge center (scales X only).
    Right,
    /// Bottom-left corner (scales both axes).
    BottomLeft,
    /// Bottom edge center (scales Y only).
    Bottom,
    /// Bottom-right corner (scales both axes).
    BottomRight,
    /// Interior (offset/translate).
    Interior,
}

impl BoundingBoxHandle {
    /// Returns true if this handle scales the X (time) axis.
    pub fn scales_x(&self) -> bool {
        matches!(
            self,
            Self::TopLeft
                | Self::TopRight
                | Self::Left
                | Self::Right
                | Self::BottomLeft
                | Self::BottomRight
        )
    }

    /// Returns true if this handle scales the Y (value) axis.
    pub fn scales_y(&self) -> bool {
        matches!(
            self,
            Self::TopLeft
                | Self::Top
                | Self::TopRight
                | Self::BottomLeft
                | Self::Bottom
                | Self::BottomRight
        )
    }

    /// Returns true if this is a corner handle (scales both axes).
    pub fn is_corner(&self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight
        )
    }
}

/// Anchor point for scaling operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AnchorMode {
    /// First keyframe in selection (earliest time).
    Start,
    /// Last keyframe in selection (latest time).
    End,
    /// Center of selection bounds.
    #[default]
    Center,
    /// Current playhead position.
    Playhead,
}

/// Configuration for bounding box appearance.
#[derive(Debug, Clone)]
pub struct BoundingBoxConfig {
    /// Color for the bounding box border.
    pub border_color: Color32,
    /// Color for the resize handles.
    pub handle_color: Color32,
    /// Color for the anchor indicator.
    pub anchor_color: Color32,
    /// Size of resize handles in pixels.
    pub handle_size: f32,
    /// Border stroke width.
    pub border_width: f32,
}

impl Default for BoundingBoxConfig {
    fn default() -> Self {
        Self {
            border_color: Color32::from_rgb(100, 150, 255),
            handle_color: Color32::from_rgb(255, 255, 255),
            anchor_color: Color32::from_rgb(255, 200, 100),
            handle_size: 6.0,
            border_width: 1.0,
        }
    }
}

/// Response from bounding box interaction.
#[derive(Debug, Clone, Default)]
pub struct BoundingBoxResponse {
    /// Handle that is currently hovered.
    pub hovered_handle: Option<BoundingBoxHandle>,
    /// Handle that started being dragged this frame.
    pub drag_started: Option<BoundingBoxHandle>,
    /// Handle currently being dragged.
    pub dragging_handle: Option<BoundingBoxHandle>,
    /// Whether drag ended this frame.
    pub drag_ended: bool,
}

/// Bounding box widget for selected keyframes.
///
/// This widget draws a bounding box around selected keyframes and provides
/// handles for offset and scale operations.
pub struct BoundingBox {
    /// Screen-space bounds of the selection.
    bounds: Rect,
    /// Screen-space anchor position.
    anchor_pos: Pos2,
    /// Configuration.
    config: BoundingBoxConfig,
}

impl BoundingBox {
    /// Create a new bounding box with the given screen-space bounds.
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            anchor_pos: bounds.center(),
            config: BoundingBoxConfig::default(),
        }
    }

    /// Set the anchor position (in screen coordinates).
    pub fn anchor(mut self, pos: Pos2) -> Self {
        self.anchor_pos = pos;
        self
    }

    /// Set the configuration.
    pub fn config(mut self, config: BoundingBoxConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the handle rectangles for hit testing.
    fn handle_rects(&self) -> [(BoundingBoxHandle, Rect); 8] {
        let hs = self.config.handle_size;
        let b = self.bounds;

        [
            (
                BoundingBoxHandle::TopLeft,
                Rect::from_center_size(b.left_top(), Vec2::splat(hs)),
            ),
            (
                BoundingBoxHandle::Top,
                Rect::from_center_size(Pos2::new(b.center().x, b.top()), Vec2::splat(hs)),
            ),
            (
                BoundingBoxHandle::TopRight,
                Rect::from_center_size(b.right_top(), Vec2::splat(hs)),
            ),
            (
                BoundingBoxHandle::Left,
                Rect::from_center_size(Pos2::new(b.left(), b.center().y), Vec2::splat(hs)),
            ),
            (
                BoundingBoxHandle::Right,
                Rect::from_center_size(Pos2::new(b.right(), b.center().y), Vec2::splat(hs)),
            ),
            (
                BoundingBoxHandle::BottomLeft,
                Rect::from_center_size(b.left_bottom(), Vec2::splat(hs)),
            ),
            (
                BoundingBoxHandle::Bottom,
                Rect::from_center_size(Pos2::new(b.center().x, b.bottom()), Vec2::splat(hs)),
            ),
            (
                BoundingBoxHandle::BottomRight,
                Rect::from_center_size(b.right_bottom(), Vec2::splat(hs)),
            ),
        ]
    }

    /// Hit test a screen position against the bounding box handles and interior.
    pub fn hit_test(&self, pos: Pos2) -> Option<BoundingBoxHandle> {
        // Check handles first (they have priority)
        for (handle, rect) in self.handle_rects() {
            if rect.contains(pos) {
                return Some(handle);
            }
        }

        // Check interior
        if self.bounds.contains(pos) {
            return Some(BoundingBoxHandle::Interior);
        }

        None
    }

    /// Paint the bounding box.
    pub fn paint(&self, painter: &Painter, hovered: Option<BoundingBoxHandle>) {
        // Draw dashed border
        self.draw_dashed_rect(painter, self.bounds);

        // Draw handles
        for (handle, rect) in self.handle_rects() {
            let is_hovered = hovered == Some(handle);
            self.draw_handle(painter, rect.center(), is_hovered);
        }

        // Draw anchor indicator
        self.draw_anchor(painter, self.anchor_pos);
    }

    /// Draw a dashed rectangle.
    fn draw_dashed_rect(&self, painter: &Painter, rect: Rect) {
        let stroke = Stroke::new(self.config.border_width, self.config.border_color);
        let dash_length = 4.0;
        let gap_length = 4.0;

        // Top edge
        self.draw_dashed_line(
            painter,
            rect.left_top(),
            rect.right_top(),
            stroke,
            dash_length,
            gap_length,
        );
        // Right edge
        self.draw_dashed_line(
            painter,
            rect.right_top(),
            rect.right_bottom(),
            stroke,
            dash_length,
            gap_length,
        );
        // Bottom edge
        self.draw_dashed_line(
            painter,
            rect.right_bottom(),
            rect.left_bottom(),
            stroke,
            dash_length,
            gap_length,
        );
        // Left edge
        self.draw_dashed_line(
            painter,
            rect.left_bottom(),
            rect.left_top(),
            stroke,
            dash_length,
            gap_length,
        );
    }

    /// Draw a dashed line between two points.
    fn draw_dashed_line(
        &self,
        painter: &Painter,
        start: Pos2,
        end: Pos2,
        stroke: Stroke,
        dash_length: f32,
        gap_length: f32,
    ) {
        let delta = end - start;
        let length = delta.length();
        if length < 0.001 {
            return;
        }

        let dir = delta / length;
        let mut pos = 0.0;
        let mut drawing = true;

        while pos < length {
            let segment_length = if drawing { dash_length } else { gap_length };
            let segment_end = (pos + segment_length).min(length);

            if drawing {
                let p0 = start + dir * pos;
                let p1 = start + dir * segment_end;
                painter.line_segment([p0, p1], stroke);
            }

            pos = segment_end;
            drawing = !drawing;
        }
    }

    /// Draw a resize handle.
    fn draw_handle(&self, painter: &Painter, center: Pos2, hovered: bool) {
        let size = if hovered {
            self.config.handle_size + 2.0
        } else {
            self.config.handle_size
        };

        // Fill
        painter.rect_filled(
            Rect::from_center_size(center, Vec2::splat(size)),
            0.0,
            self.config.handle_color,
        );

        // Border
        painter.rect_stroke(
            Rect::from_center_size(center, Vec2::splat(size)),
            0.0,
            Stroke::new(1.0, self.config.border_color),
            egui::StrokeKind::Outside,
        );
    }

    /// Draw the anchor indicator (diamond shape).
    fn draw_anchor(&self, painter: &Painter, center: Pos2) {
        let size = 5.0;

        // Diamond shape: top, right, bottom, left.
        let points = vec![
            Pos2::new(center.x, center.y - size),
            Pos2::new(center.x + size, center.y),
            Pos2::new(center.x, center.y + size),
            Pos2::new(center.x - size, center.y),
        ];

        painter.add(egui::Shape::convex_polygon(
            points,
            self.config.anchor_color,
            Stroke::new(1.0, Color32::WHITE),
        ));
    }
}

/// Calculate the bounding rectangle for a set of screen positions.
pub fn calculate_bounds(positions: &[Pos2]) -> Option<Rect> {
    if positions.is_empty() {
        return None;
    }

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for pos in positions {
        min_x = min_x.min(pos.x);
        max_x = max_x.max(pos.x);
        min_y = min_y.min(pos.y);
        max_y = max_y.max(pos.y);
    }

    // Add some padding to make single points visible
    let padding = 10.0;
    if (max_x - min_x) < 1.0 {
        min_x -= padding;
        max_x += padding;
    }
    if (max_y - min_y) < 1.0 {
        min_y -= padding;
        max_y += padding;
    }

    Some(Rect::from_min_max(
        Pos2::new(min_x, min_y),
        Pos2::new(max_x, max_y),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_properties() {
        assert!(BoundingBoxHandle::TopLeft.scales_x());
        assert!(BoundingBoxHandle::TopLeft.scales_y());
        assert!(BoundingBoxHandle::TopLeft.is_corner());

        assert!(BoundingBoxHandle::Top.scales_y());
        assert!(!BoundingBoxHandle::Top.scales_x());
        assert!(!BoundingBoxHandle::Top.is_corner());

        assert!(BoundingBoxHandle::Left.scales_x());
        assert!(!BoundingBoxHandle::Left.scales_y());
        assert!(!BoundingBoxHandle::Left.is_corner());

        assert!(!BoundingBoxHandle::Interior.scales_x());
        assert!(!BoundingBoxHandle::Interior.scales_y());
    }

    #[test]
    fn calculate_bounds_empty() {
        assert!(calculate_bounds(&[]).is_none());
    }

    #[test]
    fn calculate_bounds_single() {
        let bounds = calculate_bounds(&[Pos2::new(100.0, 50.0)]).unwrap();
        // Single point gets padding
        assert!(bounds.width() > 0.0);
        assert!(bounds.height() > 0.0);
        assert!(bounds.contains(Pos2::new(100.0, 50.0)));
    }

    #[test]
    fn calculate_bounds_multiple() {
        let bounds = calculate_bounds(&[
            Pos2::new(10.0, 20.0),
            Pos2::new(100.0, 80.0),
            Pos2::new(50.0, 50.0),
        ])
        .unwrap();

        assert_eq!(bounds.min.x, 10.0);
        assert_eq!(bounds.min.y, 20.0);
        assert_eq!(bounds.max.x, 100.0);
        assert_eq!(bounds.max.y, 80.0);
    }

    #[test]
    fn hit_test_handles() {
        let bounds = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));
        let bbox = BoundingBox::new(bounds);

        // Corner should be TopLeft
        assert_eq!(
            bbox.hit_test(Pos2::new(0.0, 0.0)),
            Some(BoundingBoxHandle::TopLeft)
        );

        // Center should be Interior
        assert_eq!(
            bbox.hit_test(Pos2::new(50.0, 50.0)),
            Some(BoundingBoxHandle::Interior)
        );

        // Outside should be None
        assert_eq!(bbox.hit_test(Pos2::new(200.0, 200.0)), None);
    }
}
