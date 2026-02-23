//! Layout: map from entity Uid to Bounds. Data only; no layout calculation.
//! In .dogl file layout section only id is written; id↔uid mapping at parser/export boundary.
//! Create via `Layout::default()` only (review1 §2.4).

use std::collections::HashMap;

use crate::domain::value_objects::{Bounds, Uid};

/// Layout data: entity uid (element, pool, lane, or stage) → bounds.
/// Use `Layout::default()` to create.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    pub bounds_by_uid: HashMap<Uid, Bounds>,
}

impl Layout {
    pub fn set(&mut self, uid: Uid, bounds: Bounds) {
        self.bounds_by_uid.insert(uid, bounds);
    }

    pub fn get(&self, uid: Uid) -> Option<&Bounds> {
        self.bounds_by_uid.get(&uid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_empty_get_none() {
        let layout = Layout::default();
        assert!(layout.get(1).is_none());
    }

    #[test]
    fn set_get_roundtrip() {
        let mut layout = Layout::default();
        let bounds = Bounds::new(0.0, 0.0, 100.0, 50.0).unwrap();
        layout.set(42, bounds.clone());
        assert_eq!(layout.get(42), Some(&bounds));
    }
}
