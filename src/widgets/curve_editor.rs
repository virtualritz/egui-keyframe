//! Bezier curve editor widget for animation curves.

use crate::core::keyframe::{KeyframeId, KeyframeType};
use crate::traits::{KeyframeSource, KeyframeView};
use crate::widgets::bounding_box::{calculate_bounds, AnchorMode, BoundingBox, BoundingBoxHandle};
use crate::widgets::keyframe_dot::KeyframeDot;
use crate::{SpaceTransform, TimeTick};
use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};
use crate::HashSet;

/// Configuration for the curve editor.
#[derive(Debug, Clone)]
pub struct CurveEditorConfig {
    /// Height of the curve editor area.
    pub height: f32,
    /// Color for the curve line.
    pub curve_color: Color32,
    /// Color for keyframe dots.
    pub keyframe_color: Color32,
    /// Color for selected keyframes.
    pub selected_color: Color32,
    /// Color for bezier handles.
    pub handle_color: Color32,
    /// Color for handle lines.
    pub handle_line_color: Color32,
    /// Grid line color.
    pub grid_color: Color32,
    /// Background color.
    pub background: Color32,
    /// Padding at top/bottom.
    pub vertical_padding: f32,
    /// Curve line width.
    pub curve_width: f32,
    /// Number of segments for bezier curve approximation.
    pub curve_segments: usize,
    /// Color for the selection bounding box border.
    pub bounding_box_color: Color32,
    /// Color for the bounding box anchor indicator.
    pub anchor_color: Color32,
    /// Size of bounding box handles.
    pub bbox_handle_size: f32,
}

impl Default for CurveEditorConfig {
    fn default() -> Self {
        Self {
            height: 200.0,
            curve_color: Color32::from_rgb(100, 180, 255),
            keyframe_color: Color32::from_rgb(100, 180, 255),
            selected_color: Color32::from_rgb(255, 200, 100),
            handle_color: Color32::from_rgb(255, 150, 100),
            handle_line_color: Color32::from_gray(120),
            grid_color: Color32::from_gray(50),
            background: Color32::from_gray(25),
            vertical_padding: 20.0,
            curve_width: 2.0,
            curve_segments: 32,
            bounding_box_color: Color32::from_rgb(100, 150, 255),
            anchor_color: Color32::from_rgb(255, 200, 100),
            bbox_handle_size: 6.0,
        }
    }
}

/// Which handle is being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleSide {
    Left,
    Right,
}

/// Information about a handle drag.
#[derive(Debug, Clone)]
pub struct HandleDrag {
    pub keyframe_id: KeyframeId,
    pub side: HandleSide,
    pub new_x: f32,
    pub new_y: f32,
}

/// Information about a keyframe move.
#[derive(Debug, Clone)]
pub struct KeyframeMove {
    pub keyframe_id: KeyframeId,
    pub new_position: TimeTick,
    pub new_value: f32,
}

/// Response from the curve editor.
#[derive(Default)]
pub struct CurveEditorResponse {
    /// The egui response.
    pub response: Option<Response>,
    /// Keyframe that was clicked.
    pub clicked_keyframe: Option<KeyframeId>,
    /// Handle that was dragged.
    pub handle_drag: Option<HandleDrag>,
    /// Keyframe that was moved.
    pub keyframe_move: Option<KeyframeMove>,
    /// Request to add keyframe at position (time, value).
    pub add_keyframe_at: Option<(TimeTick, f32)>,
    /// Keyframe to delete.
    pub delete_keyframe: Option<KeyframeId>,
    /// Batch offset for selected keyframes (delta_time, delta_value).
    pub offset_keyframes: Option<(TimeTick, f32)>,
    /// Batch scale for selected keyframes (anchor_time, anchor_value, time_scale, value_scale).
    pub scale_keyframes: Option<(TimeTick, f32, f64, f64)>,
    /// Whether a bounding box transform drag ended (for undo grouping).
    pub transform_ended: bool,
    /// Request to select all keyframes (Cmd+A).
    pub select_all: bool,
    /// Request to deselect all keyframes (Escape).
    pub deselect_all: bool,
    /// Pan delta in screen pixels (for smooth_scroll_delta).
    pub pan_delta: Option<Vec2>,
    /// Horizontal zoom (time axis): (zoom_factor, center_time).
    /// zoom_factor > 1.0 = zoom in, < 1.0 = zoom out.
    pub zoom_horizontal: Option<(f32, TimeTick)>,
    /// Vertical zoom (value axis): zoom_factor.
    /// zoom_factor > 1.0 = zoom in, < 1.0 = zoom out.
    pub zoom_vertical: Option<f32>,
    /// Request to change interpolation type for a keyframe.
    pub set_interpolation: Option<(KeyframeId, KeyframeType)>,
    /// Request to fit view to all keyframes (press F).
    pub fit_view: bool,
}

