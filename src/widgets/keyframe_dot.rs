//! Keyframe dot/diamond marker.

use egui::{Color32, Painter, Pos2, Stroke};

/// Renders a keyframe marker (diamond shape).
pub struct KeyframeDot {
    /// Position in screen coordinates.
    pub pos: Pos2,
    /// Size of the diamond.
    pub size: f32,
    /// Fill color.
    pub color: Color32,
    /// Whether this keyframe is selected.
    pub selected: bool,
    /// Whether this keyframe is hovered.
    pub hovered: bool,
}

impl KeyframeDot {
    /// Create a new keyframe dot.
    pub fn new(pos: Pos2) -> Self {
        Self {
            pos,
            size: 5.0,
            color: Color32::from_rgb(100, 150, 255),
            selected: false,
            hovered: false,
        }
    }

    /// Set the size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the color.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    /// Set selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set hovered state.
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    /// Paint the keyframe dot.
    pub fn paint(&self, painter: &Painter) {
        let size = if self.hovered {
            self.size * 1.3
        } else {
            self.size
        };

        let color = if self.selected {
            Color32::from_rgb(255, 200, 100)
        } else {
            self.color
        };

        let stroke = if self.selected {
            Stroke::new(2.0, Color32::WHITE)
        } else {
            Stroke::new(1.0, Color32::from_gray(200))
        };

        // Diamond shape
        let points = vec![
            Pos2::new(self.pos.x, self.pos.y - size),
            Pos2::new(self.pos.x + size, self.pos.y),
            Pos2::new(self.pos.x, self.pos.y + size),
            Pos2::new(self.pos.x - size, self.pos.y),
        ];

        painter.add(egui::Shape::convex_polygon(points, color, stroke));
    }

    /// Check if a point is within the hit area.
    pub fn hit_test(&self, point: Pos2) -> bool {
        // Larger hit area for easier clicking.
        let hit_size = self.size * 2.0;
        let dx = (point.x - self.pos.x).abs();
        let dy = (point.y - self.pos.y).abs();
        // Diamond hit test: |x| + |y| <= size.
        dx + dy <= hit_size
    }
}

/// Renders an aggregate keyframe marker (multiple keyframes at same time).
pub struct AggregateKeyframeDot {
    /// Position in screen coordinates.
    pub pos: Pos2,
    /// Size of the marker.
    pub size: f32,
    /// Number of keyframes in this aggregate.
    pub count: usize,
    /// Whether all are selected.
    pub all_selected: bool,
    /// Whether some are selected.
    pub some_selected: bool,
    /// Whether hovered.
    pub hovered: bool,
}

impl AggregateKeyframeDot {
    /// Create a new aggregate dot.
    pub fn new(pos: Pos2, count: usize) -> Self {
        Self {
            pos,
            size: 6.0,
            count,
            all_selected: false,
            some_selected: false,
            hovered: false,
        }
    }

    /// Paint the aggregate dot.
    pub fn paint(&self, painter: &Painter) {
        let size = if self.hovered {
            self.size * 1.3
        } else {
            self.size
        };

        let color = if self.all_selected {
            Color32::from_rgb(255, 200, 100)
        } else if self.some_selected {
            Color32::from_rgb(200, 175, 130)
        } else {
            Color32::from_rgb(100, 150, 255)
        };

        let stroke = if self.all_selected || self.some_selected {
            Stroke::new(2.0, Color32::WHITE)
        } else {
            Stroke::new(1.5, Color32::from_gray(200))
        };

        // Larger diamond for aggregates
        let points = vec![
            Pos2::new(self.pos.x, self.pos.y - size),
            Pos2::new(self.pos.x + size, self.pos.y),
            Pos2::new(self.pos.x, self.pos.y + size),
            Pos2::new(self.pos.x - size, self.pos.y),
        ];

        painter.add(egui::Shape::convex_polygon(points, color, stroke));

        // Show count if > 2
        if self.count > 2 {
            painter.text(
                Pos2::new(self.pos.x, self.pos.y - size - 8.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{}", self.count),
                egui::FontId::proportional(9.0),
                Color32::from_gray(180),
            );
        }
    }
}
