//! Flow: sequence, default, message, data association. All references by Uid.
//! Single canonical constructor: `Flow::new(uid, from_uid, to_uid, flow_type)` (review1 §2.2).
//! The four helpers below are thin wrappers for call-site convenience only.

use crate::domain::value_objects::{FlowType, Uid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    pub uid: Uid,
    pub from_uid: Uid,
    pub to_uid: Uid,
    pub flow_type: FlowType,
    /// Optional condition (e.g. for gateway branch); DMN routing uses Dmn rules, not this.
    pub condition: Option<String>,
}

impl Flow {
    /// Single way to create a flow. Use this or construct with literal; wrappers below are optional.
    pub fn new(uid: Uid, from_uid: Uid, to_uid: Uid, flow_type: FlowType) -> Self {
        Self {
            uid,
            from_uid,
            to_uid,
            flow_type,
            condition: None,
        }
    }

    /// Convenience: `Flow::new(uid, from_uid, to_uid, FlowType::Sequence)`.
    pub fn sequence(uid: Uid, from_uid: Uid, to_uid: Uid) -> Self {
        Self::new(uid, from_uid, to_uid, FlowType::Sequence)
    }

    /// Convenience: `Flow::new(uid, from_uid, to_uid, FlowType::Default)`.
    pub fn default_flow(uid: Uid, from_uid: Uid, to_uid: Uid) -> Self {
        Self::new(uid, from_uid, to_uid, FlowType::Default)
    }

    /// Convenience: `Flow::new(uid, from_uid, to_uid, FlowType::Message)`.
    pub fn message(uid: Uid, from_uid: Uid, to_uid: Uid) -> Self {
        Self::new(uid, from_uid, to_uid, FlowType::Message)
    }

    /// Convenience: `Flow::new(uid, from_uid, to_uid, FlowType::DataAssociation)`.
    pub fn data_association(uid: Uid, from_uid: Uid, to_uid: Uid) -> Self {
        Self::new(uid, from_uid, to_uid, FlowType::DataAssociation)
    }

    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_canonical_sequence_wrapper_delegates() {
        let f = Flow::new(1, 10, 20, FlowType::Sequence);
        assert_eq!(f.uid, 1);
        assert_eq!(f.from_uid, 10);
        assert_eq!(f.to_uid, 20);
        assert_eq!(f.flow_type, FlowType::Sequence);
        let g = Flow::sequence(2, 11, 21);
        assert_eq!(g.flow_type, FlowType::Sequence);
        assert_eq!(g.from_uid, 11);
    }

    #[test]
    fn with_condition() {
        let f = Flow::new(1, 10, 20, FlowType::Default).with_condition("x > 0");
        assert_eq!(f.condition.as_deref(), Some("x > 0"));
    }
}
