//! DopeSheet timeline widget.
//!
//! The DopeSheet provides a timeline view with:
//! - Left panel: Property tree (hierarchical list of animated properties)
//! - Right panel: Keyframe tracks (keyframe dots on timeline)

mod property_tree;
mod selection;
mod track_area;

use crate::core::keyframe::KeyframeId;
use crate::traits::{AnimationDataProvider, PropertyRow};
use crate::{SpaceTransform, TimeTick};
use egui::{Color32, Rect, Response, Sense, Ui, Vec2};
use crate::HashSet;

pub use property_tree::PropertyTree;
pub use selection::SelectionState;
pub use track_area::TrackArea;

/// Configuration for the DopeSheet.
#[derive(Debug, Clone)]
pub struct DopeSheetConfig {
    /// Width of the property tree panel.
    pub tree_width: f32,
    /// Height of each row.
    pub row_height: f32,
    /// Indentation per nesting level.
    pub indent_per_level: f32,
    /// Background color for the tree panel.
    pub tree_background: Color32,
    /// Background color for the track area.
    pub track_background: Color32,
    /// Color for alternating rows.
    pub alt_row_color: Color32,
    /// Color for row separator lines.
    pub separator_color: Color32,
    /// Color for the playhead.
    pub playhead_color: Color32,
    /// Whether to show aggregate keyframes for parent rows.
    pub show_aggregates: bool,
}

impl Default for DopeSheetConfig {
    fn default() -> Self {
        Self {
            tree_width: 200.0,
            row_height: 24.0,
            indent_per_level: 16.0,
            tree_background: Color32::from_gray(35),
            track_background: Color32::from_gray(25),
            alt_row_color: Color32::from_gray(30),
            separator_color: Color32::from_gray(45),
            playhead_color: Color32::from_rgb(255, 100, 100),
            show_aggregates: true,
        }
    }
}

/// Response from the DopeSheet.
#[derive(Default)]
pub struct DopeSheetResponse {
    /// The egui response for the whole widget.
    pub response: Option<Response>,
    /// Row that was clicked.
    pub clicked_row: Option<String>,
    /// Keyframe that was clicked.
    pub clicked_keyframe: Option<KeyframeId>,
    /// Keyframes that were box-selected.
    pub box_selected: Vec<KeyframeId>,
    /// Row expansion toggle request.
    pub toggle_collapse: Option<String>,
    /// Time position clicked (for scrubbing or adding keyframes).
    pub clicked_time: Option<TimeTick>,
    /// Row that was double-clicked.
    pub double_clicked_row: Option<String>,
    /// Animation commands to execute (from user interactions).
    pub commands: Vec<crate::traits::AnimationCommand>,
}

/// The main DopeSheet widget.
///
/// Layout:
/// ```text
/// +------------------+-----------------------------------+
/// | Property Tree    | Keyframe Track Area               |
/// |                  |                                   |
/// | - Object A       | [◆]-----[◆]-------[◆]             |
/// |   - Position     |   ◆-------◆---------◆             |
/// |     - X          |   ◆-------◆---------◆             |
/// |     - Y          |   ◆---◆-------------◆             |
/// |   - Rotation     |   ◆-----◆---------◆               |
/// | + Object B       | [◆]---------[◆]---[◆]             |
/// +------------------+-----------------------------------+
/// ```
pub struct DopeSheet<'a, P: AnimationDataProvider> {
    provider: &'a P,
    space: &'a SpaceTransform,
    selected_keyframes: &'a HashSet<KeyframeId>,
    selected_rows: &'a HashSet<String>,
    config: DopeSheetConfig,
}

impl<'a, P: AnimationDataProvider> DopeSheet<'a, P> {
    /// Create a new DopeSheet.
    pub fn new(
        provider: &'a P,
        space: &'a SpaceTransform,
        selected_keyframes: &'a HashSet<KeyframeId>,
        selected_rows: &'a HashSet<String>,
    ) -> Self {
        Self {
            provider,
            space,
            selected_keyframes,
            selected_rows,
            config: DopeSheetConfig::default(),
        }
    }

