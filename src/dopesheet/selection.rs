//! Selection state management for the DopeSheet.

use crate::core::keyframe::KeyframeId;
use crate::HashSet;

/// Selection state for the DopeSheet.
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Selected keyframe IDs.
    pub keyframes: HashSet<KeyframeId>,
    /// Selected row IDs.
    pub rows: HashSet<String>,
    /// Whether box selection is active.
    pub box_selecting: bool,
    /// Box selection start point (screen coordinates).
    pub box_start: Option<egui::Pos2>,
}

impl SelectionState {
    /// Create a new selection state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all selections.
    pub fn clear(&mut self) {
        self.keyframes.clear();
        self.rows.clear();
    }

    /// Clear keyframe selection only.
    pub fn clear_keyframes(&mut self) {
        self.keyframes.clear();
    }

    /// Clear row selection only.
    pub fn clear_rows(&mut self) {
        self.rows.clear();
    }

    /// Select a keyframe.
    pub fn select_keyframe(&mut self, id: KeyframeId, add_to_selection: bool) {
        if !add_to_selection {
            self.keyframes.clear();
        }
        self.keyframes.insert(id);
    }

    /// Toggle keyframe selection.
    pub fn toggle_keyframe(&mut self, id: KeyframeId) {
        if self.keyframes.contains(&id) {
            self.keyframes.remove(&id);
        } else {
            self.keyframes.insert(id);
        }
    }

    /// Select a row.
    pub fn select_row(&mut self, id: String, add_to_selection: bool) {
        if !add_to_selection {
            self.rows.clear();
        }
        self.rows.insert(id);
    }

    /// Toggle row selection.
    pub fn toggle_row(&mut self, id: &str) {
        if self.rows.contains(id) {
            self.rows.remove(id);
        } else {
            self.rows.insert(id.to_string());
        }
    }

    /// Select multiple keyframes.
    pub fn select_keyframes(&mut self, ids: impl IntoIterator<Item = KeyframeId>, add_to_selection: bool) {
        if !add_to_selection {
            self.keyframes.clear();
        }
        self.keyframes.extend(ids);
    }

    /// Check if a keyframe is selected.
    pub fn is_keyframe_selected(&self, id: &KeyframeId) -> bool {
        self.keyframes.contains(id)
    }

    /// Check if a row is selected.
    pub fn is_row_selected(&self, id: &str) -> bool {
        self.rows.contains(id)
    }

    /// Get the number of selected keyframes.
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Get the number of selected rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Start box selection.
    pub fn start_box_selection(&mut self, pos: egui::Pos2) {
        self.box_selecting = true;
        self.box_start = Some(pos);
    }

    /// End box selection.
    pub fn end_box_selection(&mut self) {
        self.box_selecting = false;
        self.box_start = None;
    }
}