/// Curve editor widget for editing bezier animation curves.
///
/// Generic over `S: KeyframeSource` to support both `Track<f32>` and
/// custom keyframe sources like `ParameterValue`.
pub struct CurveEditor<'a, S: KeyframeSource> {
    source: &'a S,
    selected: &'a HashSet<KeyframeId>,
    space: &'a SpaceTransform,
    value_range: (f32, f32),
    config: CurveEditorConfig,
    id_source: Option<egui::Id>,
    anchor_mode: AnchorMode,
    current_time: TimeTick,
}

impl<'a, S: KeyframeSource> CurveEditor<'a, S> {
    /// Create a new curve editor.
    pub fn new(
        source: &'a S,
        selected: &'a HashSet<KeyframeId>,
        space: &'a SpaceTransform,
        value_range: (f32, f32),
    ) -> Self {
        Self {
            source,
            selected,
            space,
            value_range,
            config: CurveEditorConfig::default(),
            id_source: None,
            anchor_mode: AnchorMode::default(),
            current_time: TimeTick::default(),
        }
    }

    /// Set the configuration.
    pub fn config(mut self, config: CurveEditorConfig) -> Self {
        self.config = config;
        self
    }

    /// Set a custom ID source.
    pub fn id_source(mut self, id: impl std::hash::Hash) -> Self {
        self.id_source = Some(egui::Id::new(id));
        self
    }

    /// Set the anchor mode for scale operations.
    pub fn anchor_mode(mut self, mode: AnchorMode) -> Self {
        self.anchor_mode = mode;
        self
    }

    /// Set the current time (for playhead anchor mode).
    pub fn current_time(mut self, time: impl Into<TimeTick>) -> Self {
        self.current_time = time.into();
        self
    }