    /// Set the configuration.
    pub fn config(mut self, config: DopeSheetConfig) -> Self {
        self.config = config;
        self
    }

    /// Set tree width (same as property_panel_width).
    pub fn tree_width(mut self, width: f32) -> Self {
        self.config.tree_width = width;
        self
    }

    /// Set the property panel width (alias for tree_width).
    pub fn property_panel_width(mut self, width: f32) -> Self {
        self.config.tree_width = width;
        self
    }

    /// Set the row height.
    pub fn row_height(mut self, height: f32) -> Self {
        self.config.row_height = height;
        self
    }

    /// Show the DopeSheet widget.
    pub fn show(self, ui: &mut Ui) -> DopeSheetResponse {
        let mut result = DopeSheetResponse::default();

        let available = ui.available_size();
        let rows = self.provider.property_rows();

        // Filter visible rows (collapsed parents hide children)
        let visible_rows = self.filter_visible_rows(&rows);

        // Calculate total height
        let content_height = visible_rows.len() as f32 * self.config.row_height;
        let height = content_height.max(available.y).min(available.y);

        let (total_rect, response) =
            ui.allocate_exact_size(Vec2::new(available.x, height), Sense::hover());

        result.response = Some(response);

        if !ui.is_rect_visible(total_rect) {
            return result;
        }

        // Split into tree and track areas
        let tree_rect = Rect::from_min_size(
            total_rect.min,
            Vec2::new(self.config.tree_width, total_rect.height()),
        );
        let track_rect = Rect::from_min_size(
            tree_rect.right_top(),
            Vec2::new(total_rect.width() - self.config.tree_width, total_rect.height()),
        );

        // Render property tree
        let tree_response = PropertyTree::new(&visible_rows, self.selected_rows)
            .config(
                self.config.tree_background,
                self.config.alt_row_color,
                self.config.row_height,
                self.config.indent_per_level,
            )
            .show(ui, tree_rect);

        if let Some(row_id) = tree_response.clicked_row {
            result.clicked_row = Some(row_id);
        }
        if let Some(row_id) = tree_response.toggle_collapse {
            result.toggle_collapse = Some(row_id);
        }
        if let Some(row_id) = tree_response.double_clicked_row {
            result.double_clicked_row = Some(row_id);
        }

        // Render track area
        let track_response = TrackArea::new(
            self.provider,
            &visible_rows,
            self.space,
            self.selected_keyframes,
        )
        .config(
            self.config.track_background,
            self.config.alt_row_color,
            self.config.row_height,
            self.config.playhead_color,
            self.config.show_aggregates,
        )
        .show(ui, track_rect);

        if let Some(kf_id) = track_response.clicked_keyframe {
            result.clicked_keyframe = Some(kf_id);
        }
        if let Some(time) = track_response.clicked_time {
            result.clicked_time = Some(time);
        }
        result.box_selected = track_response.box_selected;

        // Draw separator between tree and tracks
        let painter = ui.painter_at(total_rect);
        painter.line_segment(
            [tree_rect.right_top(), tree_rect.right_bottom()],
            egui::Stroke::new(1.0, self.config.separator_color),
        );

        result
    }

    fn filter_visible_rows(&self, rows: &[PropertyRow]) -> Vec<PropertyRow> {
        let mut visible = Vec::new();
        let mut collapsed_depth: Option<usize> = None;

        for row in rows {
            // Skip if we're inside a collapsed parent
            if let Some(cd) = collapsed_depth {
                if row.depth > cd {
                    continue;
                } else {
                    collapsed_depth = None;
                }
            }

            visible.push(row.clone());

            // If this row is collapsed, skip its children
            if row.is_collapsed && row.can_collapse {
                collapsed_depth = Some(row.depth);
            }
        }

        visible
    }
}
