//! Track area panel for the DopeSheet.

use crate::core::keyframe::KeyframeId;
use crate::traits::{AnimationDataProvider, PropertyRow};
use crate::widgets::keyframe_dot::{AggregateKeyframeDot, KeyframeDot};
use crate::widgets::time_ruler::draw_time_grid;
use crate::{SpaceTransform, TimeTick};
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use crate::{HashMap, HashSet};

/// Response from the track area.
#[derive(Default)]
pub struct TrackAreaResponse {
    /// Keyframe that was clicked.
    pub clicked_keyframe: Option<KeyframeId>,
    /// Time position clicked.
    pub clicked_time: Option<TimeTick>,
    /// Keyframes selected via box selection.
    pub box_selected: Vec<KeyframeId>,
}

/// Track area panel widget.
pub struct TrackArea<'a, P: AnimationDataProvider> {
    provider: &'a P,
    rows: &'a [PropertyRow],
    space: &'a SpaceTransform,
    selected_keyframes: &'a HashSet<KeyframeId>,
    background: Color32,
    alt_row_color: Color32,
    row_height: f32,
    playhead_color: Color32,
    show_aggregates: bool,
}

impl<'a, P: AnimationDataProvider> TrackArea<'a, P> {
    /// Create a new track area.
    pub fn new(
        provider: &'a P,
        rows: &'a [PropertyRow],
        space: &'a SpaceTransform,
        selected_keyframes: &'a HashSet<KeyframeId>,
    ) -> Self {
        Self {
            provider,
            rows,
            space,
            selected_keyframes,
            background: Color32::from_gray(25),
            alt_row_color: Color32::from_gray(30),
            row_height: 24.0,
            playhead_color: Color32::from_rgb(255, 100, 100),
            show_aggregates: true,
        }
    }

    /// Set configuration.
    pub fn config(
        mut self,
        background: Color32,
        alt_row_color: Color32,
        row_height: f32,
        playhead_color: Color32,
        show_aggregates: bool,
    ) -> Self {
        self.background = background;
        self.alt_row_color = alt_row_color;
        self.row_height = row_height;
        self.playhead_color = playhead_color;
        self.show_aggregates = show_aggregates;
        self
    }

    /// Show the track area.
    pub fn show(self, ui: &mut Ui, rect: Rect) -> TrackAreaResponse {
        let mut result = TrackAreaResponse::default();

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 0.0, self.background);

        // Time grid
        draw_time_grid(&painter, rect, self.space, Color32::from_gray(40), None);

        // Render rows
        let mut keyframe_positions: Vec<(KeyframeId, Pos2, usize)> = Vec::new(); // (id, pos, row_index)

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

            let y_center = row_rect.center().y;

            // Draw keyframes for this row
            if let Some(track_id) = row.track_id {
                if let Some(positions) = self.provider.keyframe_positions(track_id) {
                    for (kf_id, position) in positions {
                        let x = self.space.unit_to_clipped(position);
                        if x >= rect.left() && x <= rect.right() {
                            let pos = Pos2::new(x, y_center);
                            let is_selected = self.selected_keyframes.contains(&kf_id);

                            KeyframeDot::new(pos)
                                .color(row.color.unwrap_or(Color32::from_rgb(100, 180, 255)))
                                .selected(is_selected)
                                .size(4.0)
                                .paint(&painter);

                            keyframe_positions.push((kf_id, pos, i));
                        }
                    }
                }
            } else if self.show_aggregates && row.can_collapse {
                // Aggregate keyframes for parent rows
                let aggregates = self.collect_aggregates(row, i);
                for (quantized_ms, kf_ids) in aggregates {
                    let position = TimeTick::new(quantized_ms as f64 / 1000.0);
                    let x = self.space.unit_to_clipped(position);
                    if x >= rect.left() && x <= rect.right() {
                        let pos = Pos2::new(x, y_center);
                        let all_selected = kf_ids.iter().all(|id| self.selected_keyframes.contains(id));
                        let some_selected = kf_ids.iter().any(|id| self.selected_keyframes.contains(id));

                        let mut dot = AggregateKeyframeDot::new(pos, kf_ids.len());
                        dot.all_selected = all_selected;
                        dot.some_selected = some_selected && !all_selected;
                        dot.paint(&painter);

                        // Store for hit testing
                        for kf_id in kf_ids {
                            keyframe_positions.push((kf_id, pos, i));
                        }
                    }
                }
            }
        }

        // Draw playhead
        let current_time = self.provider.current_time();
        let playhead_x = self.space.unit_to_clipped(current_time);
        if playhead_x >= rect.left() && playhead_x <= rect.right() {
            painter.line_segment(
                [
                    Pos2::new(playhead_x, rect.top()),
                    Pos2::new(playhead_x, rect.bottom()),
                ],
                Stroke::new(2.0, self.playhead_color),
            );

            // Playhead head (small triangle at top)
            let head_size = 6.0;
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(playhead_x, rect.top()),
                    Pos2::new(playhead_x - head_size, rect.top() - head_size),
                    Pos2::new(playhead_x + head_size, rect.top() - head_size),
                ],
                self.playhead_color,
                Stroke::NONE,
            ));
        }

        // Handle interactions
        let response = ui.allocate_rect(rect, Sense::click_and_drag());

        if let Some(pos) = response.interact_pointer_pos() {
            // Check for keyframe clicks
            if response.clicked() {
                for (kf_id, kf_pos, _) in &keyframe_positions {
                    let dx = (pos.x - kf_pos.x).abs();
                    let dy = (pos.y - kf_pos.y).abs();
                    if dx + dy < 10.0 {
                        result.clicked_keyframe = Some(*kf_id);
                        break;
                    }
                }

                // If no keyframe clicked, report time click
                if result.clicked_keyframe.is_none() {
                    result.clicked_time = Some(self.space.clipped_to_unit(pos.x));
                }
            }
        }

        result
    }

    /// Collect aggregate keyframes for a parent row.
    /// Returns a map from quantized time (milliseconds as i64) to keyframe IDs.
    fn collect_aggregates(&self, parent_row: &PropertyRow, parent_index: usize) -> HashMap<i64, Vec<KeyframeId>> {
        let mut aggregates: HashMap<i64, Vec<KeyframeId>> = HashMap::new();

        // Find all child rows
        let parent_depth = parent_row.depth;
        for row in self.rows.iter().skip(parent_index + 1) {
            if row.depth <= parent_depth {
                break; // No longer a child
            }

            if let Some(track_id) = row.track_id {
                if let Some(positions) = self.provider.keyframe_positions(track_id) {
                    for (kf_id, position) in positions {
                        // Quantize to avoid floating point issues (millisecond precision)
                        let quantized = (position.value() * 1000.0).round() as i64;
                        aggregates.entry(quantized).or_default().push(kf_id);
                    }
                }
            }
        }

        aggregates
    }
}
