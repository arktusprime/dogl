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
        }
    }
}

impl std::error::Error for DomainError {}
