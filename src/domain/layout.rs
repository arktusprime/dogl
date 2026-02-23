//! **Layout group** (domain): bounds `{ x, y, w, h }` for diagram positioning. Data only; no layout calculation.
//!
//! Covers:
//! - **Elements** (Event, Task, Gateway, Artifact) — shape position per uid.
//! - **Pool, Lane, Stage** — each has a uid; their bounds are stored in the same map for BPMN diagram layout (BPMNDI).
//!
//! **Grouped representation:** In the `.dogl` file the layout section is scoped to the collab and grouped by pool.
//! [`LayoutGroupedByPool`] and [`PoolLayoutData`] model that structure (id-based). Convert to/from flat [`Layout`]
//! (uid-based) via [`crate::domain::collab::layout_from_grouped`] and [`crate::domain::collab::layout_to_grouped`].
//! Create via `Layout::default()` only (review1 §2.4).

use std::collections::HashMap;

use crate::domain::value_objects::{Bounds, ElementId, LaneId, PoolId, StageId, Uid};

/// Layout data: entity uid → bounds. Keys are uids of **elements**, **Pool**, **Lane**, or **Stage**.
/// Use `Layout::default()` to create.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    pub bounds_by_uid: HashMap<Uid, Bounds>,
}

impl Layout {
    /// Sets bounds for any layoutable entity (element, Pool, Lane, or Stage) by uid.
    pub fn set(&mut self, uid: Uid, bounds: Bounds) {
        self.bounds_by_uid.insert(uid, bounds);
    }

    /// Returns bounds for the given entity uid, if present.
    pub fn get(&self, uid: Uid) -> Option<&Bounds> {
        self.bounds_by_uid.get(&uid)
    }
}

/// Layout data for one pool: bounds for the pool shape, each lane, each stage, and each element (by id).
/// Mirrors the structure of the layout section in `.dogl` (grouped by pool).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PoolLayoutData {
    /// Bounds of the pool participant shape.
    pub pool: Option<Bounds>,
    /// Lane id → bounds.
    pub lanes: HashMap<LaneId, Bounds>,
    /// Stage id → bounds.
    pub stages: HashMap<StageId, Bounds>,
    /// Element id → bounds (elements are scoped to this pool).
    pub elements: HashMap<ElementId, Bounds>,
}

/// Layout section grouped by pool (pool id → per-pool layout). Used at parser/export boundary for `.dogl` and JSON.
pub type LayoutGroupedByPool = HashMap<PoolId, PoolLayoutData>;

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

    #[test]
    fn layout_holds_bounds_for_element_pool_lane_stage() {
        let mut layout = Layout::default();
        let elem_bounds = Bounds::new(10.0, 20.0, 80.0, 40.0).unwrap();
        let pool_bounds = Bounds::new(0.0, 0.0, 400.0, 300.0).unwrap();
        let lane_bounds = Bounds::new(0.0, 50.0, 400.0, 60.0).unwrap();
        let stage_bounds = Bounds::new(100.0, 0.0, 120.0, 300.0).unwrap();
        layout.set(1, elem_bounds.clone());   // e.g. element uid
        layout.set(2, pool_bounds.clone());  // e.g. pool uid
        layout.set(3, lane_bounds.clone());  // e.g. lane uid
        layout.set(4, stage_bounds.clone()); // e.g. stage uid
        assert_eq!(layout.get(1), Some(&elem_bounds));
        assert_eq!(layout.get(2), Some(&pool_bounds));
        assert_eq!(layout.get(3), Some(&lane_bounds));
        assert_eq!(layout.get(4), Some(&stage_bounds));
    }
}
