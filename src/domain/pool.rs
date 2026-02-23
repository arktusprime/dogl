//! Pool, Lane, Stage, Quadrant: structure for organizing elements. All have uid for in-code references.
//! Aggregate invariant: sequence flow endpoints must be elements in this pool (review1 §2.3).

use std::collections::HashSet;

use crate::domain::element::Element;
use crate::domain::error::DomainError;
use crate::domain::flow::Flow;
use crate::domain::traits::Identifiable;
use crate::domain::value_objects::{LaneId, PoolId, StageId, Uid};

#[derive(Debug, Clone, PartialEq)]
pub struct Lane {
    pub uid: Uid,
    pub id: LaneId,
}

impl Lane {
    pub fn new(uid: Uid, id: impl Into<LaneId>) -> Self {
        Self { uid, id: id.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    pub uid: Uid,
    pub id: StageId,
}

impl Stage {
    pub fn new(uid: Uid, id: impl Into<StageId>) -> Self {
        Self { uid, id: id.into() }
    }
}

/// Lane × stage intersection: contains elements for that quadrant.
#[derive(Debug, Clone, PartialEq)]
pub struct Quadrant {
    pub uid: Uid,
    pub lane_id: LaneId,
    pub stage_id: StageId,
    pub elements: Vec<Element>,
}

impl Quadrant {
    pub fn new(uid: Uid, lane_id: impl Into<LaneId>, stage_id: impl Into<StageId>) -> Self {
        Self {
            uid,
            lane_id: lane_id.into(),
            stage_id: stage_id.into(),
            elements: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pool {
    pub uid: Uid,
    pub id: PoolId,
    pub lanes: Vec<Lane>,
    pub stages: Vec<Stage>,
    /// Elements live in lane×stage quadrants.
    pub quadrants: Vec<Quadrant>,
    /// Sequence flows within this pool (=>, =>d).
    pub sequence_flows: Vec<Flow>,
}

impl Pool {
    pub fn new(uid: Uid, id: impl Into<PoolId>) -> Self {
        Self {
            uid,
            id: id.into(),
            lanes: Vec::new(),
            stages: Vec::new(),
            quadrants: Vec::new(),
            sequence_flows: Vec::new(),
        }
    }

    /// Adds a sequence flow. Returns `Err` if from_uid or to_uid is not an element in this pool.
    pub fn add_sequence_flow(&mut self, flow: Flow) -> Result<(), DomainError> {
        let uids: HashSet<Uid> = self
            .quadrants
            .iter()
            .flat_map(|q| q.elements.iter().map(Identifiable::uid))
            .collect();
        if !uids.contains(&flow.from_uid) {
            return Err(DomainError::FlowEndpointNotFound { uid: flow.from_uid });
        }
        if !uids.contains(&flow.to_uid) {
            return Err(DomainError::FlowEndpointNotFound { uid: flow.to_uid });
        }
        self.sequence_flows.push(flow);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::element::{Element, Event, EventCode};
    use crate::domain::value_objects::name_from_id;

    #[test]
    fn add_sequence_flow_valid_endpoints_ok() {
        let uid_pool = 1u128;
        let uid_a = 10u128;
        let uid_b = 20u128;
        let mut pool = Pool::new(uid_pool, "P1");
        let quad = Quadrant::new(2, "L1", "S1");
        let mut quad = quad;
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
        let flow = Flow::new(100, uid_a, uid_b, crate::domain::value_objects::FlowType::Sequence);
        assert!(pool.add_sequence_flow(flow).is_ok());
        assert_eq!(pool.sequence_flows.len(), 1);
    }

    #[test]
    fn add_sequence_flow_unknown_endpoint_rejects() {
        let mut pool = Pool::new(1, "P1");
        let flow = Flow::new(100, 999, 998, crate::domain::value_objects::FlowType::Sequence);
        assert!(matches!(
            pool.add_sequence_flow(flow),
            Err(DomainError::FlowEndpointNotFound { .. })
        ));
    }
}
