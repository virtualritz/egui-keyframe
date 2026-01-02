//! Animation track containing a sequence of keyframes.

use super::keyframe::{Keyframe, KeyframeId};
use super::time::TimeTick;
use indexmap::IndexMap;
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Unique identifier for a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrackId(pub Uuid);

impl TrackId {
    /// Create a new random track ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TrackId {
    fn default() -> Self {
        Self::new()
    }
}

/// An animation track containing a sequence of keyframes for a single property.
///
/// The generic type `T` is the value type being animated.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Track<T> {
    /// Unique identifier for this track.
    pub id: TrackId,
    /// Keyframes indexed by their ID.
    keyframes: IndexMap<KeyframeId, Keyframe<T>>,
}

impl<T: Clone> Default for Track<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Track<T> {
    /// Create a new empty track.
    pub fn new() -> Self {
        Self {
            id: TrackId::new(),
            keyframes: IndexMap::new(),
        }
    }

    /// Create a track with a specific ID.
    pub fn with_id(id: TrackId) -> Self {
        Self {
            id,
            keyframes: IndexMap::new(),
        }
    }

    /// Add a keyframe to the track.
    ///
    /// Returns the keyframe ID.
    pub fn add_keyframe(&mut self, keyframe: Keyframe<T>) -> KeyframeId {
        let id = keyframe.id;
        self.keyframes.insert(id, keyframe);
        id
    }

    /// Remove a keyframe by ID.
    ///
    /// Returns the removed keyframe if it existed.
    pub fn remove_keyframe(&mut self, id: KeyframeId) -> Option<Keyframe<T>> {
        self.keyframes.shift_remove(&id)
    }

    /// Get a keyframe by ID.
    pub fn get_keyframe(&self, id: KeyframeId) -> Option<&Keyframe<T>> {
        self.keyframes.get(&id)
    }

    /// Get a mutable reference to a keyframe by ID.
    pub fn get_keyframe_mut(&mut self, id: KeyframeId) -> Option<&mut Keyframe<T>> {
        self.keyframes.get_mut(&id)
    }

