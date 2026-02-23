//! Domain invariant violations (review1 §2.3). No I/O; only state-based checks.
//! This module has no dependency on other domain types to avoid cycles.

/// Error when a domain invariant is violated at construction or mutation.
#[derive(Debug, Clone, PartialEq)]
pub enum DomainError {
    /// Position requires line >= 1 and column >= 1.
    InvalidPosition { line: usize, column: usize },
    /// Bounds requires w >= 0 and h >= 0.
    InvalidBounds { w: f64, h: f64 },
    /// Dmn allows at most one default rule.
    DmnMultipleDefaults,
    /// Flow endpoint (from_uid or to_uid) is not an element in the aggregate.
    FlowEndpointNotFound { uid: u128 },
    /// Layout section references a pool id that does not exist in the collab.
    LayoutUnknownPoolId { pool_id: String },
    /// Layout section references a lane id that does not exist in the pool.
    LayoutUnknownLaneId { pool_id: String, lane_id: String },
    /// Layout section references a stage id that does not exist in the pool.
    LayoutUnknownStageId { pool_id: String, stage_id: String },
    /// Layout section references an element id that is not found in the pool.
    LayoutUnknownElementId { pool_id: String, element_id: String },
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::InvalidPosition { line, column } => {
                write!(f, "invalid position: line={}, column={} (must be >= 1)", line, column)
            }
            DomainError::InvalidBounds { w, h } => {
                write!(f, "invalid bounds: w={}, h={} (w and h must be >= 0)", w, h)
            }
            DomainError::DmnMultipleDefaults => {
                write!(f, "Dmn allows at most one default rule")
            }
            DomainError::FlowEndpointNotFound { uid } => {
                write!(f, "flow endpoint uid {} not found in aggregate", uid)
            }
            DomainError::LayoutUnknownPoolId { pool_id } => {
                write!(f, "layout: unknown pool id {:?}", pool_id)
            }
            DomainError::LayoutUnknownLaneId { pool_id, lane_id } => {
                write!(f, "layout: unknown lane id {:?} in pool {:?}", lane_id, pool_id)
            }
            DomainError::LayoutUnknownStageId { pool_id, stage_id } => {
                write!(f, "layout: unknown stage id {:?} in pool {:?}", stage_id, pool_id)
            }
            DomainError::LayoutUnknownElementId { pool_id, element_id } => {
                write!(f, "layout: unknown element id {:?} in pool {:?}", element_id, pool_id)
            }
        }
    }
}

impl std::error::Error for DomainError {}
