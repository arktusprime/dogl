//! Domain invariant violations (review1 §2.3). No I/O; only state-based checks.
//! This module has no dependency on other domain types to avoid cycles.

/// Error when a domain invariant is violated at construction or mutation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DomainError {
    /// Position requires line >= 1 and column >= 1.
    #[error("invalid position: line={line}, column={column} (must be >= 1)")]
    InvalidPosition { line: usize, column: usize },
    /// Bounds requires w >= 0 and h >= 0.
    #[error("invalid bounds: w={w}, h={h} (w and h must be >= 0)")]
    InvalidBounds { w: f64, h: f64 },
    /// Dmn allows at most one default rule.
    #[error("Dmn allows at most one default rule")]
    DmnMultipleDefaults,
    /// Flow endpoint (from_uid or to_uid) is not an element in the aggregate.
    #[error("flow endpoint uid {uid} not found in aggregate")]
    FlowEndpointNotFound { uid: u64 },
    /// Layout section references a pool id that does not exist in the collab.
    #[error("layout: unknown pool id {pool_id:?}")]
    LayoutUnknownPoolId { pool_id: String },
    /// Layout section references a lane id that does not exist in the pool.
    #[error("layout: unknown lane id {lane_id:?} in pool {pool_id:?}")]
    LayoutUnknownLaneId { pool_id: String, lane_id: String },
    /// Layout section references a stage id that does not exist in the pool.
    #[error("layout: unknown stage id {stage_id:?} in pool {pool_id:?}")]
    LayoutUnknownStageId { pool_id: String, stage_id: String },
    /// Layout section references an element id that is not found in the pool.
    #[error("layout: unknown element id {element_id:?} in pool {pool_id:?}")]
    LayoutUnknownElementId { pool_id: String, element_id: String },
}
