//! Property tree panel for the DopeSheet.

use crate::traits::PropertyRow;
use egui::{Color32, CursorIcon, Pos2, Rect, Sense, Ui, Vec2};
use crate::HashSet;

/// Response from the property tree.
#[derive(Default)]
pub struct PropertyTreeResponse {
    /// Row that was clicked.
    pub clicked_row: Option<String>,
    /// Row expansion toggle request.
    pub toggle_collapse: Option<String>,
    /// Row that was double-clicked.
    pub double_clicked_row: Option<String>,
}

/// Property tree panel widget.
pub struct PropertyTree<'a> {
    rows: &'a [PropertyRow],
    selected_rows: &'a HashSet<String>,
    background: Color32,
    alt_row_color: Color32,
    row_height: f32,
    indent_per_level: f32,
}

impl<'a> PropertyTree<'a> {
    /// Create a new property tree.
    pub fn new(rows: &'a [PropertyRow], selected_rows: &'a HashSet<String>) -> Self {
        Self {
            rows,
            selected_rows,
            background: Color32::from_gray(35),
            alt_row_color: Color32::from_gray(30),
            row_height: 24.0,
            indent_per_level: 16.0,
        }
    }

    /// Set configuration.
    pub fn config(
        mut self,
        background: Color32,
        alt_row_color: Color32,
        row_height: f32,
        indent_per_level: f32,
    ) -> Self {
        self.background = background;
        self.alt_row_color = alt_row_color;
        self.row_height = row_height;
        self.indent_per_level = indent_per_level;
        self
    }

    /// Show the property tree.
    pub fn show(self, ui: &mut Ui, rect: Rect) -> PropertyTreeResponse {
        let mut result = PropertyTreeResponse::default();

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 0.0, self.background);

        // Render rows
        for (i, row) in self.rows.iter().enumerate() {
            let row_rect = Rect::from_min_size(
                Pos2::new(rect.left(), rect.top() + i as f32 * self.row_height),
                Vec2::new(rect.width(), self.row_height),
            );

            if !ui.is_rect_visible(row_rect) {
                continue;
            }

            // Alternating row background
            if i % 2 == 1 {
                painter.rect_filled(row_rect, 0.0, self.alt_row_color);
            }

            // Selection highlight
            let is_selected = self.selected_rows.contains(&row.id);
            if is_selected {
                painter.rect_filled(row_rect, 0.0, ui.visuals().selection.bg_fill);
            }

            // Allocate interaction area
            let response = ui.allocate_rect(row_rect, Sense::click());

            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                if !is_selected {
                    painter.rect_filled(
                        row_rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 10),
                    );
                }
            }

            if response.clicked() {
                result.clicked_row = Some(row.id.clone());
            }

            if response.double_clicked() {
                result.double_clicked_row = Some(row.id.clone());
            }

            // Content
            let indent = row.depth as f32 * self.indent_per_level;
            let mut x = rect.left() + 4.0 + indent;
            let y_center = row_rect.center().y;

            // Collapse arrow
            if row.can_collapse {
                let arrow_rect = Rect::from_center_size(Pos2::new(x + 6.0, y_center), Vec2::splat(12.0));
                let arrow_response = ui.allocate_rect(arrow_rect, Sense::click());

                if arrow_response.clicked() {
                    result.toggle_collapse = Some(row.id.clone());
                }

                let arrow = if row.is_collapsed { "▶" } else { "▼" };
                let arrow_color = if arrow_response.hovered() {
                    Color32::WHITE
                } else {
                    Color32::from_gray(150)
                };

                painter.text(
                    arrow_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    arrow,
                    egui::FontId::proportional(9.0),
                    arrow_color,
                );

                x += 16.0;
            } else {
                x += 8.0; // Alignment space for leaves
            }

            // Label
            let label_color = if is_selected {
                ui.visuals().selection.stroke.color
            } else if row.track_id.is_some() {
                Color32::from_gray(200) // Leaf nodes
            } else {
                Color32::from_gray(180) // Parent nodes
            };

            painter.text(
                Pos2::new(x, y_center),
                egui::Align2::LEFT_CENTER,
                &row.label,
                egui::FontId::proportional(12.0),
                label_color,
            );

            // Color indicator for tracks
            if let Some(color) = row.color {
                let indicator_rect = Rect::from_min_size(
                    Pos2::new(row_rect.right() - 12.0, y_center - 3.0),
                    Vec2::new(6.0, 6.0),
                );
                painter.rect_filled(indicator_rect, 2.0, color);
            }
        }

        result
    }
}