    /// Show the curve editor widget.
    pub fn show(self, ui: &mut Ui) -> CurveEditorResponse {
        let id = self
            .id_source
            .unwrap_or_else(|| ui.make_persistent_id("curve_editor"));

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), self.config.height),
            Sense::click_and_drag(),
        );

        let mut result = CurveEditorResponse {
            response: Some(response.clone()),
            ..Default::default()
        };

        if !ui.is_rect_visible(rect) {
            return result;
        }

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 0.0, self.config.background);

        // Draw grid
        self.draw_grid(&painter, rect);

        // Draw curves between keyframes
        let keyframes = self.source.keyframes_sorted();
        let keyframe_refs: Vec<&KeyframeView> = keyframes.iter().collect();
        for window in keyframes.windows(2) {
            let left = &window[0];
            let right = &window[1];
            if left.connected_right {
                self.draw_curve_segment(&painter, rect, left, right);
            }
        }

        // Collect selected keyframe positions for bounding box
        let mut selected_positions: Vec<Pos2> = Vec::new();
        let mut selected_keyframe_data: Vec<(KeyframeId, TimeTick, f32)> = Vec::new();

        // Draw keyframes and handles
        let pointer_pos = response.hover_pos();
        let mut hovered_keyframe = None;

        for kf in &keyframes {
            let is_selected = self.selected.contains(&kf.id);
            let screen_pos = self.keyframe_to_screen(rect, kf);

            if is_selected {
                selected_positions.push(screen_pos);
                selected_keyframe_data.push((kf.id, kf.position, kf.value));
            }

            // Check if hovered
            let is_hovered = pointer_pos
                .map(|p| {
                    let dx = (p.x - screen_pos.x).abs();
                    let dy = (p.y - screen_pos.y).abs();
                    dx + dy < 12.0
                })
                .unwrap_or(false);

            if is_hovered {
                hovered_keyframe = Some(kf.id);
            }

            // Draw handles for selected keyframes
            if is_selected {
                self.draw_handles(&painter, rect, kf, &keyframe_refs);
            }

            // Draw keyframe dot
            KeyframeDot::new(screen_pos)
                .color(self.config.keyframe_color)
                .selected(is_selected)
                .hovered(is_hovered)
                .paint(&painter);
        }

        // Draw bounding box if multiple keyframes selected
        let mut hovered_bbox_handle = None;
        if selected_positions.len() > 1 {
            if let Some(bounds) = calculate_bounds(&selected_positions) {
                let anchor_pos = self.calculate_anchor_screen_pos(rect, &selected_keyframe_data);

                let bbox_config = crate::widgets::bounding_box::BoundingBoxConfig {
                    border_color: self.config.bounding_box_color,
                    handle_color: Color32::WHITE,
                    anchor_color: self.config.anchor_color,
                    handle_size: self.config.bbox_handle_size,
                    border_width: 1.0,
                };

                let bbox = BoundingBox::new(bounds).anchor(anchor_pos).config(bbox_config);

                // Hit test for hover state
                if let Some(pos) = pointer_pos {
                    hovered_bbox_handle = bbox.hit_test(pos);
                }

                bbox.paint(&painter, hovered_bbox_handle);
            }
        }

        // Handle interactions
        self.handle_interactions(
            ui,
            id,
            rect,
            &response,
            &keyframe_refs,
            hovered_keyframe,
            hovered_bbox_handle,
            &selected_keyframe_data,
            &mut result,
        );

        result
    }

    /// Calculate the anchor position in screen coordinates.
    fn calculate_anchor_screen_pos(
        &self,
        rect: Rect,
        selected_data: &[(KeyframeId, TimeTick, f32)],
    ) -> Pos2 {
        if selected_data.is_empty() {
            return rect.center();
        }

        match self.anchor_mode {
            AnchorMode::Start => {
                // First keyframe (earliest time)
                let first = selected_data
                    .iter()
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap();
                Pos2::new(
                    self.space.unit_to_clipped(first.1),
                    self.value_to_y(rect, first.2),
                )
            }
            AnchorMode::End => {
                // Last keyframe (latest time)
                let last = selected_data
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap();
                Pos2::new(
                    self.space.unit_to_clipped(last.1),
                    self.value_to_y(rect, last.2),
                )
            }
            AnchorMode::Center => {
                // Center of bounds
                let min_t = selected_data
                    .iter()
                    .map(|d| d.1)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let max_t = selected_data
                    .iter()
                    .map(|d| d.1)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let min_v = selected_data
                    .iter()
                    .map(|d| d.2)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let max_v = selected_data
                    .iter()
                    .map(|d| d.2)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();

                let center_t = min_t.lerp(max_t, 0.5);
                let center_v = (min_v + max_v) / 2.0;

                Pos2::new(
                    self.space.unit_to_clipped(center_t),
                    self.value_to_y(rect, center_v),
                )
            }
            AnchorMode::Playhead => {
                // Playhead position with interpolated value
                let playhead_x = self.space.unit_to_clipped(self.current_time);

                // Find approximate value at playhead by interpolation
                let min_v = selected_data
                    .iter()
                    .map(|d| d.2)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let max_v = selected_data
                    .iter()
                    .map(|d| d.2)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap();
                let center_v = (min_v + max_v) / 2.0;

                Pos2::new(playhead_x, self.value_to_y(rect, center_v))
            }
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        // Horizontal grid lines for values
        let (min_val, max_val) = self.value_range;
        let value_range = max_val - min_val;

        // Determine nice value intervals
        let target_lines = 5;
        let ideal_interval = value_range / target_lines as f32;
        let nice_intervals = [0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0];

        let mut interval = 1.0;
        for &ni in &nice_intervals {
            if ni >= ideal_interval {
                interval = ni;
                break;
            }
        }

        let first_line = (min_val / interval).ceil() * interval;
        let mut v = first_line;
        while v <= max_val {
            let y = self.value_to_y(rect, v);
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, self.config.grid_color),
            );

            // Value label
            painter.text(
                Pos2::new(rect.left() + 4.0, y - 2.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{:.1}", v),
                egui::FontId::proportional(9.0),
                Color32::from_gray(100),
            );

            v += interval;
        }

        // Vertical grid lines for time
        crate::widgets::time_ruler::draw_time_grid(
            painter,
            rect,
            self.space,
            self.config.grid_color,
            None,
        );
    }

    fn draw_curve_segment(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        left: &KeyframeView,
        right: &KeyframeView,
    ) {
        let left_pos = self.keyframe_to_screen(rect, left);
        let right_pos = self.keyframe_to_screen(rect, right);

        match left.keyframe_type {
            KeyframeType::Hold => {
                // Step function: horizontal then vertical
                let mid = Pos2::new(right_pos.x, left_pos.y);
                painter.line_segment(
                    [left_pos, mid],
                    Stroke::new(self.config.curve_width, self.config.curve_color),
                );
                painter.line_segment(
                    [mid, right_pos],
                    Stroke::new(
                        self.config.curve_width,
                        self.config.curve_color.linear_multiply(0.5),
                    ),
                );
            }
            KeyframeType::Linear => {
                // Straight line
                painter.line_segment(
                    [left_pos, right_pos],
                    Stroke::new(self.config.curve_width, self.config.curve_color),
                );
            }
            KeyframeType::Bezier => {
                // Bezier curve - use egui's built-in cubic bezier
                let dx = right_pos.x - left_pos.x;
                let dy = right_pos.y - left_pos.y;

                let cp1 = Pos2::new(
                    left_pos.x + dx * left.handles.right_x,
                    left_pos.y + dy * left.handles.right_y,
                );
                let cp2 = Pos2::new(
                    left_pos.x + dx * right.handles.left_x,
                    left_pos.y + dy * right.handles.left_y,
                );

                painter.add(Shape::CubicBezier(egui::epaint::CubicBezierShape {
                    points: [left_pos, cp1, cp2, right_pos],
                    closed: false,
                    fill: Color32::TRANSPARENT,
                    stroke: Stroke::new(self.config.curve_width, self.config.curve_color).into(),
                }));
            }
        }
    }

    fn draw_handles(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        kf: &KeyframeView,
        all_keyframes: &[&KeyframeView],
    ) {
        let kf_pos = self.keyframe_to_screen(rect, kf);

        // Find adjacent keyframes
        let mut prev_kf: Option<&KeyframeView> = None;
        let mut next_kf: Option<&KeyframeView> = None;

        for (i, other) in all_keyframes.iter().enumerate() {
            if other.id == kf.id {
                if i > 0 {
                    prev_kf = Some(all_keyframes[i - 1]);
                }
                if i + 1 < all_keyframes.len() {
                    next_kf = Some(all_keyframes[i + 1]);
                }
                break;
            }
        }

        // Draw left handle (if there's a previous keyframe)
        if let Some(prev) = prev_kf {
            if prev.connected_right {
                let prev_pos = self.keyframe_to_screen(rect, prev);
                let dx = kf_pos.x - prev_pos.x;
                let dy = kf_pos.y - prev_pos.y;

                let handle_pos = Pos2::new(
                    prev_pos.x + dx * kf.handles.left_x,
                    prev_pos.y + dy * kf.handles.left_y,
                );

                // Handle line
                painter.line_segment(
                    [kf_pos, handle_pos],
                    Stroke::new(1.0, self.config.handle_line_color),
                );

                // Handle circle
                painter.circle_filled(handle_pos, 4.0, self.config.handle_color);
                painter.circle_stroke(handle_pos, 4.0, Stroke::new(1.0, Color32::WHITE));
            }
        }

        // Draw right handle (if connected to next keyframe)
        if let Some(next) = next_kf {
            if kf.connected_right {
                let next_pos = self.keyframe_to_screen(rect, next);
                let dx = next_pos.x - kf_pos.x;
                let dy = next_pos.y - kf_pos.y;

                let handle_pos = Pos2::new(
                    kf_pos.x + dx * kf.handles.right_x,
                    kf_pos.y + dy * kf.handles.right_y,
                );

                // Handle line
                painter.line_segment(
                    [kf_pos, handle_pos],
                    Stroke::new(1.0, self.config.handle_line_color),
                );

                // Handle circle
                painter.circle_filled(handle_pos, 4.0, self.config.handle_color);
                painter.circle_stroke(handle_pos, 4.0, Stroke::new(1.0, Color32::WHITE));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_interactions(
        &self,
        ui: &mut Ui,
        id: egui::Id,
        rect: Rect,
        response: &Response,
        keyframes: &[&KeyframeView],
        hovered_keyframe: Option<KeyframeId>,
        hovered_bbox_handle: Option<BoundingBoxHandle>,
        selected_keyframe_data: &[(KeyframeId, TimeTick, f32)],
        result: &mut CurveEditorResponse,
    ) {
        // Keyboard shortcuts
        if response.has_focus() || response.hovered() {
            // Cmd+A to select all
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::A)) {
                result.select_all = true;
            }

            // Escape to deselect all
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                result.deselect_all = true;
            }

            // Delete key
            if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                if let Some(kf_id) = self.selected.iter().next().copied() {
                    result.delete_keyframe = Some(kf_id);
                }
            }

            // F key to fit view to all keyframes
            if ui.input(|i| i.key_pressed(egui::Key::F)) {
                result.fit_view = true;
            }
        }

        // Handle zoom and pan (matching timeline behavior)
        if response.hovered() {
            // Ctrl+scroll or pinch gesture for zoom
            let zoom_delta = ui.input(|i| i.zoom_delta_2d());
            if zoom_delta.x != 1.0 {
                if let Some(pos) = response.hover_pos() {
                    let center_time = self.space.clipped_to_unit(pos.x);
                    result.zoom_horizontal = Some((zoom_delta.x, center_time));
                }
            }
            if zoom_delta.y != 1.0 {
                result.zoom_vertical = Some(zoom_delta.y);
            }

            // Smooth scroll for panning
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
            if scroll_delta != Vec2::ZERO {
                result.pan_delta = Some(scroll_delta);
            }
        }

        // Right-click drag for directional zoom (detect dominant axis)
        if response.dragged_by(egui::PointerButton::Secondary) {
            let drag_delta = response.drag_delta();
            if drag_delta != Vec2::ZERO {
                // Determine zoom based on which direction dominates
                let zoom_speed = 0.01;
                if drag_delta.x.abs() > drag_delta.y.abs() {
                    // Horizontal drag -> horizontal zoom (time axis)
                    if let Some(pos) = response.hover_pos() {
                        let center_time = self.space.clipped_to_unit(pos.x);
                        let zoom_factor = (drag_delta.x * zoom_speed).exp();
                        result.zoom_horizontal = Some((zoom_factor, center_time));
                    }
                } else {
                    // Vertical drag -> vertical zoom (value axis)
                    let zoom_factor = (-drag_delta.y * zoom_speed).exp();
                    result.zoom_vertical = Some(zoom_factor);
                }
                return; // Don't process other drag interactions
            }
        }

        // Middle-mouse drag or Alt+LMB drag for panning
        let is_middle_drag = ui.input(|i| i.pointer.middle_down());
        let is_alt_drag = ui.input(|i| i.modifiers.alt) && response.dragged();

        if (is_middle_drag || is_alt_drag) && response.hovered() {
            let drag_delta = ui.input(|i| i.pointer.delta());
            if drag_delta != Vec2::ZERO {
                result.pan_delta = Some(drag_delta);
                return; // Don't process other drag interactions
            }
        }

        // Right-click on keyframe for context menu (only if not dragging)
        if response.secondary_clicked() {
            if let Some(kf_id) = hovered_keyframe {
                // Store the keyframe ID for context menu
                ui.memory_mut(|mem| mem.data.insert_temp(id.with("context_kf"), kf_id));
            }
        }

        // Show context menu
        let context_kf: Option<KeyframeId> = ui.memory(|mem| mem.data.get_temp(id.with("context_kf")));
        if let Some(kf_id) = context_kf {
            // Find the keyframe to get its current type
            let current_type = keyframes
                .iter()
                .find(|kf| kf.id == kf_id)
                .map(|kf| kf.keyframe_type);

            let mut close_menu = false;
            egui::Area::new(id.with("interp_menu"))
                .order(egui::Order::Foreground)
                .fixed_pos(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(120.0);
                        ui.label("Interpolation");
                        ui.separator();

                        let types = [
                            (KeyframeType::Hold, "Hold (Step)"),
                            (KeyframeType::Linear, "Linear"),
                            (KeyframeType::Bezier, "Bezier"),
                        ];

                        for (kf_type, label) in types {
                            let is_current = current_type == Some(kf_type);
                            let text = if is_current {
                                format!("✓ {}", label)
                            } else {
                                format!("   {}", label)
                            };

                            if ui.selectable_label(is_current, text).clicked() {
                                if !is_current {
                                    result.set_interpolation = Some((kf_id, kf_type));
                                }
                                close_menu = true;
                            }
                        }

                        // Close on click outside or Escape
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) ||
                           (ui.input(|i| i.pointer.any_click()) && !ui.ui_contains_pointer()) {
                            close_menu = true;
                        }
                    });
                });

            if close_menu {
                ui.memory_mut(|mem| mem.data.remove::<KeyframeId>(id.with("context_kf")));
            }
        }

        // Double-click to add keyframe
        if response.double_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let time = self.space.clipped_to_unit(pos.x);
                let value = self.y_to_value(rect, pos.y);
                result.add_keyframe_at = Some((time, value));
                return;
            }
        }

        // Single click on keyframe to select
        if response.clicked() {
            if let Some(kf_id) = hovered_keyframe {
                result.clicked_keyframe = Some(kf_id);
            }
        }

        // Drag interactions
        if response.dragged() {
            let drag_delta = response.drag_delta();

            // Bounding box drag handling (for multiple selected keyframes)
            if selected_keyframe_data.len() > 1 {
                if let Some(handle) = hovered_bbox_handle {
                    match handle {
                        BoundingBoxHandle::Interior => {
                            // Offset all selected keyframes
                            let delta_time = self.screen_delta_to_time(drag_delta.x);
                            let delta_value = self.screen_delta_to_value(rect, drag_delta.y);

                            // Constrain to axis if shift is held
                            let (final_time, final_value) =
                                if ui.input(|i| i.modifiers.shift) {
                                    if drag_delta.x.abs() > drag_delta.y.abs() {
                                        (delta_time, 0.0)
                                    } else {
                                        (TimeTick::default(), delta_value)
                                    }
                                } else {
                                    (delta_time, delta_value)
                                };

                            result.offset_keyframes = Some((final_time, final_value));
                        }
                        _ => {
                            // Scale operation for edge/corner handles
                            if let Some(scale) = self.calculate_scale_from_drag(
                                rect,
                                handle,
                                drag_delta,
                                selected_keyframe_data,
                            ) {
                                result.scale_keyframes = Some(scale);
                            }
                        }
                    }
                    return;
                }
            }

            // Single keyframe drag
            if let Some(kf_id) = hovered_keyframe {
                if self.selected.contains(&kf_id) {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let time = self.space.clipped_to_unit(pos.x);
                        let value = self.y_to_value(rect, pos.y);
                        result.keyframe_move = Some(KeyframeMove {
                            keyframe_id: kf_id,
                            new_position: time,
                            new_value: value,
                        });
                    }
                }
            }
        }

        // Drag ended - signal for undo grouping
        if response.drag_stopped() && selected_keyframe_data.len() > 1 && hovered_bbox_handle.is_some() {
            result.transform_ended = true;
        }
    }

    /// Convert screen X delta to time delta.
    fn screen_delta_to_time(&self, delta_x: f32) -> TimeTick {
        TimeTick::new(delta_x as f64 / self.space.pixels_per_unit)
    }

    /// Convert screen Y delta to value delta.
    fn screen_delta_to_value(&self, rect: Rect, delta_y: f32) -> f32 {
        let (min_val, max_val) = self.value_range;
        let value_range = max_val - min_val;
        let usable_height = rect.height() - 2.0 * self.config.vertical_padding;
        // Y is inverted (screen Y goes down, value goes up)
        -delta_y * value_range / usable_height
    }

    /// Calculate scale factors from a bounding box handle drag.
    fn calculate_scale_from_drag(
        &self,
        rect: Rect,
        handle: BoundingBoxHandle,
        drag_delta: Vec2,
        selected_data: &[(KeyframeId, TimeTick, f32)],
    ) -> Option<(TimeTick, f32, f64, f64)> {
        if selected_data.is_empty() {
            return None;
        }

        // Calculate bounds
        let min_t = selected_data
            .iter()
            .map(|d| d.1)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let max_t = selected_data
            .iter()
            .map(|d| d.1)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;
        let min_v = selected_data
            .iter()
            .map(|d| d.2)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;
        let max_v = selected_data
            .iter()
            .map(|d| d.2)
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;

        let time_range = (max_t - min_t).value();
        let value_range = max_v - min_v;

        // Get anchor position
        let (anchor_time, anchor_value) = match self.anchor_mode {
            AnchorMode::Start => (min_t, min_v),
            AnchorMode::End => (max_t, max_v),
            AnchorMode::Center => (min_t.lerp(max_t, 0.5), (min_v + max_v) / 2.0),
            AnchorMode::Playhead => {
                let center_v = (min_v + max_v) / 2.0;
                (self.current_time, center_v)
            }
        };

        // Convert drag delta to time/value space
        let delta_time = self.screen_delta_to_time(drag_delta.x).value();
        let delta_value = self.screen_delta_to_value(rect, drag_delta.y);

        // Calculate scale based on handle
        let mut time_scale = 1.0;
        let mut value_scale = 1.0;

        if handle.scales_x() && time_range.abs() > 1e-6 {
            // Scale factor based on how much the drag expanded/contracted the bounds
            let expansion = match handle {
                BoundingBoxHandle::Left | BoundingBoxHandle::TopLeft | BoundingBoxHandle::BottomLeft => {
                    -delta_time // Moving left edge left expands
                }
                _ => delta_time, // Moving right edge right expands
            };
            time_scale = 1.0 + expansion / time_range;
            time_scale = time_scale.max(0.01); // Prevent negative/zero scale
        }

        if handle.scales_y() && value_range.abs() > 1e-6 {
            let expansion = match handle {
                BoundingBoxHandle::Top | BoundingBoxHandle::TopLeft | BoundingBoxHandle::TopRight => {
                    delta_value // Moving top edge up expands (note: delta_value is already negated for screen Y)
                }
                _ => -delta_value, // Moving bottom edge down expands
            };
            value_scale = 1.0 + (expansion / value_range) as f64;
            value_scale = value_scale.max(0.01);
        }

        Some((anchor_time, anchor_value, time_scale, value_scale))
    }

    fn keyframe_to_screen(&self, rect: Rect, kf: &KeyframeView) -> Pos2 {
        let x = self.space.unit_to_clipped(kf.position);
        let y = self.value_to_y(rect, kf.value);
        Pos2::new(x, y)
    }

    fn value_to_y(&self, rect: Rect, value: f32) -> f32 {
        let (min_val, max_val) = self.value_range;
        let value_range = max_val - min_val;
        if value_range.abs() < 1e-6 {
            return rect.center().y;
        }

        let normalized = (value - min_val) / value_range;
        let usable_height = rect.height() - 2.0 * self.config.vertical_padding;
        rect.bottom() - self.config.vertical_padding - normalized * usable_height
    }

    fn y_to_value(&self, rect: Rect, y: f32) -> f32 {
        let (min_val, max_val) = self.value_range;
        let value_range = max_val - min_val;

        let usable_height = rect.height() - 2.0 * self.config.vertical_padding;
        let normalized = (rect.bottom() - self.config.vertical_padding - y) / usable_height;
        min_val + normalized * value_range
    }
}
