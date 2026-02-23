//! Value objects: identifiers, Uid, Position, Bounds, FlowType, SchemaVersion, Expression.
//! No I/O or external format dependencies. Invariant validation at construction (review1 §2.3).

/// Stable identity for entities within the aggregate. All in-code references use Uid.
/// Generated at parse/create time; id↔uid mapping at parser/export boundary.
pub type Uid = u128;

/// Identifier for a collab (name, PascalCase by convention). Notation and display; references in code use Uid.
pub type CollabId = String;

/// Identifier for a pool within a collab.
pub type PoolId = String;

/// Identifier for a lane within a pool.
pub type LaneId = String;

/// Identifier for a stage within a pool.
pub type StageId = String;

/// Identifier for an element within its scope (pool or lane×stage).
pub type ElementId = String;

/// Generate display name from id when not given in notation (review1 §2.5).
/// PascalCase → sentence, e.g. "StartOrder" → "Start order".
pub fn name_from_id(id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }
    let mut s = String::new();
    for (i, c) in id.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push(' ');
        }
        for lc in c.to_lowercase() {
            s.push(lc);
        }
    }
    if let Some(first) = s.chars().next() {
        let rest: String = s.chars().skip(1).collect();
        format!("{}{}", first.to_uppercase().collect::<String>(), rest)
    } else {
        s
    }
}

/// Source location for error reporting (e.g. parse errors). Invariant: line >= 1, column >= 1.
/// Only construct via [`Position::new`] so invalid state is unrepresentable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    line: usize,
    column: usize,
    offset: Option<usize>,
}

impl Position {
    /// Creates a position. Returns `Err` if line or column is 0 (invariant: >= 1).
    pub fn new(
        line: usize,
        column: usize,
    ) -> Result<Self, crate::domain::error::DomainError> {
        if line >= 1 && column >= 1 {
            Ok(Self {
                line,
                column,
                offset: None,
            })
        } else {
            Err(crate::domain::error::DomainError::InvalidPosition { line, column })
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }
    pub fn column(&self) -> usize {
        self.column
    }
    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Layout bounds (x, y, width, height). Used for elements, pools, lanes, stages.
/// Invariant: w >= 0, h >= 0. Only construct via [`Bounds::new`] so invalid state is unrepresentable.
#[derive(Debug, Clone, PartialEq)]
pub struct Bounds {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Bounds {
    /// Creates bounds. Returns `Err` if w or h is negative (invariant: >= 0).
    pub fn new(
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    ) -> Result<Self, crate::domain::error::DomainError> {
        if w >= 0.0 && h >= 0.0 {
            Ok(Self { x, y, w, h })
        } else {
            Err(crate::domain::error::DomainError::InvalidBounds { w, h })
        }
    }

    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }
    pub fn w(&self) -> f64 {
        self.w
    }
    pub fn h(&self) -> f64 {
        self.h
    }
}

/// Type of flow: sequence, default (from gateway), message (cross-pool), data association.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowType {
    Sequence,
    Default,
    Message,
    DataAssociation,
}

/// AST/domain schema version for compatibility. Used in DoglFile and JSON output so clients can
/// detect format version (e.g. when parsing JSON AST or exchanging with tools).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaVersion(pub String);

impl SchemaVersion {
    /// Current domain schema version (AST and JSON).
    pub const CURRENT: &'static str = "1.0";

    /// Returns the current schema version. Use when building DoglFile or when emitting JSON.
    pub fn current() -> Self {
        Self(Self::CURRENT.to_string())
    }

    /// Version string for serialization or display.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Expression attached to an element (e.g. @do, @do.exec, @dmn, @call). Domain stores only key and value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub key: String,
    pub value: String,
}

impl Expression {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::DomainError;

    #[test]
    fn name_from_id_pascal_to_sentence() {
        assert_eq!(name_from_id("StartOrder"), "Start order");
        assert_eq!(name_from_id("A"), "A");
        assert_eq!(name_from_id(""), "");
    }

    #[test]
    fn position_valid_accepts() {
        let p = Position::new(1, 1).unwrap();
        assert_eq!(p.line(), 1);
        assert_eq!(p.column(), 1);
        let p = Position::new(2, 10).unwrap();
        assert_eq!(p.line(), 2);
        assert_eq!(p.column(), 10);
    }

    #[test]
    fn position_invalid_rejects() {
        assert!(matches!(Position::new(0, 1), Err(DomainError::InvalidPosition { .. })));
        assert!(matches!(Position::new(1, 0), Err(DomainError::InvalidPosition { .. })));
    }

    #[test]
    fn bounds_valid_accepts() {
        let b = Bounds::new(0.0, 0.0, 10.0, 20.0).unwrap();
        assert_eq!(b.w(), 10.0);
        assert_eq!(b.h(), 20.0);
    }

    #[test]
    fn bounds_invalid_rejects() {
        assert!(Bounds::new(0.0, 0.0, -1.0, 1.0).is_err());
        assert!(Bounds::new(0.0, 0.0, 1.0, -0.1).is_err());
    }

    #[test]
    fn expression_new() {
        let e = Expression::new("do", "exec");
        assert_eq!(e.key, "do");
        assert_eq!(e.value, "exec");
    }

    #[test]
    fn schema_version_current_for_ast_json() {
        let v = SchemaVersion::current();
        assert_eq!(v.as_str(), SchemaVersion::CURRENT);
        assert_eq!(v.as_str(), "1.0");
    }
}
