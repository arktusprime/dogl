//! Collab: aggregate root. Contains pools, message flows, DMN blocks, optional layout.
//! Aggregate invariant: message flow endpoints must be elements in this collab (review1 §2.3).

use std::collections::HashSet;

use crate::domain::dmn::Dmn;
use crate::domain::error::DomainError;
use crate::domain::flow::Flow;
use crate::domain::layout::Layout;
use crate::domain::pool::Pool;
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
        let uids: HashSet<Uid> = self
            .pools
            .iter()
            .flat_map(|p| {
                p.quadrants
                    .iter()
                    .flat_map(|q| q.elements.iter().map(Identifiable::uid))
            })
            .collect();
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
}
