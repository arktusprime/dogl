//! Collab: aggregate root. Contains pools, message flows, DMN blocks, optional layout.
//! Aggregate invariant: message flow endpoints must be elements in this collab (review1 §2.3).

use std::collections::HashMap;

use crate::domain::dmn::Dmn;
use crate::domain::error::DomainError;
use crate::domain::flow::Flow;
use crate::domain::layout::{Layout, LayoutGroupedByPool, PoolLayoutData};
use crate::domain::pool::{all_element_uids, Pool};
use crate::domain::traits::Identifiable;
use crate::domain::value_objects::{CollabId, Uid};

#[derive(Debug, Clone, PartialEq)]
pub struct Collab {
    pub uid: Uid,
    pub id: CollabId,
    /// Pools (participants). If empty, one implicit pool is assumed at parse time.
    pub pools: Vec<Pool>,
    /// Cross-pool message flows (->).
    pub message_flows: Vec<Flow>,
    /// Standalone DMN decision tables referenced by gateways via @dmn: "Id".
    pub dmn_blocks: Vec<Dmn>,
    /// Optional layout data (bounds keyed by entity uid).
    pub layout: Option<Layout>,
}

impl Collab {
    pub fn new(uid: Uid, id: impl Into<CollabId>) -> Self {
        Self {
            uid,
            id: id.into(),
            pools: Vec::new(),
            message_flows: Vec::new(),
            dmn_blocks: Vec::new(),
            layout: None,
        }
    }

    /// Adds a message flow. Returns `Err` if from_uid or to_uid is not an element in this collab.
    pub fn add_message_flow(&mut self, flow: Flow) -> Result<(), DomainError> {
        let uids = all_element_uids(&self.pools);
        if !uids.contains(&flow.from_uid) {
            return Err(DomainError::FlowEndpointNotFound { uid: flow.from_uid });
        }
        if !uids.contains(&flow.to_uid) {
            return Err(DomainError::FlowEndpointNotFound { uid: flow.to_uid });
        }
        self.message_flows.push(flow);
        Ok(())
    }
}

/// Builds a flat Layout (uid → bounds) from grouped layout data. Fails if any id in the grouped
/// data does not exist in the collab (unknown pool, lane, stage, or element id).
/// Used when loading layout from JSON or other formats that use grouped, id-based layout.
pub fn layout_from_grouped(collab: &Collab, g: &LayoutGroupedByPool) -> Result<Layout, DomainError> {
    let mut layout = Layout::default();
    for (pool_id, data) in g {
        let pool = collab
            .pools
            .iter()
            .find(|p| p.id == *pool_id)
            .ok_or_else(|| DomainError::LayoutUnknownPoolId { pool_id: pool_id.clone() })?;
        if let Some(bounds) = &data.pool {
            layout.set(pool.uid, bounds.clone());
        }
        for (lane_id, bounds) in &data.lanes {
            let lane = pool
                .lanes
                .iter()
                .find(|l| l.id == *lane_id)
                .ok_or_else(|| DomainError::LayoutUnknownLaneId {
                    pool_id: pool_id.clone(),
                    lane_id: lane_id.clone(),
                })?;
            layout.set(lane.uid, bounds.clone());
        }
        for (stage_id, bounds) in &data.stages {
            let stage = pool
                .stages
                .iter()
                .find(|s| s.id == *stage_id)
                .ok_or_else(|| DomainError::LayoutUnknownStageId {
                    pool_id: pool_id.clone(),
                    stage_id: stage_id.clone(),
                })?;
            layout.set(stage.uid, bounds.clone());
        }
        for (element_id, bounds) in &data.elements {
            let element = pool
                .quadrants
                .iter()
                .flat_map(|q| q.elements.iter())
                .find(|e| e.id() == element_id)
                .ok_or_else(|| DomainError::LayoutUnknownElementId {
                    pool_id: pool_id.clone(),
                    element_id: element_id.clone(),
                })?;
            layout.set(element.uid(), bounds.clone());
        }
    }
    Ok(layout)
}