    /// Get all keyframes sorted by position.
    pub fn keyframes_sorted(&self) -> Vec<&Keyframe<T>> {
        let mut keyframes: Vec<_> = self.keyframes.values().collect();
        keyframes.sort_by(|a, b| {
            a.position
                .partial_cmp(&b.position)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        keyframes
    }

    /// Get keyframes around a given position.
    ///
    /// Returns `(left, right)` where:
    /// - `left` is the keyframe at or before the position
    /// - `right` is the keyframe after the position
    pub fn keyframes_around(
        &self,
        position: impl Into<TimeTick>,
    ) -> (Option<&Keyframe<T>>, Option<&Keyframe<T>>) {
        let position = position.into();
        let sorted = self.keyframes_sorted();
        let mut left = None;
        let mut right = None;

        for kf in sorted {
            if kf.position <= position {
                left = Some(kf);
            } else if right.is_none() {
                right = Some(kf);
                break;
            }
        }

        (left, right)
    }

    /// Find the keyframe at the exact position, if any.
    pub fn keyframe_at_position(
        &self,
        position: impl Into<TimeTick>,
        tolerance: impl Into<TimeTick>,
    ) -> Option<&Keyframe<T>> {
        let position = position.into();
        let tolerance = tolerance.into();
        self.keyframes
            .values()
            .find(|kf| (kf.position - position).abs() < tolerance)
    }

    /// Get the time range covered by keyframes.
    ///
    /// Returns `None` if the track has no keyframes.
    pub fn time_range(&self) -> Option<(TimeTick, TimeTick)> {
        let sorted = self.keyframes_sorted();
        if sorted.is_empty() {
            return None;
        }
        Some((sorted.first()?.position, sorted.last()?.position))
    }

    /// Get the value range (min, max) across all keyframes.
    ///
    /// Only works for types that can be compared.
    pub fn value_range(&self) -> Option<(T, T)>
    where
        T: PartialOrd,
    {
        if self.keyframes.is_empty() {
            return None;
        }

        let mut min = None;
        let mut max = None;

        for kf in self.keyframes.values() {
            match &min {
                None => min = Some(kf.value.clone()),
                Some(m) if kf.value < *m => min = Some(kf.value.clone()),
                _ => {}
            }
            match &max {
                None => max = Some(kf.value.clone()),
                Some(m) if kf.value > *m => max = Some(kf.value.clone()),
                _ => {}
            }
        }

        Some((min?, max?))
    }

    /// Number of keyframes in the track.
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Check if the track has no keyframes.
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Iterate over all keyframes (unordered by position).
    pub fn iter(&self) -> impl Iterator<Item = &Keyframe<T>> {
        self.keyframes.values()
    }

    /// Iterate over all keyframe IDs.
    pub fn keyframe_ids(&self) -> impl Iterator<Item = KeyframeId> + '_ {
        self.keyframes.keys().copied()
    }

    /// Get all keyframe positions with their IDs.
    pub fn positions(&self) -> Vec<(KeyframeId, TimeTick)> {
        self.keyframes
            .iter()
            .map(|(id, kf)| (*id, kf.position))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_add_and_get() {
        let mut track = Track::<f32>::new();
        let kf1 = Keyframe::new(0.0, 10.0);
        let kf2 = Keyframe::new(1.0, 20.0);

        let id1 = track.add_keyframe(kf1);
        let id2 = track.add_keyframe(kf2);

        assert_eq!(track.len(), 2);
        assert_eq!(track.get_keyframe(id1).unwrap().value, 10.0);
        assert_eq!(track.get_keyframe(id2).unwrap().value, 20.0);
    }

    #[test]
    fn track_sorted() {
        let mut track = Track::<f32>::new();
        track.add_keyframe(Keyframe::new(2.0, 30.0));
        track.add_keyframe(Keyframe::new(0.0, 10.0));
        track.add_keyframe(Keyframe::new(1.0, 20.0));

        let sorted = track.keyframes_sorted();
        assert_eq!(sorted[0].position, TimeTick::new(0.0));
        assert_eq!(sorted[1].position, TimeTick::new(1.0));
        assert_eq!(sorted[2].position, TimeTick::new(2.0));
    }

    #[test]
    fn track_keyframes_around() {
        let mut track = Track::<f32>::new();
        track.add_keyframe(Keyframe::new(0.0, 10.0));
        track.add_keyframe(Keyframe::new(2.0, 30.0));

        let (left, right) = track.keyframes_around(1.0);
        assert_eq!(left.unwrap().position, TimeTick::new(0.0));
        assert_eq!(right.unwrap().position, TimeTick::new(2.0));

        let (left, right) = track.keyframes_around(0.0);
        assert_eq!(left.unwrap().position, TimeTick::new(0.0));
        assert_eq!(right.unwrap().position, TimeTick::new(2.0));
    }

    #[test]
    fn track_time_range() {
        let mut track = Track::<f32>::new();
        assert!(track.time_range().is_none());

        track.add_keyframe(Keyframe::new(1.0, 10.0));
        track.add_keyframe(Keyframe::new(5.0, 50.0));

        let (start, end) = track.time_range().unwrap();
        assert_eq!(start, TimeTick::new(1.0));
        assert_eq!(end, TimeTick::new(5.0));
    }

    #[test]
    fn track_value_range() {
        let mut track = Track::<f32>::new();
        track.add_keyframe(Keyframe::new(0.0, 10.0));
        track.add_keyframe(Keyframe::new(1.0, 50.0));
        track.add_keyframe(Keyframe::new(2.0, 30.0));

        let (min, max) = track.value_range().unwrap();
        assert_eq!(min, 10.0);
        assert_eq!(max, 50.0);
    }
}