/// Builds the grouped layout (by pool, id-based) from the flat Layout and collab structure.
/// Used when serializing to JSON or other formats; `.dogl` export emits inline bounds per entity.
pub fn layout_to_grouped(collab: &Collab, layout: &Layout) -> LayoutGroupedByPool {
    let mut out = HashMap::new();
    for pool in &collab.pools {
        let pool_bounds = layout.get(pool.uid).cloned();
        let mut lanes = HashMap::new();
        for lane in &pool.lanes {
            if let Some(b) = layout.get(lane.uid) {
                lanes.insert(lane.id.clone(), b.clone());
            }
        }
        let mut stages = HashMap::new();
        for stage in &pool.stages {
            if let Some(b) = layout.get(stage.uid) {
                stages.insert(stage.id.clone(), b.clone());
            }
        }
        let mut elements = HashMap::new();
        for quad in &pool.quadrants {
            for el in &quad.elements {
                if let Some(b) = layout.get(el.uid()) {
                    elements.insert(el.id().to_string(), b.clone());
                }
            }
        }
        out.insert(
            pool.id.clone(),
            PoolLayoutData {
                pool: pool_bounds,
                lanes,
                stages,
                elements,
            },
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::element::{Element, Event, EventCode};
    use crate::domain::flow::Flow;
    use crate::domain::value_objects::{name_from_id, FlowType};

    fn collab_with_two_elements_in_pool() -> Collab {
        let uid_collab = 1u128;
        let uid_pool = 2u128;
        let uid_a = 10u128;
        let uid_b = 20u128;
        let mut pool = Pool::new(uid_pool, "P1");
        let mut quad = crate::domain::pool::Quadrant::new(3, "L1", "S1");
        quad.elements.push(Element::Event(Event {
            uid: uid_a,
            id: "A".to_string(),
            name: name_from_id("A"),
            code: EventCode::Start,
            expressions: vec![],
        }));
        quad.elements.push(Element::Event(Event {
            uid: uid_b,
            id: "B".to_string(),
            name: name_from_id("B"),
            code: EventCode::End,
            expressions: vec![],
        }));
        pool.quadrants.push(quad);
        let mut c = Collab::new(uid_collab, "C1");
        c.pools.push(pool);
        c
    }

    #[test]
    fn add_message_flow_valid_endpoints_ok() {
        let mut c = collab_with_two_elements_in_pool();
        let flow = Flow::new(100, 10, 20, FlowType::Message);
        assert!(c.add_message_flow(flow).is_ok());
        assert_eq!(c.message_flows.len(), 1);
    }

    #[test]
    fn add_message_flow_unknown_endpoint_rejects() {
        let mut c = collab_with_two_elements_in_pool();
        let flow = Flow::new(100, 999, 10, FlowType::Message);
        assert!(matches!(
            c.add_message_flow(flow),
            Err(DomainError::FlowEndpointNotFound { .. })
        ));
    }

    /// Collab with one pool, one lane, one stage, two elements (for layout conversion tests).
    fn collab_with_pool_lane_stage_and_elements() -> Collab {
        let uid_collab = 1u128;
        let uid_pool = 2u128;
        let uid_lane = 3u128;
        let uid_stage = 4u128;
        let uid_quad = 5u128;
        let uid_a = 10u128;
        let uid_b = 20u128;
        let mut pool = Pool::new(uid_pool, "P1");
        pool.lanes.push(crate::domain::pool::Lane::new(uid_lane, "L1"));
        pool.stages.push(crate::domain::pool::Stage::new(uid_stage, "S1"));
        let mut quad = crate::domain::pool::Quadrant::new(uid_quad, "L1", "S1");
        quad.elements.push(Element::Event(Event {
            uid: uid_a,
            id: "A".to_string(),
            name: name_from_id("A"),
            code: EventCode::Start,
            expressions: vec![],
        }));
        quad.elements.push(Element::Event(Event {
            uid: uid_b,
            id: "B".to_string(),
            name: name_from_id("B"),
            code: EventCode::End,
            expressions: vec![],
        }));
        pool.quadrants.push(quad);
        let mut c = Collab::new(uid_collab, "C1");
        c.pools.push(pool);
        c
    }

    #[test]
    fn layout_from_grouped_valid_builds_flat_layout() {
        use crate::domain::layout::PoolLayoutData;
        use crate::domain::value_objects::Bounds;
        use std::collections::HashMap;

        let collab = collab_with_pool_lane_stage_and_elements();
        let pool_bounds = Bounds::new(0.0, 0.0, 400.0, 300.0).unwrap();
        let lane_bounds = Bounds::new(0.0, 50.0, 400.0, 60.0).unwrap();
        let stage_bounds = Bounds::new(100.0, 0.0, 120.0, 300.0).unwrap();
        let elem_a_bounds = Bounds::new(10.0, 20.0, 80.0, 40.0).unwrap();
        let elem_b_bounds = Bounds::new(200.0, 20.0, 80.0, 40.0).unwrap();
        let mut lanes = HashMap::new();
        lanes.insert("L1".to_string(), lane_bounds.clone());
        let mut stages = HashMap::new();
        stages.insert("S1".to_string(), stage_bounds.clone());
        let mut elements = HashMap::new();
        elements.insert("A".to_string(), elem_a_bounds.clone());
        elements.insert("B".to_string(), elem_b_bounds.clone());
        let mut grouped = HashMap::new();
        grouped.insert(
            "P1".to_string(),
            PoolLayoutData {
                pool: Some(pool_bounds.clone()),
                lanes,
                stages,
                elements,
            },
        );
        let layout = layout_from_grouped(&collab, &grouped).unwrap();
        let pool = &collab.pools[0];
        let lane = &pool.lanes[0];
        let stage = &pool.stages[0];
        assert_eq!(layout.get(pool.uid), Some(&pool_bounds));
        assert_eq!(layout.get(lane.uid), Some(&lane_bounds));
        assert_eq!(layout.get(stage.uid), Some(&stage_bounds));
        assert_eq!(layout.get(10), Some(&elem_a_bounds));
        assert_eq!(layout.get(20), Some(&elem_b_bounds));
    }

    #[test]
    fn layout_from_grouped_unknown_pool_id_rejects() {
        use crate::domain::layout::PoolLayoutData;
        use std::collections::HashMap;

        let collab = collab_with_pool_lane_stage_and_elements();
        let mut grouped = HashMap::new();
        grouped.insert("NoSuchPool".to_string(), PoolLayoutData::default());
        let err = layout_from_grouped(&collab, &grouped).unwrap_err();
        assert!(matches!(err, DomainError::LayoutUnknownPoolId { pool_id } if pool_id == "NoSuchPool"));
    }

    #[test]
    fn layout_from_grouped_unknown_lane_id_rejects() {
        use crate::domain::layout::PoolLayoutData;
        use crate::domain::value_objects::Bounds;
        use std::collections::HashMap;

        let collab = collab_with_pool_lane_stage_and_elements();
        let mut lanes = HashMap::new();
        lanes.insert("NoSuchLane".to_string(), Bounds::new(0.0, 0.0, 100.0, 50.0).unwrap());
        let mut grouped = HashMap::new();
        grouped.insert(
            "P1".to_string(),
            PoolLayoutData {
                pool: None,
                lanes,
                stages: HashMap::new(),
                elements: HashMap::new(),
            },
        );
        let err = layout_from_grouped(&collab, &grouped).unwrap_err();
        assert!(matches!(
            err,
            DomainError::LayoutUnknownLaneId { pool_id, lane_id }
                if pool_id == "P1" && lane_id == "NoSuchLane"
        ));
    }

    #[test]
    fn layout_from_grouped_unknown_stage_id_rejects() {
        use crate::domain::layout::PoolLayoutData;
        use crate::domain::value_objects::Bounds;
        use std::collections::HashMap;

        let collab = collab_with_pool_lane_stage_and_elements();
        let mut stages = HashMap::new();
        stages.insert("NoSuchStage".to_string(), Bounds::new(0.0, 0.0, 120.0, 300.0).unwrap());
        let mut grouped = HashMap::new();
        grouped.insert(
            "P1".to_string(),
            PoolLayoutData {
                pool: None,
                lanes: HashMap::new(),
                stages,
                elements: HashMap::new(),
            },
        );
        let err = layout_from_grouped(&collab, &grouped).unwrap_err();
        assert!(matches!(
            err,
            DomainError::LayoutUnknownStageId { pool_id, stage_id }
                if pool_id == "P1" && stage_id == "NoSuchStage"
        ));
    }

    #[test]
    fn layout_from_grouped_unknown_element_id_rejects() {
        use crate::domain::layout::PoolLayoutData;
        use crate::domain::value_objects::Bounds;
        use std::collections::HashMap;

        let collab = collab_with_pool_lane_stage_and_elements();
        let mut elements = HashMap::new();
        elements.insert("NoSuchElement".to_string(), Bounds::new(0.0, 0.0, 10.0, 10.0).unwrap());
        let mut grouped = HashMap::new();
        grouped.insert(
            "P1".to_string(),
            PoolLayoutData {
                pool: None,
                lanes: HashMap::new(),
                stages: HashMap::new(),
                elements,
            },
        );
        let err = layout_from_grouped(&collab, &grouped).unwrap_err();
        assert!(matches!(
            err,
            DomainError::LayoutUnknownElementId {
                pool_id,
                element_id
            } if pool_id == "P1" && element_id == "NoSuchElement"
        ));
    }

    #[test]
    fn layout_to_grouped_roundtrip() {
        use crate::domain::value_objects::Bounds;

        let collab = collab_with_pool_lane_stage_and_elements();
        let mut layout = Layout::default();
        let pool = &collab.pools[0];
        let lane = &pool.lanes[0];
        let stage = &pool.stages[0];
        let pool_bounds = Bounds::new(0.0, 0.0, 400.0, 300.0).unwrap();
        let lane_bounds = Bounds::new(0.0, 50.0, 400.0, 60.0).unwrap();
        let stage_bounds = Bounds::new(100.0, 0.0, 120.0, 300.0).unwrap();
        let elem_a_bounds = Bounds::new(10.0, 20.0, 80.0, 40.0).unwrap();
        layout.set(pool.uid, pool_bounds.clone());
        layout.set(lane.uid, lane_bounds.clone());
        layout.set(stage.uid, stage_bounds.clone());
        layout.set(10, elem_a_bounds.clone());
        let grouped = layout_to_grouped(&collab, &layout);
        assert_eq!(grouped.len(), 1);
        let data = grouped.get("P1").unwrap();
        assert_eq!(data.pool.as_ref(), Some(&pool_bounds));
        assert_eq!(data.lanes.get("L1"), Some(&lane_bounds));
        assert_eq!(data.stages.get("S1"), Some(&stage_bounds));
        assert_eq!(data.elements.get("A"), Some(&elem_a_bounds));
        assert!(data.elements.get("B").is_none());
    }
}
